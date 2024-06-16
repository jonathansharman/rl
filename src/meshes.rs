use ggez::{
	glam::Vec2,
	graphics::{Color, DrawMode, Mesh, Rect},
	Context, GameResult,
};

pub struct Meshes {
	pub wall: Mesh,
	pub floor: Mesh,
	pub human: Mesh,
	pub goblin: Mesh,
}

impl Meshes {
	pub fn new(ctx: &mut Context) -> GameResult<Meshes> {
		Ok(Meshes {
			wall: Mesh::new_rectangle(
				ctx,
				DrawMode::fill(),
				Rect {
					x: 0.0,
					y: 0.0,
					w: 1.0,
					h: 1.0,
				},
				Color::from_rgb(128, 0, 0),
			)?,
			floor: Mesh::new_rectangle(
				ctx,
				DrawMode::fill(),
				Rect {
					x: 0.0,
					y: 0.0,
					w: 1.0,
					h: 1.0,
				},
				Color::from_rgb(128, 128, 128),
			)?,
			human: Mesh::new_ellipse(
				ctx,
				DrawMode::fill(),
				Vec2::new(0.0, 0.0),
				0.5,
				0.5,
				1.0,
				Color::BLUE,
			)?,
			goblin: Mesh::new_ellipse(
				ctx,
				DrawMode::fill(),
				Vec2::new(0.0, 0.0),
				0.5,
				0.5,
				1.0,
				Color::RED,
			)?,
		})
	}
}
