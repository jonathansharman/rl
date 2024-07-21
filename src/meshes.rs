use ggez::{
	glam::Vec2,
	graphics::{Color, DrawMode, Mesh, Rect},
	Context, GameResult,
};

pub struct Meshes {
	pub wall: Mesh,
	pub stone_floor: Mesh,
	pub wood_floor: Mesh,
	// Objects
	pub human: Mesh,
	pub goblin: Mesh,
	// Items
	pub item: Mesh,
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
			stone_floor: Mesh::new_rectangle(
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
			wood_floor: Mesh::new_rectangle(
				ctx,
				DrawMode::fill(),
				Rect {
					x: 0.0,
					y: 0.0,
					w: 1.0,
					h: 1.0,
				},
				Color::from_rgb(96, 58, 32),
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
			item: Mesh::new_rectangle(
				ctx,
				DrawMode::fill(),
				Rect {
					x: -0.4,
					y: -0.4,
					w: 0.8,
					h: 0.8,
				},
				Color::GREEN,
			)?,
		})
	}
}
