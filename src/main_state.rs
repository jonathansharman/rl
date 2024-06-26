use ggez::{
	event,
	graphics::{Canvas, Color},
	input::keyboard::{KeyCode, KeyInput},
	Context, GameResult,
};

use crate::{
	coordinates::{TileVector, TILE_DOWN, TILE_LEFT, TILE_RIGHT, TILE_UP},
	creature::Creature,
	level::Level,
	meshes::Meshes,
	shared::Shared,
};

enum Action {
	Move { offset: TileVector },
}

pub struct MainState {
	pub player: Shared<Creature>,
	pub level: Level,
	pub meshes: Meshes,
}

impl MainState {
	fn act(&mut self, action: Action) {
		match action {
			Action::Move { offset } => {
				let player = &mut self.player.borrow_mut();
				self.level.translate_creature(player, offset);
				self.level.update_vision(player.coords);
			}
		}
	}
}

impl event::EventHandler<ggez::GameError> for MainState {
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
		let action = match keycode {
			KeyCode::Escape => {
				ctx.request_quit();
				None
			}
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
