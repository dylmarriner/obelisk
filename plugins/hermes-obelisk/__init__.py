"""Obelisk Hermes plugin — unified token optimization.

Combines command-output compression with per-turn token tracking,
context-fill nudges, and session rollup.

Plugin layout:
  ~/.hermes/plugins/obelisk/
      __init__.py              (this file — merged registration)
      plugin.yaml              (hooks + metadata)
      README.md
      schemas.py               (obelisk tool schemas)
      tools.py                 (obelisk tool handlers)
      hermes_hook_bridge.py    (TO bridge → measure.py)
      hermes_state.py          (read-only state.db reader)
      hermes_session.py        (session normalizer + quality scorer)
      runtime_env.py           (runtime detection helpers)
      token_estimate.py        (char→token estimator)
      utf8_io.py               (UTF-8 I/O enforcement)
      measure-path             (one-line locator → measure.py in the TO checkout)
      skills/                  (bundled obelisk skills)
"""

from __future__ import annotations

import logging
import sys
import threading
from pathlib import Path
from typing import Any

from .schemas import TOOL_SCHEMAS
from .tools import handle_tool, pre_tool_call, slash_obelisk, slash_obelisk_doctor, slash_obelisk_stats

logger = logging.getLogger(__name__)

PLUGIN_ROOT = Path(__file__).parent.resolve()

# ---------------------------------------------------------------------------
# Usage tracking integration: resolve sibling modules
# ---------------------------------------------------------------------------

_PLUGIN_DIR_STR = str(PLUGIN_ROOT)
sys.path[:] = [p for p in sys.path if p != _PLUGIN_DIR_STR]
sys.path.append(_PLUGIN_DIR_STR)

_BRIDGE_SENTINEL: Any = object()
_bridge_cache: Any = _BRIDGE_SENTINEL


def _import_bridge():
    global _bridge_cache
    if _bridge_cache is not _BRIDGE_SENTINEL:
        return _bridge_cache
    try:
        import hermes_hook_bridge as _bridge  # noqa: PLC0415
        _bridge_cache = _bridge
        return _bridge
    except Exception as exc:
        logger.debug("[obelisk] hermes_hook_bridge not available: %s", exc)
        return None


# ---------------------------------------------------------------------------
# TO: Per-session token accumulation (thread-safe, in-process)
# ---------------------------------------------------------------------------

_LOCK = threading.Lock()
_TALLY: dict[str, dict[str, int]] = {}
_NUDGED: set[str] = set()
_ROLLED_UP: set[str] = set()

_NUDGE_THRESHOLD = 0.70
_DEFAULT_CONTEXT_WINDOW = 200_000

try:
    from hermes_doctor import DASHBOARD_PORT as _DASHBOARD_PORT  # noqa: PLC0415
except Exception:
    _DASHBOARD_PORT = 24844


def _context_window(model: str) -> int:
    try:
        from hermes_session import context_window_for_model  # noqa: PLC0415
        return context_window_for_model(model or "")
    except Exception:
        return _DEFAULT_CONTEXT_WINDOW


def _estimate_fill_from_history(conversation_history: list[Any]) -> int:
    chars = 0
    for msg in (conversation_history or []):
        if not isinstance(msg, dict):
            continue
        content = msg.get("content")
        if isinstance(content, str):
            chars += len(content)
        elif isinstance(content, list):
            for part in content:
                if isinstance(part, dict):
                    chars += len(str(part.get("text") or ""))
    return int(chars / 3.3)


def _quality_grade(fill_ratio: float, message_count: int, model: str = "", ctx_win: int = 0) -> str:
    try:
        from hermes_session import compute_quality_score as _cqs  # noqa: PLC0415
        window = ctx_win if ctx_win > 0 else _DEFAULT_CONTEXT_WINDOW
        approx_input = int(fill_ratio * window)
        result = _cqs(
            input_tokens=approx_input,
            output_tokens=0,
            message_count=message_count,
            model=model or "",
            context_window=window,
        )
        return result["grade"]
    except Exception:
        if fill_ratio < 0.30 and message_count <= 20:
            return "S"
        if fill_ratio < 0.50 and message_count <= 40:
            return "A"
        if fill_ratio < 0.70 and message_count <= 60:
            return "B"
        if fill_ratio < 0.85 and message_count <= 100:
            return "C"
        if fill_ratio < 0.95:
            return "D"
        return "F"


