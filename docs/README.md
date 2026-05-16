# Engineering Knowledge Base

This repository uses `docs/` as the source of truth for engineering knowledge that should live next to the code.

## Start Here
- [Architecture](./architecture/system-overview.md)
- [Data Flow](./architecture/data-flow.md)
- [Module Map](./architecture/module-map.md)
- [ADRs](./adr/README.md)
- [Bugs](./bugs/README.md)
- [Incidents](./incidents/README.md)
- [Changes](./changes/README.md)
- [Runbooks](./runbooks/README.md)

## What Goes Where
- `architecture/`: stable system understanding, boundaries, dependencies, and request/data flow
- `adr/`: important technical decisions and their consequences
- `bugs/`: recurring or important defects, root cause, and prevention
- `incidents/`: production-impacting or high-severity issues
- `changes/`: behavior-changing fixes or features worth preserving for future engineers
- `runbooks/`: operational and debugging procedures
- `templates/`: copyable Markdown templates for new records

## Update Rules
- Update `architecture/` when flows, boundaries, ownership, or dependencies change.
- Add a `bugs/` record for non-trivial, recurring, or high-impact defects.
- Add an `incidents/` record when an issue affects production or needs coordinated response.
- Add an ADR when the team makes a technical decision that future engineers will need context for.
- Add a `changes/` record when a feature or fix changes behavior, contracts, or operational assumptions.

## Pull Request Workflow
Every PR should answer one question: "What knowledge would the next engineer need after this change?"

Use this checklist in the PR description:
- `Docs impact: none` with a short reason, or
- links to updated files in `docs/`

## Authoring Notes
- Prefer short factual writing over long narratives.
- Link related code paths, PRs, migrations, and other KB records.
- Keep historical records append-only; update summary/index files when needed.
