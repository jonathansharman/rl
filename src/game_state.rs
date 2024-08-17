use ggez::{
	event,
	graphics::{Canvas, Color},
	input::keyboard::{KeyCode, KeyInput},
	Context, GameResult,
};
use rand_pcg::Pcg32;

use crate::{
	creature::Creature,
	geometry::{TileVector, TILE_DOWN, TILE_LEFT, TILE_RIGHT, TILE_UP},
	level::Level,
	meshes::Meshes,
	shared::Shared,
};

enum Action {
	Wait,
	Move { offset: TileVector },
}

pub struct GameState {
	pub rng: Pcg32,
	pub player: Shared<Creature>,
	pub level: Level,
	pub meshes: Meshes,
}

impl GameState {
	fn act(&mut self, action: Action) {
		match action {
			Action::Wait => {}
			Action::Move { offset } => {
				self.level
					.translate_creature(&mut self.player.borrow_mut(), offset);
			}
		}
		self.level.update_dijkstra_maps();
		self.level.update(&mut self.rng);
		self.level.update_vision(self.player.borrow().coords);
	}
}

impl event::EventHandler<ggez::GameError> for GameState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
	}

	fn key_down_event(
		&mut self,
		ctx: &mut Context,
		input: KeyInput,
		_repeat: bool,
	) -> GameResult {
		let Some(keycode) = input.keycode else {
			return Ok(());
		};

		if let KeyCode::Escape = keycode {
			ctx.request_quit();
		}

		// Disable player actions when dead.
		if self.player.borrow().dead() {
			return Ok(());
		}

		let action = match keycode {
			KeyCode::Space | KeyCode::Z => Some(Action::Wait),
			KeyCode::Up => Some(Action::Move { offset: TILE_UP }),
			KeyCode::Down => Some(Action::Move { offset: TILE_DOWN }),
			KeyCode::Left => Some(Action::Move { offset: TILE_LEFT }),
			KeyCode::Right => Some(Action::Move { offset: TILE_RIGHT }),
			_ => None,
		};
		if let Some(action) = action {
			self.act(action);
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
		self.level.draw(&mut canvas, &self.meshes);
		canvas.finish(ctx)
	}
}