# ---------------------------------------------------------------------------
# TO Hook: post_api_request — accumulate per-turn token usage
# ---------------------------------------------------------------------------

def on_post_api_request(**kwargs: Any) -> None:
    try:
        session_id: str = kwargs.get("session_id") or ""
        if not session_id:
            return
        usage = kwargs.get("usage") or {}
        if not isinstance(usage, dict):
            return
        delta = {
            "input":       int(usage.get("input_tokens", 0) or 0),
            "output":      int(usage.get("output_tokens", 0) or 0),
            "cache_read":  int(usage.get("cache_read_tokens", 0) or 0),
            "cache_write": int(usage.get("cache_write_tokens", 0) or 0),
            "reasoning":   int(usage.get("reasoning_tokens", 0) or 0),
        }
        with _LOCK:
            tally = _TALLY.setdefault(session_id, {
                "input": 0, "output": 0,
                "cache_read": 0, "cache_write": 0, "reasoning": 0,
            })
            for k, v in delta.items():
                tally[k] += v
    except Exception as exc:
        logger.debug("[obelisk] post_api_request accumulation error: %s", exc)


# ---------------------------------------------------------------------------
# TO Hook: pre_llm_call — context nudge
# ---------------------------------------------------------------------------

def on_pre_llm_call(**kwargs: Any) -> dict[str, str] | None:
    try:
        session_id: str = kwargs.get("session_id") or ""
        model: str = kwargs.get("model") or ""
        conversation_history = kwargs.get("conversation_history") or []
        message_count = len(conversation_history) if isinstance(conversation_history, list) else 0

        with _LOCK:
            already_nudged = session_id in _NUDGED
            tally = dict(_TALLY.get(session_id) or {})

        if already_nudged:
            return None

        tally_input = tally.get("input", 0)
        if tally_input > 0:
            current_input = tally_input
        else:
            current_input = _estimate_fill_from_history(conversation_history)

        ctx_win = _context_window(model)
        fill = current_input / ctx_win if ctx_win > 0 else 0.0

        if fill < _NUDGE_THRESHOLD:
            return None

        grade = _quality_grade(fill, message_count, model=model, ctx_win=ctx_win)
        fill_pct = min(100, int(fill * 100))
        tip = (
            "Consider /compact to free context."
            if fill >= 0.85
            else "Avoid adding large files; prefer targeted reads."
        )
        nudge = (
            f"[Obelisk] Context ~{fill_pct}% full "
            f"(~{current_input:,} input tokens vs assumed {ctx_win:,} window) "
            f"Grade: {grade}. {tip}"
        )

        with _LOCK:
            _NUDGED.add(session_id)

        return {"context": nudge}
    except Exception as exc:
        logger.debug("[obelisk] pre_llm_call nudge error: %s", exc)
        return None


# ---------------------------------------------------------------------------
# TO Hook: session finalize / end — rollup to trends.db
# ---------------------------------------------------------------------------

def _do_rollup(session_id: str, platform: str, reason: str) -> None:
    if not session_id:
        return
    with _LOCK:
        if session_id in _ROLLED_UP:
            logger.debug("[obelisk] rollup already fired for %s, skipping", session_id)
            return
        _ROLLED_UP.add(session_id)
    bridge = _import_bridge()
    if bridge is None:
        logger.debug("[obelisk] bridge unavailable, skipping rollup for %s", session_id)
    else:
        try:
            bridge.run_rollup(session_id=session_id, platform=platform, reason=reason)
        except Exception as exc:
            logger.debug("[obelisk] rollup error for %s: %s", session_id, exc)
    with _LOCK:
        _TALLY.pop(session_id, None)
        _NUDGED.discard(session_id)


def on_session_finalize(**kwargs: Any) -> None:
    try:
        session_id: str = kwargs.get("session_id") or ""
        platform: str = kwargs.get("platform") or "hermes"
        reason: str = kwargs.get("reason") or ""
        _do_rollup(session_id, platform, reason)
    except Exception as exc:
        logger.debug("[obelisk] on_session_finalize error: %s", exc)


def on_session_end(**kwargs: Any) -> None:
    try:
        session_id: str = kwargs.get("session_id") or ""
        platform: str = kwargs.get("platform") or "hermes"
        reason: str = kwargs.get("reason") or ""
        _do_rollup(session_id, platform, reason)
    except Exception as exc:
        logger.debug("[obelisk] on_session_end error: %s", exc)


