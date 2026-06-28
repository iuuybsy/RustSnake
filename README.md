# RustSnake

A classic Snake game built with Rust and [ggez](https://github.com/ggez/ggez).

## Gameplay

Control a snake on a 41×41 grid, eat apples to grow, and avoid biting yourself.

| Action     | Key            |
|------------|----------------|
| Move up    | <kbd>W</kbd>   |
| Move down  | <kbd>S</kbd>   |
| Move left  | <kbd>A</kbd>   |
| Move right | <kbd>D</kbd>   |

- The board wraps around at the edges — exiting one side enters from the opposite side.
- You cannot reverse direction in a single tick (e.g. pressing <kbd>A</kbd> while moving right is ignored).
- The game resets automatically when the snake bites itself or fills every cell.

## Requirements

- [Rust](https://www.rust-lang.org/) **1.85+** (edition 2024)

## Build & Run

```sh
git clone https://github.com/iuuybsy/RustSnake.git
cd RustSnake
cargo run --release
```

The window opens at 1230×1230 pixels (41 cells × 30 px each).

## Project Structure

```
Snake/
├── Cargo.toml          # Crate manifest
├── LICENSE             # MIT
├── README.md
├── resources_dir/      # Reserved for assets (currently unused)
└── src/
    └── main.rs         # All game logic and rendering
```

## Dependencies

| Crate | Version | Purpose              |
|-------|---------|----------------------|
| ggez  | 0.10    | Game engine & window |
| rand  | 0.10    | Apple placement RNG  |

## License

MIT — see [LICENSE](./LICENSE) for details.
