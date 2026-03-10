# Copilot Instructions

## Design Document & Implementation Plan

This project follows a canonical **Game Design Document** at [`docs/design.md`](../docs/design.md) and a tracked **MVP Implementation Plan** at [`docs/mvp-implementation-plan.md`](../docs/mvp-implementation-plan.md).

**AI agents must always:**

1. **Read `docs/design.md` first** before implementing any feature — every new system must trace back to a documented section in the design doc.
2. **Update `docs/mvp-implementation-plan.md`** when a milestone task is completed: change the `- [ ]` checkbox to `- [x]` for each finished task. Update the plan within the same PR that implements the work.
3. **Never implement features that are marked as out-of-scope** in `docs/design.md §9` unless a dedicated issue has been opened and the design doc has been updated to bring them into scope.
4. **Keep the implementation plan and design doc in sync.** If a task's implementation differs from the plan, update the relevant section of the plan to reflect the actual approach taken.
5. **Update `docs/design.md` when an issue changes game design decisions** — any issue that alters gameplay values, controls, rules, or system behaviour must be reflected in the design doc in the same PR that implements the change.

---

## Bevy Documentation

This project uses [Bevy Engine](https://bevyengine.org/) for game development. Local Bevy documentation is generated and committed to the `docs/bevy/` directory.

**Always use the local Bevy documentation** in `docs/bevy/` when:

- Looking up Bevy APIs, types, traits, or functions
- Understanding Bevy's ECS (Entity Component System) architecture
- Checking component, system, or plugin usage

The documentation is stored as a single JSON file at `docs/bevy/bevy.json`.

This documentation is generated specifically for the version of Bevy used in this project (`bevy = "0.18"`), ensuring accurate API references that match the code.

### Regenerating Documentation

If the local documentation is outdated or missing, regenerate it by:

1. Triggering the **"Generate Bevy Documentation"** GitHub Actions workflow manually (via `workflow_dispatch`), or
2. Running locally: `cargo rustdoc -p bevy -- -Z unstable-options --output-format json && mkdir -p docs/bevy && cp target/doc/bevy.json docs/bevy/bevy.json`
