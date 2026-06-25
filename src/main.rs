use ggez::{
    Context, ContextBuilder, GameError,
    conf::Conf,
    event::{self, EventHandler},
    glam::Vec2,
    graphics::{self, Canvas, Color, DrawMode, Mesh, MeshBuilder, Rect},
    winit::keyboard::Key,
};

const TARGET_FPS: u32 = 30;
const SQUARE_LENGTH: f32 = 60.0;
const GRID_COLS: u32 = 21;
const GRID_ROWS: u32 = 21;

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

struct SnakeGridMap {
    map_info: Vec<SnakeGameElement>,
}

impl SnakeGridMap {
    pub fn new() -> Result<Self, GameError> {
        let mut map_info: Vec<SnakeGameElement> =
            vec![SnakeGameElement::Empty; (GRID_COLS * GRID_ROWS) as usize];
        map_info[224] = SnakeGameElement::Apple;
        for i in 216..219 {
            map_info[i] = SnakeGameElement::Body;
        }
        Ok(SnakeGridMap { map_info: map_info })
    }

    fn cal_ind(x: u32, y: u32) -> usize {
        (x + y * GRID_COLS) as usize
    }

    pub fn get_element(&self, x: u32, y: u32) -> Result<SnakeGameElement, GameError> {
        let ind: usize = SnakeGridMap::cal_ind(x, y);
        Ok(self.map_info[ind])
    }

    pub fn set_element(
        &mut self,
        x: u32,
        y: u32,
        element: SnakeGameElement,
    ) -> Result<(), GameError> {
        let ind: usize = SnakeGridMap::cal_ind(x, y);
        self.map_info[ind] = element;
        Ok(())
    }

    pub fn is_apple(&self, x: u32, y: u32) -> bool {
        let ind: usize = SnakeGridMap::cal_ind(x, y);
        self.map_info[ind] == SnakeGameElement::Apple
    }

    pub fn is_empty(&self, x: u32, y: u32) -> bool {
        let ind: usize = SnakeGridMap::cal_ind(x, y);
        self.map_info[ind] == SnakeGameElement::Empty
    }

    pub fn is_body(&self, x: u32, y: u32) -> bool {
        let ind: usize = SnakeGridMap::cal_ind(x, y);
        self.map_info[ind] == SnakeGameElement::Body
    }
}

struct SnakeGameState {
    mesh: Mesh,

    head_index: usize,

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
                            Color::GREEN,
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
            head_index: 218,
            map_info: map_info,
            move_direc: MoveDirection::Right,
        })
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
        if ctx.time.check_update_time(TARGET_FPS) {}
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        repeated: bool,
    ) -> Result<(), GameError> {
        if !repeated {
            println!(
                "Key pressed: physical key {:?}, logical key {:?}, modifier {:?}, repeat: {}",
                input.event.physical_key, input.event.logical_key, input.mods, repeated
            );

            let key_info = input.event.logical_key;

            match self.move_direc {
                MoveDirection::Left => match key_info {
                    Key::Character(key) => {
                        if key == "s" {
                            self.move_direc = MoveDirection::Down;
                            println!("S pressed!");
                        } else if key == "w" {
                            self.move_direc = MoveDirection::Up;
                            println!("W pressed!");
                        }
                    }
                    _ => {}
                },
                MoveDirection::Up => match key_info {
                    Key::Character(key) => {
                        if key == "a" {
                            self.move_direc = MoveDirection::Left;
                            println!("A pressed!");
                        } else if key == "d" {
                            self.move_direc = MoveDirection::Right;
                            println!("D pressed!");
                        }
                    }
                    _ => {}
                },
                MoveDirection::Right => match key_info {
                    Key::Character(key) => {
                        if key == "s" {
                            self.move_direc = MoveDirection::Down;
                            println!("S pressed!");
                        } else if key == "w" {
                            self.move_direc = MoveDirection::Up;
                            println!("W pressed!");
                        }
                    }
                    _ => {}
                },
                MoveDirection::Down => match key_info {
                    Key::Character(key) => {
                        if key == "a" {
                            self.move_direc = MoveDirection::Left;
                            println!("A pressed!");
                        } else if key == "d" {
                            self.move_direc = MoveDirection::Right;
                            println!("D pressed!");
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

    let mut snake_game_state = SnakeGameState::new(&mut snake_context)?;

    event::run(snake_context, snake_game_loop, snake_game_state)
}
