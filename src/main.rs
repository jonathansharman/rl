mod coordinates;
mod creature;
mod level;
mod main_state;
mod meshes;
mod vision;

use coordinates::{ScreenPoint, ScreenRectangle, ScreenVector};
use ggez::{
	conf::{WindowMode, WindowSetup},
	event, GameError, GameResult,
};
use level::Level;
use main_state::MainState;
use meshes::Meshes;
use rand::prelude::*;
use rand_pcg::Pcg32;

fn main() -> GameResult {
	let viewport = ScreenRectangle {
		pos: ScreenPoint::new(0.0, 0.0),
		size: ScreenVector::new(1280.0, 720.0),
	};
	let mut rng: Pcg32 = Pcg32::from_entropy();
	let mut level = Level::generate(viewport, &mut rng);
	let player_id = level.spawn_player().map_err(|_| {
		GameError::CustomError("player spawning was blocked".into())
	})?;
	level.update_vision(player_id);

	let (mut ctx, event_loop) =
		ggez::ContextBuilder::new("RL", "Jonathan Sharman")
			.window_setup(WindowSetup {
				title: "RL".to_string(),
				// TODO: icon
				..Default::default()
			})
			.window_mode(WindowMode {
				width: viewport.size.x,
				height: viewport.size.y,
				resizable: true,
				..Default::default()
			})
			.build()?;
	let meshes = Meshes::new(&mut ctx)?;
	let state = MainState {
		player_id,
		level,
		meshes,
	};
	event::run(ctx, event_loop, state);
}
