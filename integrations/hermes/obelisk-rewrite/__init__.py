"""Hermes plugin adapter for Obelisk command rewriting.

All rewrite logic lives in Obelisk's Rust ``obelisk rewrite`` command; this
module only bridges Hermes ``pre_tool_call`` payloads to that command and
fails open (any error means the original command runs untouched).

Mirrors the rtk-rewrite plugin's contract: exit 0 + a changed line on stdout
means "use this instead"; anything else means "leave it alone."
"""

import shutil
import subprocess
import sys

_obelisk_available = None
_obelisk_missing_warned = False


def register(ctx):
    """Register the Hermes pre-tool callback."""
    if not _check_obelisk():
        return

    ctx.register_hook("pre_tool_call", _pre_tool_call)


def _check_obelisk():
    """Return whether the obelisk binary is in PATH, warning once when missing."""
    global _obelisk_available, _obelisk_missing_warned

    if _obelisk_available is None:
        _obelisk_available = shutil.which("obelisk") is not None

    if not _obelisk_available and not _obelisk_missing_warned:
        _warn("obelisk binary not found in PATH; Hermes hook not registered")
        _obelisk_missing_warned = True

    return _obelisk_available


def _pre_tool_call(tool_name=None, args=None, **_kwargs):
    """Rewrite mutable Hermes terminal command args when Obelisk provides a change."""
    try:
        if tool_name != "terminal" or not isinstance(args, dict):
            return

        command = args.get("command")
        if not isinstance(command, str) or not command.strip():
            return

        try:
            result = subprocess.run(
                ["obelisk", "rewrite", command],
                shell=False,
                timeout=2,
                capture_output=True,
                text=True,
            )
        except subprocess.TimeoutExpired:
            _warn("obelisk rewrite timed out")
            return

        if result.returncode != 0:
            return

        rewritten = result.stdout.strip()
        if rewritten and rewritten != command:
            args["command"] = rewritten
    except Exception as e:
        _warn(str(e))
        return


def _warn(message):
    print(f"obelisk: hermes plugin warning: {message}", file=sys.stderr)
