# Release v0.27.2

## Fixes

### Agent session type hint reuse

This patch release includes a focused runtime fix in `crates/aster/src/agents/agent.rs`:

- remember the current `SessionType` after loading or initializing the runtime session
- reuse that hint when preparing tools and prompts
- avoid redundant session-store reloads during reply and runtime preparation paths
- add regression tests covering runtime initialization and reply flows

## Release notes

- workspace version metadata is updated to `0.27.2`
- local crate dependency versions are aligned to `0.27.2`
- desktop package metadata and generated OpenAPI version are aligned to `0.27.2`
- lockfiles are refreshed together with the release validation flow

---

**Full Changelog**: v0.27.1...v0.27.2
