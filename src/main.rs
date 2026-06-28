//! A classic Snake game implemented in Rust using the [`ggez`] game engine.
//!
//! The snake moves on a toroidal grid — wrapping around edges instead of
//! hitting a wall. The player controls direction with WASD keys. Eating an
//! apple grows the snake by one segment. The game resets when the snake
//! bites itself or fills every cell on the board.

use ggez::{
    Context, ContextBuilder, GameError,
    conf::Conf,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawMode, Mesh, MeshBuilder, Rect},
    winit::keyboard::Key,
};
use rand::prelude::*;
use std::collections::{HashSet, VecDeque};

// ---------------------------------------------------------------------------
// Configuration constants
// ---------------------------------------------------------------------------

/// Game tick rate in frames per second.
const TARGET_FPS: u32 = 10;

/// Side length of a single grid cell in logical pixels.
const SQUARE_LENGTH: f32 = 30.0;

/// Number of columns on the game board.
const GRID_COLS: u32 = 41;

/// Number of rows on the game board.
const GRID_ROWS: u32 = 41;

// Initial apple / snake positions

/// Initial x-coordinate of the apple.
const INIT_APPLE_X: u32 = (GRID_COLS + 1) / 2 + 3;

/// Initial y-coordinate of the apple (same row as the snake).
const INIT_APPLE_Y: u32 = (GRID_ROWS + 1) / 2;

/// Leftmost x-coordinate of the initial snake body.
const INIT_BODY_LEFT: u32 = 6;

/// Rightmost x-coordinate of the initial snake body (inclusive).
const INIT_BODY_RIGHT: u32 = 8;

// ---------------------------------------------------------------------------
// Movement direction
// ---------------------------------------------------------------------------

/// Cardinal direction in which the snake is currently moving.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// Coordinate type
// ---------------------------------------------------------------------------

/// A discrete (column, row) position on the game grid.
///
/// `(0, 0)` is the top-left corner.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SnakeGameCoord {
    x: u32,
    y: u32,
}

// ---------------------------------------------------------------------------
// Game grid — all pure-logic state
// ---------------------------------------------------------------------------

/// Holds the mutable board state: snake body, apple position, cell occupancy,
/// and the RNG for apple placement.
struct SnakeGameGrid {
    /// Current apple position.
    apple_coord: SnakeGameCoord,

    /// Snake body segments, ordered from head (front) to tail (back).
    body: VecDeque<SnakeGameCoord>,

    /// Set of body coordinates excluding the head, used for self-bite
    /// collision detection.
    tail_coords: HashSet<SnakeGameCoord>,

    /// Set of cells that do **not** contain a snake segment. The apple may
    /// only spawn on one of these cells.
    empty_cells: HashSet<SnakeGameCoord>,

    /// Thread-local random number generator for apple placement.
    rng: ThreadRng,
}

impl SnakeGameGrid {
    /// Creates a new game grid with the initial snake body and apple
    /// positions defined by the configuration constants.
    pub fn new() -> Self {
        let apple_coord = SnakeGameCoord {
            x: INIT_APPLE_X,
            y: INIT_APPLE_Y,
        };

        // Build the initial horizontal snake body from left to right,
        // pushing front so the rightmost segment ends up as the head.
        let mut body: VecDeque<SnakeGameCoord> = VecDeque::new();
        let mut tail_coords: HashSet<SnakeGameCoord> = HashSet::new();
        for i in INIT_BODY_LEFT..=INIT_BODY_RIGHT {
            let body_coord = SnakeGameCoord {
                x: i,
                y: INIT_APPLE_Y,
            };
            body.push_front(body_coord);
            if i < INIT_BODY_RIGHT {
                tail_coords.insert(body_coord);
            }
        }

        // Populate the set of empty cells — every cell that is not part of
        // the initial snake body.
        let mut empty_cells: HashSet<SnakeGameCoord> = HashSet::new();
        for i in 0..GRID_COLS {
            for j in 0..GRID_ROWS {
                if j == INIT_APPLE_Y && i >= INIT_BODY_LEFT && i <= INIT_BODY_RIGHT {
                    continue;
                }
                empty_cells.insert(SnakeGameCoord { x: i, y: j });
            }
        }

        let rng = rand::rng();

        SnakeGameGrid {
            apple_coord,
            body,
            tail_coords,
            empty_cells,
            rng,
        }
    }

