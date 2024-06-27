use ggez::graphics::{Canvas, DrawParam};

use crate::{coordinates::TilePoint, level::Layout, meshes::Meshes};

/// A type of [`Creature`].
#[derive(Debug)]
pub enum Species {
	Human,
	Goblin,
}

#[derive(Debug)]
pub enum Behavior {
	PlayerControlled,
	AIControlled,
}

/// An animate being.
#[derive(Debug)]
pub struct Creature {
	pub species: Species,
	pub behavior: Behavior,
	pub coords: TilePoint,
	pub health: i32,
	pub strength: i32,
}

impl Creature {
	pub fn draw(&self, canvas: &mut Canvas, meshes: &Meshes, layout: &Layout) {
		let tile_layout = layout.tile_layout(self.coords);
		let mesh = match self.species {
			Species::Human => &meshes.human,
			Species::Goblin => &meshes.goblin,
		};
		canvas.draw(
			mesh,
			DrawParam::new()
				.dest(tile_layout.pos + tile_layout.size / 2.0)
				.scale(tile_layout.size),
		);
	}

	pub fn dead(&self) -> bool {
		self.health <= 0
	}
}
