# Copilot Repository Instructions

Use `docs/` as the source of truth for repository knowledge.

When a task involves any of the following, use the `knowledge-base-engineering` skill and update the relevant files in `docs/`:
- bugs or incidents
- issue impact
- root cause
- solution or fix summary
- affected modules, services, endpoints, jobs, migrations, or data flows
- architecture decisions and ADRs
- prevention steps, guardrails, or runbooks

Follow these rules:
- Do not create a second knowledge base outside `docs/` unless explicitly asked.
- Start from [docs/README.md](../docs/README.md) and the templates in `docs/templates/`.
- Keep records concise, factual, and cross-linked to code paths, migrations, ADRs, bugs, incidents, or runbooks when relevant.
- Update `docs/architecture/` when flows, module ownership, boundaries, or dependencies change.
- Add or update `docs/bugs/` for non-trivial, recurring, or high-impact defects.
- Add or update `docs/incidents/` for production-impacting or coordinated response issues.
- Add or update `docs/changes/` for behavior-changing fixes or features.
- Add an ADR in `docs/adr/` when the change answers "why are we doing it this way?"