# ---------------------------------------------------------------------------
# Slash command: /obelisk-token — token/cost summary
# ---------------------------------------------------------------------------

def _handle_token_command(args: str = "", **kwargs: Any) -> str:
    """Show a token/cost summary for the current or most-recent session."""
    try:
        bridge = _import_bridge()
        if bridge is None:
            return "[Obelisk] Token tracking bridge not available."
        session_id: str = kwargs.get("session_id") or ""
        result = bridge.run_summary(session_id=session_id)
        return result or "[Obelisk] No session data available yet. Complete a session first."
    except Exception as exc:
        logger.debug("[obelisk] token command handler error: %s", exc)
        return f"[Obelisk] Token summary error: {exc}"


# ---------------------------------------------------------------------------
# CLI command: hermes obelisk-token — open dashboard
# ---------------------------------------------------------------------------

def _setup_cli_token(subparser: Any) -> None:
    try:
        subparser.add_argument(
            "--port", type=int, default=_DASHBOARD_PORT,
            help=f"Dashboard port (default: {_DASHBOARD_PORT})",
        )
        subparser.add_argument(
            "--session", default="",
            help="Session ID to summarise (default: most recent)",
        )
    except Exception:
        pass


def _handle_cli_token(args: Any) -> None:
    try:
        bridge = _import_bridge()
        if bridge is None:
            print("[Obelisk] Token bridge not available.")
            return
        port = getattr(args, "port", _DASHBOARD_PORT)
        session_id = getattr(args, "session", "") or ""
        bridge.run_dashboard(session_id=session_id, port=port)
    except Exception as exc:
        logger.debug("[obelisk] CLI token handler error: %s", exc)
        print(f"[Obelisk] Error: {exc}")


# ---------------------------------------------------------------------------
# Plugin entry point
# ---------------------------------------------------------------------------

def register(ctx) -> None:
    """Register Obelisk tools, commands, skills, hooks, and CLI subcommands."""

    # --- Obelisk tools ---
    for schema in TOOL_SCHEMAS:
        name = schema["name"]

        def _handler(params, _name=name, **kwargs):
            return handle_tool(_name, params, **kwargs)

        ctx.register_tool(
            name=name,
            toolset="obelisk",
            schema=schema,
            handler=_handler,
            description=schema.get("description", "Obelisk tool"),
        )

    # --- Obelisk pre_tool_call hook ---
    if hasattr(ctx, "register_hook"):
        ctx.register_hook("pre_tool_call", pre_tool_call)

    # --- Usage tracking hooks ---
    if hasattr(ctx, "register_hook"):
        ctx.register_hook("post_api_request", on_post_api_request)
        ctx.register_hook("pre_llm_call", on_pre_llm_call)
        ctx.register_hook("on_session_finalize", on_session_finalize)
        ctx.register_hook("on_session_end", on_session_end)

    # --- Obelisk slash commands ---
    if hasattr(ctx, "register_command"):
        ctx.register_command("obelisk", slash_obelisk, "Show Obelisk plugin help and available tools.")
        ctx.register_command("obelisk-stats", slash_obelisk_stats, "Show Obelisk token savings stats.")
        ctx.register_command("obelisk-doctor", slash_obelisk_doctor, "Check Obelisk installation status.")
        ctx.register_command("obelisk-token", _handle_token_command,
                            "Show token/cost summary for recent sessions.",
                            args_hint="[session_id]")

    # --- Obelisk CLI commands ---
    if hasattr(ctx, "register_cli_command"):
        def _setup_doctor(_parser):
            return None

        def _doctor(_args):
            return slash_obelisk_doctor()

        def _setup_stats(_parser):
            return None

        def _stats(_args):
            return slash_obelisk_stats()

        ctx.register_cli_command("obelisk-doctor", "Check Obelisk installation status", _setup_doctor, _doctor)
        ctx.register_cli_command("obelisk-stats", "Show Obelisk token savings stats", _setup_stats, _stats)
        ctx.register_cli_command("obelisk-token",
                                f"Open the usage dashboard (port {_DASHBOARD_PORT}).",
                                _setup_cli_token, _handle_cli_token)

    # --- Obelisk skills ---
    if hasattr(ctx, "register_skill"):
        skill_dir = PLUGIN_ROOT / "skills"
        for name in ["pack-context", "inspect-symbol", "compact-output", "restore-context"]:
            path = skill_dir / f"{name}.md"
            if path.exists():
                ctx.register_skill(name, str(path))
