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

#[derive(Clone, Copy, PartialEq)]
enum SnakeGameElement {
    Body,
    Apple,
    Empty,
}

enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq)]
struct SnakeGameCord {
    x: u32,
    y: u32,
}

impl SnakeGameCord {
    pub fn new(x: u32, y: u32) -> Self {
        SnakeGameCord { x, y }
    }
}

struct SnakeGridMap {
    app_cord: SnakeGameCord,
    body_deque: VecDeque<SnakeGameCord>,
}

impl SnakeGridMap {
    pub fn new() -> Result<Self, GameError> {
        let apple_x = (GRID_COLS + 1) / 2 + 3;
        let apple_y = (GRID_ROWS + 1) / 2;
        let app_cord = SnakeGameCord::new(apple_x, apple_y);
        let mut body_deque: VecDeque<SnakeGameCord> = VecDeque::new();
        for i in 6..9 {
            let body_cord = SnakeGameCord::new(i, apple_y);
            body_deque.push_front(body_cord);
        }
        Ok(SnakeGridMap {
            app_cord: app_cord,
            body_deque: body_deque,
        })
    }

    fn is_valid_apple_cord(&self, x: u32, y: u32) -> bool {
        if x == self.app_cord.x && y == self.app_cord.y {
            return false;
        }
        let cur_cord = SnakeGameCord::new(x, y);
        if self.body_deque.contains(&cur_cord) {
            return false;
        }
        true
    }

    pub fn gen_new_apple(&mut self) -> Result<(), GameError> {
        let mut rng = rand::rng();
        let mut x: u32 = rng.random_range(0..GRID_COLS);
        let mut y: u32 = rng.random_range(0..GRID_ROWS);
        while !self.is_valid_apple_cord(x, y) {
            x = rng.random_range(0..GRID_COLS);
            y = rng.random_range(0..GRID_ROWS);
        }

        self.app_cord.x = x;
        self.app_cord.y = y;

        Ok(())
    }

    pub fn get_element(&self, x: u32, y: u32) -> Result<SnakeGameElement, GameError> {
        if x == self.app_cord.x && y == self.app_cord.y {
            return Ok(SnakeGameElement::Apple);
        }
        let cur_cord = SnakeGameCord::new(x, y);
        if self.body_deque.contains(&cur_cord) {
            return Ok(SnakeGameElement::Body);
        }
        Ok(SnakeGameElement::Empty)
    }

    pub fn is_eating_apple(&self) -> bool {
        self.body_deque.contains(&self.app_cord)
    }
}

struct SnakeGameState {
    mesh: Mesh,
    map_info: SnakeGridMap,
    move_direc: MoveDirection,
}

impl SnakeGameState {
    fn gen_mesh(ctx: &mut Context, map_info: &SnakeGridMap) -> Result<Mesh, GameError> {
        let silver_color = Color::from_rgb(192, 192, 192);
        let grey_color = Color::from_rgb(128, 128, 128);

        let mut builder = MeshBuilder::new();

        for i in 0..GRID_COLS {
            for j in 0..GRID_ROWS {
                let x_pos = i as f32 * SQUARE_LENGTH;
                let y_pos = j as f32 * SQUARE_LENGTH;

                match map_info.get_element(i, j).unwrap() {
                    SnakeGameElement::Empty => {
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
                    SnakeGameElement::Apple => {
                        builder.rectangle(
                            DrawMode::fill(),
                            Rect::new(x_pos, y_pos, SQUARE_LENGTH, SQUARE_LENGTH),
                            Color::RED,
                        )?;
                    }
                    SnakeGameElement::Body => {
                        builder.rectangle(
                            DrawMode::fill(),
                            Rect::new(x_pos, y_pos, SQUARE_LENGTH, SQUARE_LENGTH),
                            Color::BLACK,
                        )?;
                    }
                }
            }
        }

        let mesh = graphics::Mesh::from_data(ctx, builder.build());
        Ok(mesh)
    }

    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
        let map_info = SnakeGridMap::new().unwrap();
        let mesh = SnakeGameState::gen_mesh(ctx, &map_info).unwrap();

        Ok(SnakeGameState {
            mesh: mesh,
            map_info: map_info,
            move_direc: MoveDirection::Right,
        })
    }

    fn check_snake_loop(&mut self, ctx: &mut Context) {
        let head_x = self.map_info.body_deque.front().unwrap().x;
        let head_y = self.map_info.body_deque.front().unwrap().y;
        let mut count = 0;
        for element in &self.map_info.body_deque {
            if head_x == element.x && head_y == element.y {
                count += 1;
                if count > 1 {
                    break;
                }
            }
        }
        if count > 1 {
            self.map_info = SnakeGridMap::new().unwrap();
            self.mesh = SnakeGameState::gen_mesh(ctx, &self.map_info).unwrap();
            self.move_direc = MoveDirection::Right;
        }
    }
}

impl EventHandler for SnakeGameState {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut my_canvas = Canvas::from_frame(ctx, Color::BLACK);
        my_canvas.draw(&self.mesh, Vec2::ZERO);
        my_canvas.finish(ctx)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        if ctx.time.check_update_time(TARGET_FPS) {
            self.mesh = SnakeGameState::gen_mesh(ctx, &self.map_info).unwrap();
            let mut head_x = self.map_info.body_deque.front().unwrap().x;
            let mut head_y = self.map_info.body_deque.front().unwrap().y;
            match self.move_direc {
                MoveDirection::Down => {
                    head_y = (head_y + 1) % GRID_ROWS;
                }
                MoveDirection::Left => {
                    if head_x == 0 {
                        head_x = GRID_COLS - 1;
                    } else {
                        head_x = head_x - 1;
                    }
                }
                MoveDirection::Up => {
                    if head_y == 0 {
                        head_y = GRID_COLS - 1;
                    } else {
                        head_y = head_y - 1;
                    }
                }
                MoveDirection::Right => {
                    head_x = (head_x + 1) % GRID_COLS;
                }
            }
            let new_head = SnakeGameCord::new(head_x, head_y);
            self.map_info.body_deque.push_front(new_head);
            if self.map_info.is_eating_apple() {
                self.map_info.gen_new_apple()?;
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

            match self.move_direc {
                MoveDirection::Left => match key_info {
                    Key::Character(key) => {
                        if key == "s" {
                            self.move_direc = MoveDirection::Down;
                        } else if key == "w" {
                            self.move_direc = MoveDirection::Up;
                        }
                    }
                    _ => {}
                },
                MoveDirection::Up => match key_info {
                    Key::Character(key) => {
                        if key == "a" {
                            self.move_direc = MoveDirection::Left;
                        } else if key == "d" {
                            self.move_direc = MoveDirection::Right;
                        }
                    }
                    _ => {}
                },
                MoveDirection::Right => match key_info {
                    Key::Character(key) => {
                        if key == "s" {
                            self.move_direc = MoveDirection::Down;
                        } else if key == "w" {
                            self.move_direc = MoveDirection::Up;
                        }
                    }
                    _ => {}
                },
                MoveDirection::Down => match key_info {
                    Key::Character(key) => {
                        if key == "a" {
                            self.move_direc = MoveDirection::Left;
                        } else if key == "d" {
                            self.move_direc = MoveDirection::Right;
                        }
                    }
                    _ => {}
                },
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
