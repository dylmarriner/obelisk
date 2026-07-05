# Obelisk Restore Context

Use `obelisk_restore` when a compact Obelisk result contains a restore handle and the compressed output is not enough.

## Rule

Restore only when needed.

```json
{"handle": "7f3a1b2c4d5e"}
```

Restore handles are the emergency hatch, not the front door. Pulling every original back into context defeats the entire point, which would be painfully on-brand for bad automation.
