mod coordinates;
mod creature;
mod item;
mod level;
mod main_state;
mod meshes;
mod shared;
mod vision;

use coordinates::{ScreenPoint, ScreenRectangle, ScreenVector};
use ggez::{
	conf::{WindowMode, WindowSetup},
	event, GameResult,
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
	let player = level.spawn_player();
	level.update_vision(player.borrow().coords);

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
		player,
		level,
		meshes,
	};
	event::run(ctx, event_loop, state);
}
