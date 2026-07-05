"""Safe Hermes tool handlers that delegate to the Obelisk binary.

This plugin intentionally keeps Obelisk as the engine. Hermes gets a thin
adapter layer for tools, commands, and hooks. No shell=True. No heroic ideas.
"""

from __future__ import annotations

import json
import os
import re
import shlex
import shutil
import subprocess
from pathlib import Path
from typing import Any, Iterable

DANGEROUS_PROGRAMS = {
    "rm",
    "rmdir",
    "mv",
    "cp",
    "chmod",
    "chown",
    "dd",
    "mkfs",
    "mount",
    "umount",
    "sudo",
    "su",
    "ssh",
    "scp",
    "rsync",
    "curl",
    "wget",
    "nc",
    "ncat",
    "telnet",
    "ftp",
    "sftp",
    "python",
    "python3",
    "node",
    "bash",
    "sh",
    "zsh",
    "fish",
}

DANGEROUS_GIT_SUBCOMMANDS = {
    "push",
    "pull",
    "commit",
    "merge",
    "rebase",
    "reset",
    "checkout",
    "switch",
    "clean",
    "stash",
    "tag",
    "branch",
    "remote",
}

SHELL_META = {"|", ">", "<", "&&", "||", ";", "`", "$(", "&"}
SECRET_HINT = re.compile(r"(?i)(api[_-]?key|token|secret|password|passwd|bearer|private[_-]?key|aws_access_key)")


def _json_response(success: bool, **payload: Any) -> str:
    return json.dumps({"success": success, **payload}, ensure_ascii=False)


def _obelisk_bin() -> str | None:
    return shutil.which("obelisk")


def _safe_cwd(cwd: str | None) -> str | None:
    if not cwd:
        return None
    path = Path(cwd).expanduser().resolve()
    if not path.exists() or not path.is_dir():
        raise ValueError(f"cwd does not exist or is not a directory: {cwd}")
    return str(path)


def _contains_shell_meta(command: str) -> bool:
    return any(token in command for token in SHELL_META)


def _split_command(command: str) -> list[str]:
    try:
        parts = shlex.split(command)
    except ValueError as exc:
        raise ValueError(f"cannot parse command: {exc}") from exc
    if not parts:
        raise ValueError("empty command")
    return parts


def _validate_read_only_command(command: str) -> list[str]:
    if SECRET_HINT.search(command):
        raise ValueError("refusing command that appears to contain or request secrets")
    if _contains_shell_meta(command):
        raise ValueError("refusing shell metacharacters, pipes, redirects, backgrounding, or command chaining")

    parts = _split_command(command)
    program = parts[0]

    if program == "obelisk":
        raise ValueError("command is already an Obelisk command")
    if program in DANGEROUS_PROGRAMS:
        raise ValueError(f"refusing potentially unsafe program: {program}")
    if program == "git" and len(parts) > 1 and parts[1] in DANGEROUS_GIT_SUBCOMMANDS:
        raise ValueError(f"refusing mutating git subcommand: git {parts[1]}")
    return parts


def _run_obelisk(args: Iterable[str], *, cwd: str | None = None, timeout_seconds: int = 120) -> str:
    obelisk = _obelisk_bin()
    if not obelisk:
        return _json_response(
            False,
            error="obelisk binary not found on PATH",
            hint="Build Obelisk and install it somewhere on PATH, for example ~/.local/bin/obelisk.",
        )

    try:
        run_cwd = _safe_cwd(cwd)
        timeout = max(1, min(int(timeout_seconds), 600))
        proc = subprocess.run(
            [obelisk, *list(args)],
            cwd=run_cwd,
            text=True,
            capture_output=True,
            timeout=timeout,
            check=False,
        )
    except Exception as exc:  # noqa: BLE001 - plugin boundary should serialize failures
        return _json_response(False, error=str(exc))

    return _json_response(
        proc.returncode == 0,
        returncode=proc.returncode,
        stdout=proc.stdout,
        stderr=proc.stderr,
    )


