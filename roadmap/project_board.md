# GitHub Projects Board Layout

Use one GitHub Projects board to track execution work across the current roadmap.

## Columns

1. **Backlog** — defined work that is not ready to start
2. **Ready** — scoped issues with acceptance criteria and milestone assignment
3. **In Progress** — work currently assigned and actively being built
4. **Review** — pull requests awaiting maintainer review or CI confirmation
5. **Done** — merged work that satisfies its milestone acceptance criteria

## Release milestones

### v0.2 — Cross-platform shell

Scope:

- shell scaffolding
- cross-platform host build workflow
- contributor workflow hardening

Suggested labels:

- `shell`
- `ci`
- `good first issue`

### v0.3 — AI scheduler

Scope:

- scheduler interfaces
- telemetry collection
- policy experimentation

Suggested labels:

- `scheduler`
- `telemetry`
- `research`

### v1.0 — Full AI OS image

Scope:

- integrated shell, kernel, and broker flows
- packaging and release artifacts
- end-to-end validation and documentation

Suggested labels:

- `release`
- `integration`
- `docs`

## Intake rules

Each issue added to the board should define:

- one target milestone (`v0.2`, `v0.3`, or `v1.0`)
- acceptance criteria
- required test evidence
- whether it is suitable for new contributors
