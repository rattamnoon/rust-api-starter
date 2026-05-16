---
name: knowledge-base-engineering
description: Maintain a code-based engineering knowledge base inside the repository. Use when documenting bugs, incidents, issue impact, root cause, fix summaries, affected modules or services, data flow, architecture decisions, or prevention steps. Invoke for postmortems, recurring bug records, architecture updates, ADRs, and onboarding-oriented engineering documentation.
---

# Knowledge Base Engineering

Use this skill when the task is to create or update repository documentation that preserves engineering context for future developers.

This repository already uses `docs/` as its knowledge base. Do not create a second documentation system unless the user explicitly asks for one.

## Core Workflow

1. **Classify the knowledge**
   - `bugs/` for non-trivial or recurring defects
   - `incidents/` for production or high-severity issues
   - `changes/` for behavior-changing fixes or features
   - `adr/` for technical decisions with tradeoffs
   - `architecture/` for stable system understanding, boundaries, and data flow
   - `runbooks/` for debugging or operational procedures

2. **Update the canonical doc**
   - Start from `docs/README.md` and the relevant file in `docs/templates/`
   - Prefer updating an existing record if the subject already exists
   - Create a new record only when the change adds a distinct historical event or decision

3. **Capture the required engineering context**
   - Summary of the issue or decision
   - Impact on users, systems, or operations
   - Root cause and contributing factors
   - Affected modules, services, endpoints, jobs, tables, or flows
   - Fix summary or decision outcome
   - Prevention steps such as tests, alerts, validations, or process changes

4. **Cross-link the repository knowledge**
   - Link related ADRs, bugs, incidents, changes, runbooks, code paths, migrations, and PRs when available
   - Keep indexes current when adding a new historical record

5. **Keep the docs concise**
   - Write short, factual Markdown
   - Optimize for fast onboarding and future debugging, not storytelling

## Repo-Specific References

Read only the files needed for the task:

- `docs/README.md` for repository rules and section ownership
- `docs/templates/bug.md` for bug records
- `docs/templates/incident.md` for incident or postmortem records
- `docs/templates/change.md` for feature or fix summaries
- `docs/templates/adr.md` for architecture decisions
- `docs/templates/module-doc.md` when documenting a module or service
- `docs/architecture/data-flow.md` and `docs/architecture/module-map.md` when the change affects system flow or ownership boundaries

## Update Rules

### For bug fixes

- Add or update a `docs/bugs/*.md` record when the defect is recurring, non-obvious, or high-impact
- Update `docs/architecture/` if the fix changes request flow, service boundaries, or data ownership
- Add a `docs/changes/*.md` note if the fix changes external behavior or operational assumptions

### For incidents

- Create or update `docs/incidents/*.md`
- Capture timeline, impact, root cause, resolution, and follow-up actions
- Link the incident to related bug records and runbooks

### For architecture changes

- Add an ADR if engineers would later ask "why was this done?"
- Update `docs/architecture/system-overview.md`, `module-map.md`, or `data-flow.md` if the model of the system changed

## Output Expectations

When you update the knowledge base, prefer delivering:

1. The new or updated Markdown files
2. A short note describing which `docs/` areas changed
3. Any remaining documentation gaps or follow-up records that should be added later
