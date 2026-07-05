# Validation notes

OpenClaw validation should be done with:

```bash
cd plugins/openclaw-obelisk
npm install
npm run plugin:validate
```

The generated `openclaw.plugin.json` must keep `contracts.tools` aligned with the tools registered in `src/index.ts`.
