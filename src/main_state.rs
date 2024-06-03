use ggez::{
	event,
	graphics::{Canvas, Color},
	input::keyboard::{KeyCode, KeyInput},
	Context, GameResult,
};

use crate::{
	coordinates::{TILE_DOWN, TILE_LEFT, TILE_RIGHT, TILE_UP},
	level::{Id, Level},
	meshes::Meshes,
};

pub struct MainState {
	pub player_id: Id,
	pub level: Level,
	pub meshes: Meshes,
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
		match keycode {
			KeyCode::Escape => {
				ctx.request_quit();
			}
			KeyCode::Up => {
				self.level.translate_object(self.player_id, TILE_UP);
				self.level.update_vision(self.player_id);
			}
			KeyCode::Down => {
				self.level.translate_object(self.player_id, TILE_DOWN);
				self.level.update_vision(self.player_id);
			}
			KeyCode::Left => {
				self.level.translate_object(self.player_id, TILE_LEFT);
				self.level.update_vision(self.player_id);
			}
			KeyCode::Right => {
				self.level.translate_object(self.player_id, TILE_RIGHT);
				self.level.update_vision(self.player_id);
			}
			_ => {}
		}
		Ok(())
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
		self.level.draw(&mut canvas, &self.meshes)?;
		canvas.finish(ctx)
	}
}
