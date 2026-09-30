# 🦫 Bevyr

A tiny retro TUI arcade game built with **Bevy 0.19** and **ratatui**.

A river meanders down the screen. You control a beaver on a dam made of three big chunks. Pick a chunk up, move left or right, and drop it so the dam covers the river when each wave reaches the bottom.

- Cover a wave → +10 points.
- Let water slip through → lose a life.
- 3 lives, endless waves, session high score.

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

## Controls

| Key | Action |
| --- | --- |
| `←` / `→` or `A` / `D` | Move beaver |
| `Space` / `Enter` | Pick up / drop a chunk |
| `Esc` | Back to menu |
| `Q` (in menu) | Quit |

The project follows the Bevy quick-start structure: `App`, systems, resources, states, and a small plugin-style module split.