    /// Moves the apple to a random cell chosen from [`empty_cells`].
    pub fn spawn_apple(&mut self) {
        if let Some(new_apple_coord) = self.empty_cells.iter().choose(&mut self.rng) {
            self.apple_coord = *new_apple_coord;
        }
    }

    /// Returns `true` when the snake head occupies the same cell as the
    /// apple.
    pub fn is_eating_apple(&self) -> bool {
        self.body.front() == Some(&self.apple_coord)
    }
}

// ---------------------------------------------------------------------------
// Game state — graphics + logic combined
// ---------------------------------------------------------------------------

/// Top-level application state bridging the game grid with rendered meshes.
struct SnakeGameState {
    /// Pre-built mesh for the checkerboard background.
    background_mesh: Mesh,

    /// Reusable mesh for the apple (a solid red square).
    apple_square_mesh: Mesh,

    /// Reusable mesh for a single snake segment (a solid black square).
    snake_square_mesh: Mesh,

    /// The logical game board.
    grid: SnakeGameGrid,

    /// Current movement direction of the snake.
    move_direction: MoveDirection,
}

impl SnakeGameState {
    /// Builds a checkerboard-pattern background mesh spanning the entire
    /// grid.
    fn build_background_mesh(ctx: &mut Context) -> Result<Mesh, GameError> {
        let silver_color = Color::from_rgb(192, 192, 192);
        let grey_color = Color::from_rgb(128, 128, 128);

        let mut builder = MeshBuilder::new();

        for i in 0..GRID_COLS {
            for j in 0..GRID_ROWS {
                let x_pos = i as f32 * SQUARE_LENGTH;
                let y_pos = j as f32 * SQUARE_LENGTH;

                // Alternate colours to produce a checkerboard.
                let current_color = {
                    if (i + j) % 2 == 1 {
                        silver_color
                    } else {
                        grey_color
                    }
                };
                builder.rectangle(
                    DrawMode::fill(),
                    Rect::new(x_pos, y_pos, SQUARE_LENGTH, SQUARE_LENGTH),
                    current_color,
                )?;
            }
        }
        let mesh = graphics::Mesh::from_data(ctx, builder.build());
        Ok(mesh)
    }

    /// Builds a single-colour square mesh sized to one grid cell.
    fn build_square_mesh(ctx: &mut Context, color: Color) -> Result<Mesh, GameError> {
        let mut builder = MeshBuilder::new();

        builder.rectangle(
            DrawMode::fill(),
            Rect::new(0.0, 0.0, SQUARE_LENGTH, SQUARE_LENGTH),
            color,
        )?;

        let mesh = graphics::Mesh::from_data(ctx, builder.build());
        Ok(mesh)
    }

    /// Initialises the game state: builds meshes, creates the grid, and
    /// sets the initial direction to [`MoveDirection::Right`].
    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        let grid = SnakeGameGrid::new();

        let background_mesh = SnakeGameState::build_background_mesh(ctx)?;
        let apple_square_mesh = SnakeGameState::build_square_mesh(ctx, Color::RED)?;
        let snake_square_mesh = SnakeGameState::build_square_mesh(ctx, Color::BLACK)?;

