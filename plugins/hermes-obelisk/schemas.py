"""Tool schemas exposed by the Obelisk Hermes plugin."""

TOOL_SCHEMAS = [
    {
        "name": "obelisk_run",
        "description": "Run a safe, read-heavy shell command through `obelisk run` and return compact, reversible output. Refuses destructive, mutating, interactive, piped, redirected, or secret-looking commands.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command to run, for example `cargo build` or `git status`.",
                },
                "timeout_seconds": {
                    "type": "integer",
                    "description": "Maximum seconds to wait before killing Obelisk.",
                    "default": 120,
                    "minimum": 1,
                    "maximum": 600,
                },
                "cwd": {
                    "type": "string",
                    "description": "Optional working directory. Defaults to Hermes current working directory.",
                },
            },
            "required": ["command"],
        },
    },
    {
        "name": "obelisk_pack",
        "description": "Build a provider-neutral, token-budgeted context pack from files, directories, diffs, history, system files, and optional tool schema JSON.",
        "parameters": {
            "type": "object",
            "properties": {
                "budget": {"type": "integer", "default": 12000, "minimum": 1000},
                "system": {"type": "array", "items": {"type": "string"}, "default": []},
                "history": {"type": "array", "items": {"type": "string"}, "default": []},
                "files": {"type": "array", "items": {"type": "string"}, "default": []},
                "dirs": {"type": "array", "items": {"type": "string"}, "default": []},
                "diff": {"type": "boolean", "default": True},
                "tools": {"type": "string", "description": "Optional tool schema JSON file path."},
                "out": {"type": "string", "description": "Optional output file path."},
                "cwd": {"type": "string", "description": "Optional working directory."},
            },
        },
    },
    {
        "name": "obelisk_outline",
        "description": "List a source file's symbols and line ranges without reading the full file into context.",
        "parameters": {
            "type": "object",
            "properties": {
                "file": {"type": "string"},
                "cwd": {"type": "string", "description": "Optional working directory."},
            },
            "required": ["file"],
        },
    },
    {
        "name": "obelisk_symbol",
        "description": "Extract one named symbol from a source file instead of reading the whole file.",
        "parameters": {
            "type": "object",
            "properties": {
                "file": {"type": "string"},
                "name": {"type": "string"},
                "cwd": {"type": "string", "description": "Optional working directory."},
            },
            "required": ["file", "name"],
        },
    },
    {
        "name": "obelisk_restore",
        "description": "Restore a compressed Obelisk blob/checkpoint by handle when compact output is insufficient.",
        "parameters": {
            "type": "object",
            "properties": {
                "handle": {"type": "string"},
                "cwd": {"type": "string", "description": "Optional working directory."},
            },
            "required": ["handle"],
        },
    },
    {
        "name": "obelisk_rewrite",
        "description": "Ask Obelisk whether a shell command should be wrapped. Returns the rewritten command or an explanation that it should be left alone.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {"type": "string"},
                "cwd": {"type": "string", "description": "Optional working directory."},
            },
            "required": ["command"],
        },
    },
    {
        "name": "obelisk_stats",
        "description": "Show token savings across Obelisk layers.",
        "parameters": {
            "type": "object",
            "properties": {"cwd": {"type": "string", "description": "Optional working directory."}},
        },
    },
    {
        "name": "obelisk_doctor",
        "description": "Verify Obelisk installation and agent wiring.",
        "parameters": {
            "type": "object",
            "properties": {"cwd": {"type": "string", "description": "Optional working directory."}},
        },
    },
]
