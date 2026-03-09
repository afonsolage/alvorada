# Copilot Instructions

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
