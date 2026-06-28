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

const TARGET_FPS: u32 = 10;
const SQUARE_LENGTH: f32 = 30.0;
const GRID_COLS: u32 = 41;
const GRID_ROWS: u32 = 41;
const INIT_APPLE_X: u32 = (GRID_COLS + 1) / 2 + 3;
const INIT_APPLE_Y: u32 = (GRID_ROWS + 1) / 2;
const INIT_BODY_LEFT: u32 = 6;
const INIT_BODY_RIGHT: u32 = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SnakeGameCoord {
    x: u32,
    y: u32,
}

struct SnakeGameGrid {
    apple_coord: SnakeGameCoord,
    body: VecDeque<SnakeGameCoord>,
    empty_coord: HashSet<SnakeGameCoord>,
    rng: ThreadRng,
}

impl SnakeGameGrid {
    pub fn new() -> Self {
        let apple_coord = SnakeGameCoord {
            x: INIT_APPLE_X,
            y: INIT_APPLE_Y,
        };
        let mut body: VecDeque<SnakeGameCoord> = VecDeque::new();
        for i in INIT_BODY_LEFT..=INIT_BODY_RIGHT {
            let body_coord = SnakeGameCoord {
                x: i,
                y: INIT_APPLE_Y,
            };
            body.push_front(body_coord);
        }
        let mut empty_coord: HashSet<SnakeGameCoord> = HashSet::new();

        for i in 0..GRID_COLS {
            for j in 0..GRID_ROWS {
                if j == INIT_APPLE_Y && i >= INIT_BODY_LEFT && i <= INIT_BODY_RIGHT {
                    continue;
                }
                empty_coord.insert(SnakeGameCoord { x: i, y: j });
            }
        }

        let rng = rand::rng();

        SnakeGameGrid {
            apple_coord,
            body,
            empty_coord,
            rng,
        }
    }

    pub fn spawn_apple(&mut self) {
        if let Some(new_apple_coord) = self.empty_coord.iter().choose(&mut self.rng) {
            self.apple_coord = new_apple_coord.clone();
        }
    }

    pub fn is_eating_apple(&self) -> bool {
        self.body.front() == Some(&self.apple_coord)
    }
}

struct SnakeGameState {
    background_mesh: Mesh,
    apple_mesh: Mesh,
    snake_mesh: Mesh,
    map_info: SnakeGameGrid,
    move_direction: MoveDirection,
}

impl SnakeGameState {
    fn build_background_mesh(ctx: &mut Context) -> Result<Mesh, GameError> {
        let silver_color = Color::from_rgb(192, 192, 192);
        let grey_color = Color::from_rgb(128, 128, 128);

        let mut builder = MeshBuilder::new();

        for i in 0..GRID_COLS {
            for j in 0..GRID_ROWS {
                let x_pos = i as f32 * SQUARE_LENGTH;
                let y_pos = j as f32 * SQUARE_LENGTH;

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

    fn build_apple_mesh(ctx: &mut Context, map_info: &SnakeGameGrid) -> Result<Mesh, GameError> {
        let mut builder = MeshBuilder::new();

        let x_pos = map_info.apple_coord.x as f32 * SQUARE_LENGTH;
        let y_pos = map_info.apple_coord.y as f32 * SQUARE_LENGTH;

        builder.rectangle(
            DrawMode::fill(),
            Rect::new(x_pos, y_pos, SQUARE_LENGTH, SQUARE_LENGTH),
            Color::RED,
        )?;

        let mesh = graphics::Mesh::from_data(ctx, builder.build());
        Ok(mesh)
    }

    fn build_snake_mesh(ctx: &mut Context, map_info: &SnakeGameGrid) -> Result<Mesh, GameError> {
        let mut builder = MeshBuilder::new();
        for element in &map_info.body {
            let x_pos = element.x as f32 * SQUARE_LENGTH;
            let y_pos = element.y as f32 * SQUARE_LENGTH;
            builder.rectangle(
                DrawMode::fill(),
                Rect::new(x_pos, y_pos, SQUARE_LENGTH, SQUARE_LENGTH),
                Color::BLACK,
            )?;
        }

        let mesh = graphics::Mesh::from_data(ctx, builder.build());
        Ok(mesh)
    }

    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        let map_info = SnakeGameGrid::new();

        let background_mesh = SnakeGameState::build_background_mesh(ctx)?;
        let apple_mesh = SnakeGameState::build_apple_mesh(ctx, &map_info)?;
        let snake_mesh = SnakeGameState::build_snake_mesh(ctx, &map_info)?;

        Ok(SnakeGameState {
            background_mesh,
            apple_mesh,
            snake_mesh,
            map_info,
            move_direction: MoveDirection::Right,
        })
    }

    fn is_opposite_direction(direction1: &MoveDirection, direction2: &MoveDirection) -> bool {
        match direction1 {
            MoveDirection::Down => direction2 == &MoveDirection::Up,
            MoveDirection::Left => direction2 == &MoveDirection::Right,
            MoveDirection::Up => direction2 == &MoveDirection::Down,
            MoveDirection::Right => direction2 == &MoveDirection::Left,
        }
    }

    fn check_self_bite(&self) -> bool {
        if let Some(snake_head) = self.map_info.body.front() {
            self.map_info
                .body
                .iter()
                .skip(1)
                .any(|seg| seg == snake_head)
        } else {
            false
        }
    }
}

impl EventHandler for SnakeGameState {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut my_canvas = Canvas::from_frame(ctx, Color::BLACK);
        my_canvas.draw(&self.background_mesh, Vec2::ZERO);
        my_canvas.draw(&self.apple_mesh, Vec2::ZERO);
        my_canvas.draw(&self.snake_mesh, Vec2::ZERO);
        my_canvas.finish(ctx)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if ctx.time.check_update_time(TARGET_FPS) {
            let snake_head = self.map_info.body.front().unwrap();
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
            self.map_info.body.push_front(new_head);
            self.map_info.empty_coord.remove(&new_head);
            if self.map_info.is_eating_apple() {
                self.map_info.spawn_apple();
                self.apple_mesh = SnakeGameState::build_apple_mesh(ctx, &self.map_info)?;
            } else {
                if let Some(old_tail) = self.map_info.body.pop_back() {
                    self.map_info.empty_coord.insert(old_tail);
                }
            }

            if self.check_self_bite() || self.map_info.empty_coord.len() == 0 {
                self.map_info = SnakeGameGrid::new();
                self.apple_mesh = SnakeGameState::build_apple_mesh(ctx, &self.map_info)?;
                self.move_direction = MoveDirection::Right;
            }
            self.snake_mesh = SnakeGameState::build_snake_mesh(ctx, &self.map_info)?;
        }
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool,
    ) -> Result<(), GameError> {
        if !repeated {
            let key_info = input.event.logical_key;
            if let Key::Character(key_str) = key_info {
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

fn main() -> Result<(), GameError> {
    let mut snake_conf = Conf::new();
    snake_conf.window_setup.title = "Rust Snake By GGEZ".to_string();
    snake_conf.window_mode.width = GRID_COLS as f32 * SQUARE_LENGTH;
    snake_conf.window_mode.height = GRID_ROWS as f32 * SQUARE_LENGTH;

    let (mut snake_context, snake_game_loop) = ContextBuilder::new("snake", "zdy")
        .default_conf(snake_conf)
        .build()?;

    let snake_game_state = SnakeGameState::new(&mut snake_context)?;

    event::run(snake_context, snake_game_loop, snake_game_state)
}
