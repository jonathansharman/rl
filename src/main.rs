mod creature;
mod dijkstra_map;
mod disjoint_sets;
mod game_state;
mod geometry;
mod item;
mod level;
mod meshes;
mod shared;
mod vision;

use game_state::GameState;
use geometry::{
	ScreenPoint, ScreenRectangle, ScreenVector, TilePoint, TileRectangle,
	TileVector,
};
use ggez::{
	conf::{WindowMode, WindowSetup},
	event, GameResult,
};
use level::Level;
use meshes::Meshes;
use rand::prelude::*;
use rand_pcg::Pcg32;

fn main() -> GameResult {
	let viewport = ScreenRectangle {
		pos: ScreenPoint::new(0.0, 0.0),
		size: ScreenVector::new(1920.0, 1080.0),
	};
	let mut rng: Pcg32 = Pcg32::from_entropy();
	let mut level = Level::generate(
		level::GenerationConfig {
			viewport,
			// 30-px tiles fitting snugly in a 1920 x 1080 viewport
			tileport: TileRectangle {
				pos: TilePoint::new(0, 0),
				size: TileVector::new(64, 36),
			},
			min_floor_ratio: 0.4,
			min_room_size: 3,
			max_room_size: 15,
		},
		&mut rng,
	);
	let player = level.spawn_player(&mut rng);
	level.update_dijkstra_maps();
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
				maximized: true,
				fullscreen_type: ggez::conf::FullscreenType::Desktop,
				resizable: true,
				..Default::default()
			})
			.build()?;
	let meshes = Meshes::new(&mut ctx)?;
	let state = GameState {
		rng,
		player,
		level,
		meshes,
	};
	event::run(ctx, event_loop, state);
}
