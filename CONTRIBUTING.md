# Contributing to IntentKernel

Thanks for helping build IntentKernel.

## Project tracking

Use a GitHub Projects board with these columns:

- Backlog
- Ready
- In Progress
- Review
- Done

Use the board together with these release milestones:

- `v0.2` — Cross-platform shell foundation
- `v0.3` — AI scheduler prototype
- `v1.0` — Full AI OS image

See [`roadmap/project_board.md`](roadmap/project_board.md) for the board layout and milestone scope.

## Commit format

All commits and pull request titles should follow Conventional Commits:

- `feat:` new functionality
- `fix:` bug fixes
- `docs:` documentation updates
- `test:` test-only changes
- `refactor:` internal restructuring without behavior changes
- `chore:` maintenance and tooling updates

## Required validation

Every pull request must:

1. Pass the host capability test workflow on Linux, macOS, and Windows.
2. Run `make test` locally when developing on Linux or macOS.
3. Include equivalent host-test evidence when developing on Windows.
4. Include tests for new logic.
5. Run `cargo fmt` and `cargo clippy` for Rust code when Rust components are added.

If a change touches kernel-facing code, include a short VM or container test plan in the pull request description.

## Design guardrails

Contributions must stay aligned with the architecture and governance documents:

- [`README.md`](README.md)
- [`roadmap/implementation_plan.md`](roadmap/implementation_plan.md)
- [`governance/principles.md`](governance/principles.md)

## Good first contributions

Good starter tasks include:

- build and CI improvements
- host capability test coverage
- documentation fixes
- milestone-scoped issues tagged `good first issue`
