# Contributing

Thanks for helping out bevyr!

## Getting started

- Rust stable + `cargo`
- `cargo run` to play
- `cargo test` to run the unit tests (they live in the same files as the code)

## Before opening a PR

1. Branch off `main`.
2. Make your change.
3. Run the checks:

   ```bash
   cargo fmt --check
   cargo clippy --all-targets
   cargo test
   ```

4. Commit with a clear message and open a PR against `main`.

## Guidelines

- Keep PRs small and focused — one feature or fix each.
- Follow the existing structure: Bevy ECS (systems, resources, states) split
  across `src/main.rs`, `src/dam.rs`, `src/river.rs`, `src/score.rs`.
- Avoid new dependencies without a good reason — say why in the PR description.
- This is a game: if a change touches movement, waves, or collisions,
  describe how it *plays* in the PR description.
