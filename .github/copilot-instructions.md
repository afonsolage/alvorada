# Copilot Instructions

## Bevy Documentation

This project uses [Bevy Engine](https://bevyengine.org/) for game development. Local Bevy documentation is generated and committed to the `docs/bevy/` directory.

**Always use the local Bevy documentation** in `docs/bevy/` when:

- Looking up Bevy APIs, types, traits, or functions
- Understanding Bevy's ECS (Entity Component System) architecture
- Checking component, system, or plugin usage

The documentation entry point is `docs/bevy/bevy/index.html`.

This documentation is generated specifically for the version of Bevy used in this project (`bevy = "0.18"`), ensuring accurate API references that match the code.

### Regenerating Documentation

If the local documentation is outdated or missing, regenerate it by:

1. Triggering the **"Generate Bevy Documentation"** GitHub Actions workflow manually (via `workflow_dispatch`), or
2. Running locally: `cargo doc --no-deps -p bevy && cp -r target/doc docs/bevy`
