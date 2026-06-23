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
const GRID_COLS: u32 = 20;
const GRID_ROWS: u32 = 20;

struct SnakeGameState {
    background_mesh: Mesh,
}

impl SnakeGameState {
    pub fn new(ctx: &mut Context) -> Result<Self, GameError> {
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

        let background_mesh = graphics::Mesh::from_data(ctx, builder.build());

        Ok(SnakeGameState {
            background_mesh: background_mesh,
        })
    }

    pub fn draw_background(&self, my_canvas: &mut Canvas) -> Result<(), GameError> {
        my_canvas.draw(&self.background_mesh, Vec2::new(0.0, 0.0));
        Ok(())
    }
}

impl EventHandler for SnakeGameState {
    fn draw(&mut self, ctx: &mut Context) -> Result<(), GameError> {
        let mut my_canvas = Canvas::from_frame(ctx, Color::BLACK);
        self.draw_background(&mut my_canvas)?;
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
        println!(
            "Key pressed: physical key {:?}, logical key {:?}, modifier {:?}, repeat: {}",
            input.event.physical_key, input.event.logical_key, input.mods, repeated
        );

        let key_info = input.event.logical_key;
        match key_info {
            Key::Character(key) => {
                if key == "d" {
                    println!("D pressed!");
                }
            }
            _ => {}
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
