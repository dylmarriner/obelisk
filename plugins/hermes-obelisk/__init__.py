"""Hermes Agent plugin for Obelisk.

Install by copying this directory to `~/.hermes/plugins/obelisk/`, then enable it:

    hermes plugins enable obelisk

The plugin exposes Obelisk as Hermes tools, slash commands, skills, and a
best-effort pre_tool_call hook.
"""

from __future__ import annotations

from pathlib import Path

from .schemas import TOOL_SCHEMAS
from .tools import handle_tool, pre_tool_call, slash_obelisk, slash_obelisk_doctor, slash_obelisk_stats

PLUGIN_ROOT = Path(__file__).parent


def _register_tools(ctx) -> None:
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


def _register_commands(ctx) -> None:
    if not hasattr(ctx, "register_command"):
        return

    ctx.register_command("obelisk", slash_obelisk, "Show Obelisk plugin help and available tools.")
    ctx.register_command("obelisk-stats", slash_obelisk_stats, "Show Obelisk token savings stats.")
    ctx.register_command("obelisk-doctor", slash_obelisk_doctor, "Check Obelisk installation status.")


def _register_cli_commands(ctx) -> None:
    if not hasattr(ctx, "register_cli_command"):
        return

    def setup(_parser):
        return None

    def doctor(_args):
        return slash_obelisk_doctor()

    def stats(_args):
        return slash_obelisk_stats()

    ctx.register_cli_command("obelisk-doctor", "Check Obelisk installation status", setup, doctor)
    ctx.register_cli_command("obelisk-stats", "Show Obelisk token savings stats", setup, stats)


def _register_skills(ctx) -> None:
    if not hasattr(ctx, "register_skill"):
        return

    skill_dir = PLUGIN_ROOT / "skills"
    for name in ["pack-context", "inspect-symbol", "compact-output", "restore-context"]:
        path = skill_dir / f"{name}.md"
        if path.exists():
            ctx.register_skill(name, str(path))


def register(ctx) -> None:
    _register_tools(ctx)

    if hasattr(ctx, "register_hook"):
        ctx.register_hook("pre_tool_call", pre_tool_call)

    _register_commands(ctx)
    _register_cli_commands(ctx)
    _register_skills(ctx)
