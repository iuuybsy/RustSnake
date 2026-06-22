use ggez::{
    Context, ContextBuilder, GameError,
    conf::Conf,
    event::{self, EventHandler},
    winit::keyboard::{KeyCode, PhysicalKey},
};

const TARGET_FPS: u32 = 30;

struct SnakeGameState {}

impl SnakeGameState {
    pub fn new() -> Self {
        SnakeGameState {}
    }
}

impl EventHandler for SnakeGameState {
    fn draw(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
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

        let key_info = input.event.physical_key;
        match key_info {
            PhysicalKey::Code(key_code) => {
                if key_code == KeyCode::KeyD {
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

    let (snake_context, snake_game_loop) = ContextBuilder::new("snake", "zdy")
        .default_conf(snake_conf)
        .build()
        .unwrap();

    let snake_game_state = SnakeGameState::new();

    event::run(snake_context, snake_game_loop, snake_game_state)
}