def _pack_args(params: dict[str, Any]) -> list[str]:
    args = ["pack", "--budget", str(int(params.get("budget", 12000)))]

    for value in params.get("system") or []:
        args.extend(["--system", str(value)])
    for value in params.get("history") or []:
        args.extend(["--history", str(value)])
    for value in params.get("files") or []:
        args.extend(["--file", str(value)])
    for value in params.get("dirs") or []:
        args.extend(["--dir", str(value)])

    if params.get("diff", True):
        args.append("--diff")
    if params.get("tools"):
        args.extend(["--tools", str(params["tools"])])
    if params.get("out"):
        args.extend(["--out", str(params["out"])])

    return args


def handle_tool(name: str, params: dict[str, Any] | None, **_: Any) -> str:
    params = params or {}
    cwd = params.get("cwd")

    try:
        if name == "obelisk_run":
            command = str(params.get("command", ""))
            parts = _validate_read_only_command(command)
            return _run_obelisk(["run", *parts], cwd=cwd, timeout_seconds=params.get("timeout_seconds", 120))

        if name == "obelisk_pack":
            return _run_obelisk(_pack_args(params), cwd=cwd, timeout_seconds=120)

        if name == "obelisk_outline":
            return _run_obelisk(["outline", str(params["file"])], cwd=cwd, timeout_seconds=30)

        if name == "obelisk_symbol":
            return _run_obelisk(["symbol", str(params["file"]), str(params["name"])], cwd=cwd, timeout_seconds=30)

        if name == "obelisk_restore":
            return _run_obelisk(["restore", str(params["handle"])], cwd=cwd, timeout_seconds=30)

        if name == "obelisk_rewrite":
            command = str(params.get("command", ""))
            _validate_read_only_command(command)
            parts = _split_command(command)
            return _run_obelisk(["rewrite", *parts], cwd=cwd, timeout_seconds=10)

        if name == "obelisk_stats":
            return _run_obelisk(["stats"], cwd=cwd, timeout_seconds=30)

        if name == "obelisk_doctor":
            return _run_obelisk(["doctor"], cwd=cwd, timeout_seconds=30)

        return _json_response(False, error=f"unknown Obelisk tool: {name}")
    except Exception as exc:  # noqa: BLE001 - serialize plugin errors for the host
        return _json_response(False, error=str(exc))


def pre_tool_call(*args: Any, **kwargs: Any) -> Any:
    """Best-effort Hermes pre_tool_call hook.

    Hermes hook signatures may evolve. This handler is deliberately flexible.
    It only suggests a rewrite for obvious shell tool calls and otherwise stays
    silent. If Hermes ignores the return shape, nothing dangerous happens.
    """

    tool_name = kwargs.get("tool_name") or (args[0] if args else "")
    params = kwargs.get("params") or (args[1] if len(args) > 1 and isinstance(args[1], dict) else {})

    if str(tool_name).lower() not in {"bash", "shell", "exec", "terminal", "run_command"}:
        return None

    command = params.get("command") if isinstance(params, dict) else None
    if not isinstance(command, str):
        return None

    try:
        _validate_read_only_command(command)
    except Exception:
        return None

    rewritten = _run_obelisk(["rewrite", *_split_command(command)], timeout_seconds=10)
    try:
        payload = json.loads(rewritten)
        if payload.get("success") and payload.get("stdout", "").strip():
            new_params = dict(params)
            new_params["command"] = payload["stdout"].strip()
            return {"action": "rewrite", "params": new_params}
    except Exception:
        return None

    return None


def slash_obelisk(args: Any = None, **_: Any) -> str:
    """Slash command handler for `/obelisk`."""

    del args
    return _json_response(
        True,
        message="Obelisk Hermes plugin is installed.",
        commands=[
            "Use obelisk_pack for compact context.",
            "Use obelisk_run for safe noisy commands.",
            "Use obelisk_outline and obelisk_symbol before reading large files.",
            "Use obelisk_restore only when compact output is insufficient.",
        ],
    )


def slash_obelisk_stats(args: Any = None, **kwargs: Any) -> str:
    del args
    return handle_tool("obelisk_stats", {"cwd": kwargs.get("cwd")})


def slash_obelisk_doctor(args: Any = None, **kwargs: Any) -> str:
    del args
    return handle_tool("obelisk_doctor", {"cwd": kwargs.get("cwd")})
