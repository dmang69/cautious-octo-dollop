# tauri-app

Cross-platform AI OS desktop shell built with Tauri + React.

## Features

- **Terminal** — AI-assisted command line with natural language suggestions
- **Process Table** — real-time process list with AI-recommended priorities
- **Metrics Dashboard** — live CPU / memory charts from the AI Runtime daemon

## Development

```bash
npm install
npm run tauri:dev
```

## Production Build

```bash
npm run tauri:build
```

Outputs platform-native installers to `src-tauri/target/release/bundle/`.
