use ggez::{
    Context, ContextBuilder, GameError,
    conf::Conf,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawMode, Mesh, MeshBuilder, Rect},
    winit::keyboard::Key,
};
use rand::prelude::*;
use std::collections::VecDeque;

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

#[derive(Clone, Copy, PartialEq, Eq)]
struct SnakeGameCoord {
    x: u32,
    y: u32,
}

impl SnakeGameCoord {
    pub fn new(x: u32, y: u32) -> Self {
        SnakeGameCoord { x, y }
    }
}

struct SnakeGrid {
    apple_coord: SnakeGameCoord,
    body_deque: VecDeque<SnakeGameCoord>,
}

impl SnakeGrid {
    pub fn new() -> Self {
        let apple_coord = SnakeGameCoord::new(INIT_APPLE_X, INIT_APPLE_Y);
        let mut body_deque: VecDeque<SnakeGameCoord> = VecDeque::new();
        for i in INIT_BODY_LEFT..=INIT_BODY_RIGHT {
            let body_cord = SnakeGameCoord::new(i, INIT_APPLE_Y);
            body_deque.push_front(body_cord);
        }
        SnakeGrid {
            apple_coord,
            body_deque,
        }
    }

    fn is_valid_apple_cord(&self, x: u32, y: u32) -> bool {
        if x == self.apple_coord.x && y == self.apple_coord.y {
            return false;
        }
        let cur_cord = SnakeGameCoord::new(x, y);
        if self.body_deque.contains(&cur_cord) {
            return false;
        }
        true
    }

    pub fn spawn_apple(&mut self) {
        let mut rng = rand::rng();
        let mut x: u32 = rng.random_range(0..GRID_COLS);
        let mut y: u32 = rng.random_range(0..GRID_ROWS);
        while !self.is_valid_apple_cord(x, y) {
            x = rng.random_range(0..GRID_COLS);
            y = rng.random_range(0..GRID_ROWS);
        }

        self.apple_coord.x = x;
        self.apple_coord.y = y;
    }

    pub fn is_eating_apple(&self) -> bool {
        self.body_deque.front() == Some(&self.apple_coord)
    }
}

struct SnakeGameState {
    background_mesh: Mesh,
    apple_mesh: Mesh,
    snake_mesh: Mesh,
    map_info: SnakeGrid,
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

    fn build_apple_mesh(ctx: &mut Context, map_info: &SnakeGrid) -> Result<Mesh, GameError> {
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

    fn build_snake_mesh(ctx: &mut Context, map_info: &SnakeGrid) -> Result<Mesh, GameError> {
        let mut builder = MeshBuilder::new();
        for element in &map_info.body_deque {
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
        let map_info = SnakeGrid::new();

        let background_mesh = SnakeGameState::build_background_mesh(ctx).unwrap();
        let apple_mesh = SnakeGameState::build_apple_mesh(ctx, &map_info).unwrap();
        let snake_mesh = SnakeGameState::build_snake_mesh(ctx, &map_info).unwrap();

        Ok(SnakeGameState {
            background_mesh: background_mesh,
            apple_mesh: apple_mesh,
            snake_mesh: snake_mesh,
            map_info: map_info,
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

    fn check_snake_loop(&mut self, ctx: &mut Context) {
        let snake_head = self.map_info.body_deque.front().unwrap();
        if self
            .map_info
            .body_deque
            .iter()
            .skip(1)
            .any(|seg| seg == snake_head)
        {
            self.map_info = SnakeGrid::new();
            self.apple_mesh = SnakeGameState::build_apple_mesh(ctx, &self.map_info).unwrap();
            self.snake_mesh = SnakeGameState::build_snake_mesh(ctx, &self.map_info).unwrap();
            self.move_direction = MoveDirection::Right;
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
            self.apple_mesh = SnakeGameState::build_apple_mesh(ctx, &self.map_info).unwrap();
            self.snake_mesh = SnakeGameState::build_snake_mesh(ctx, &self.map_info).unwrap();
            let mut head_x = self.map_info.body_deque.front().unwrap().x;
            let mut head_y = self.map_info.body_deque.front().unwrap().y;
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
            let new_head = SnakeGameCoord::new(head_x, head_y);
            self.map_info.body_deque.push_front(new_head);
            if self.map_info.is_eating_apple() {
                self.map_info.spawn_apple();
            } else {
                self.map_info.body_deque.pop_back();
            }
            self.check_snake_loop(ctx);
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
        .build()
        .unwrap();

    let snake_game_state = SnakeGameState::new(&mut snake_context)?;

    event::run(snake_context, snake_game_loop, snake_game_state)
}
