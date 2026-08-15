# CYVORIQ Erase

CYVORIQ Erase engineering repository.

## Frozen development baseline

- Node 24 LTS
- npm 11.x
- React 19.2.8
- React DOM 19.2.8
- Vite 8.2.0
- React Router 8.3.0
- TypeScript 5.9.3
- @vitejs/plugin-react 6.0.4

## Frontend

```bash
cd frontend
npm install
npm run build
npm run dev -- --host 0.0.0.0
```

## Current scope

This package establishes the non-destructive frontend foundation only. It does not include production API, database, Cloudflare Worker, Hyperdrive, sanitization execution, or device-agent destructive capability.
