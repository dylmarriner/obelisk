# Local validation

Use this file as a quick local checklist while the package is still pre-release.

```bash
cd plugins/openclaw-obelisk
npm install
npm run check
npm run build
npm run plugin:validate
openclaw plugins install .
openclaw plugins inspect obelisk --runtime
```

Remove or fold this into README once the package is validated in a real OpenClaw checkout.
