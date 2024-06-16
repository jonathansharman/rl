use ggez::{
	event,
	graphics::{Canvas, Color},
	input::keyboard::{KeyCode, KeyInput},
	Context, GameResult,
};

use crate::{
	coordinates::{TileVector, TILE_DOWN, TILE_LEFT, TILE_RIGHT, TILE_UP},
	level::{Collision, Id, Level},
	meshes::Meshes,
};

enum Action {
	Move { offset: TileVector },
}

pub struct MainState {
	pub player_id: Id,
	pub level: Level,
	pub meshes: Meshes,
}

impl MainState {
	fn act(&mut self, action: Action) {
		match action {
			Action::Move { offset } => {
				match self.level.translate_object(self.player_id, offset) {
					Ok(_) => self.level.update_vision(self.player_id),
					Err(collision) => {
						if let Collision::Object(collider_id) = collision {
							// TODO: Handle collision.
						}
					}
				};
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
		self.level.draw(&mut canvas, &self.meshes)?;
		canvas.finish(ctx)
	}
}
