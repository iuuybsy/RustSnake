use ggez::{
    conf::Conf,
    event::{self, EventHandler},
    *,
};

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

    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }
}

fn main() {
    let mut snake_conf = Conf::new();
    snake_conf.window_setup.title = "Rust Snake By GGEZ".to_string();

    let (snake_context, snake_game_loop) = ContextBuilder::new("snake", "zdy")
        .default_conf(snake_conf)
        .build()
        .unwrap();

    let snake_game_state = SnakeGameState::new();

    let _ = event::run(snake_context, snake_game_loop, snake_game_state);
}