        Ok(SnakeGameState {
            background_mesh,
            apple_square_mesh,
            snake_square_mesh,
            grid,
            move_direction: MoveDirection::Right,
        })
    }

    /// Returns `true` when `current` and `other` point in exactly opposite
    /// directions. Used to prevent the snake from reversing into itself.
    fn is_opposite_direction(current: &MoveDirection, other: &MoveDirection) -> bool {
        match current {
            MoveDirection::Down => other == &MoveDirection::Up,
            MoveDirection::Left => other == &MoveDirection::Right,
            MoveDirection::Up => other == &MoveDirection::Down,
            MoveDirection::Right => other == &MoveDirection::Left,
        }
    }

    /// Returns `true` when the snake head collides with any other body
    /// segment.
    fn check_self_bite(&self) -> bool {
        if let Some(snake_head) = self.grid.body.front() {
            self.grid.tail_coords.contains(snake_head)
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// ggez EventHandler implementation
// ---------------------------------------------------------------------------

impl EventHandler for SnakeGameState {
    /// Renders the background, apple, and every snake segment for the
    /// current frame.
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        canvas.draw(&self.background_mesh, Vec2::ZERO);

        let apple_pos = Vec2::new(
            self.grid.apple_coord.x as f32 * SQUARE_LENGTH,
            self.grid.apple_coord.y as f32 * SQUARE_LENGTH,
        );
        canvas.draw(&self.apple_square_mesh, apple_pos);

        for segment in &self.grid.body {
            let segment_pos = Vec2::new(
                segment.x as f32 * SQUARE_LENGTH,
                segment.y as f32 * SQUARE_LENGTH,
            );
            canvas.draw(&self.snake_square_mesh, segment_pos);
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    /// Advances the game by one tick — moving the snake, handling apple
    /// consumption, and checking for game-over conditions.
    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if ctx.time.check_update_time(TARGET_FPS) {
            if let Some(snake_head) = self.grid.body.front() {
                // Compute the next head position with toroidal wrapping:
                // leaving one edge enters from the opposite edge.
                let mut head_x = snake_head.x;
                let mut head_y = snake_head.y;
                match self.move_direction {
                    MoveDirection::Down => {
                        head_y = (head_y + 1) % GRID_ROWS;
                    }
                    MoveDirection::Left => {
                        head_x = (head_x + GRID_COLS - 1) % GRID_COLS;
                    }
                    MoveDirection::Up => {
                        head_y = (head_y + GRID_ROWS - 1) % GRID_ROWS;
                    }
                    MoveDirection::Right => {
                        head_x = (head_x + 1) % GRID_COLS;
                    }
                }
                let new_head = SnakeGameCoord {
                    x: head_x,
                    y: head_y,
                };

                // Move the head forward. The previous head position joins
                // the tail set, and the new head is removed from empty
                // cells.
                self.grid.tail_coords.insert(*snake_head);
                self.grid.body.push_front(new_head);
                self.grid.empty_cells.remove(&new_head);

                if self.grid.is_eating_apple() {
                    // Grow: do not remove the tail; spawn a new apple.
                    self.grid.spawn_apple();
                } else {
                    // Shrink the tail to keep the body length constant.
                    if let Some(old_tail) = self.grid.body.pop_back() {
                        self.grid.empty_cells.insert(old_tail);
                        self.grid.tail_coords.remove(&old_tail);
                    }
                }

                // Game-over: snake bites itself, or every cell is occupied.
                if self.check_self_bite() || self.grid.empty_cells.len() == 0 {
                    self.grid = SnakeGameGrid::new();
                    self.move_direction = MoveDirection::Right;
                }
            }
        }
        Ok(())
    }

    /// Handles WASD key presses for direction changes.
    ///
    /// Repeated-key events (auto-repeat from holding a key) are ignored.
    /// Opposite-direction changes are also ignored to prevent the snake
    /// from reversing into itself instantly.
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool,
    ) -> Result<(), GameError> {
        if !repeated {
            let key = input.event.logical_key;
            if let Key::Character(key_str) = key {
                let new_direction = match key_str.as_str() {
                    "s" => Some(MoveDirection::Down),
                    "a" => Some(MoveDirection::Left),
                    "w" => Some(MoveDirection::Up),
                    "d" => Some(MoveDirection::Right),
                    _ => None,
                };
                if let Some(new_direction) = new_direction {
                    if !SnakeGameState::is_opposite_direction(&self.move_direction, &new_direction)
                    {
                        self.move_direction = new_direction;
                    }
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Sets up the ggez context, window configuration, and launches the game
/// loop.
fn main() -> Result<(), GameError> {
    let mut conf = Conf::new();
    conf.window_setup.title = "Rust Snake By GGEZ".to_string();
    conf.window_mode.width = GRID_COLS as f32 * SQUARE_LENGTH;
    conf.window_mode.height = GRID_ROWS as f32 * SQUARE_LENGTH;

    let (mut ctx, event_loop) = ContextBuilder::new("snake", "zdy")
        .default_conf(conf)
        .build()?;

    let state = SnakeGameState::new(&mut ctx)?;

    event::run(ctx, event_loop, state)
}
