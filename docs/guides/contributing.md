# Contributing Guide

## Development Workflow

1. Fork the repository and create a feature branch from `develop`:
   ```bash
   git checkout -b feature/your-feature develop
   ```

2. Make your changes following the conventions below.

3. Run the full test suite:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   ```

4. Push and open a Pull Request targeting `develop`.

## Code Conventions

- **Rust**: follow `rustfmt` defaults (`cargo fmt --all`).
- **TypeScript/React**: ESLint + Prettier (run `npm run lint` in `shell/tauri-app`).
- **Python**: Black + isort (`black . && isort .` in `models/`).

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(ai-runtime): add streaming telemetry endpoint
fix(kernel-interface): handle EPERM in set_priority
docs(guides): add model training walkthrough
```

## Pull Request Checklist

- [ ] Tests pass on Linux, Windows, macOS
- [ ] `cargo clippy` reports no warnings
- [ ] Documentation updated for public API changes
- [ ] `model_registry.json` updated if ONNX models changed

## Security Issues

Please report security vulnerabilities privately via GitHub Security Advisories — do **not** open a public issue.
