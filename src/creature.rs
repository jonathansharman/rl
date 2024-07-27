use ggez::graphics::{Canvas, DrawParam};

use crate::{coordinates::TilePoint, level::TileLayout, meshes::Meshes};

/// A type of [`Creature`].
#[derive(Clone, Copy, Debug)]
pub enum Species {
	Human,
	Goblin,
	Ogre,
}

impl Species {
	fn base_health(&self) -> u32 {
		match self {
			Species::Human => 10,
			Species::Goblin => 3,
			Species::Ogre => 15,
		}
	}

	fn base_strength(&self) -> u32 {
		match self {
			Species::Human => 2,
			Species::Goblin => 1,
			Species::Ogre => 3,
		}
	}
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
	health: u32,
	pub strength: u32,
}

impl Creature {
	pub fn new(
		species: Species,
		behavior: Behavior,
		coords: TilePoint,
	) -> Creature {
		Creature {
			species,
			behavior,
			coords,
			health: species.base_health(),
			strength: species.base_strength(),
		}
	}

	pub fn draw(
		&self,
		canvas: &mut Canvas,
		meshes: &Meshes,
		layout: &TileLayout,
	) {
		let tile_layout = layout.to_screen(self.coords);
		let mesh = match self.species {
			Species::Human => &meshes.human,
			Species::Goblin => &meshes.goblin,
			Species::Ogre => &meshes.ogre,
		};
		canvas.draw(
			mesh,
			DrawParam::new()
				.dest(tile_layout.pos + tile_layout.size / 2.0)
				.scale(tile_layout.size),
		);
	}

	pub fn take_damage(&mut self, damage: u32) {
		self.health = self.health.saturating_sub(damage);
	}

	pub fn dead(&self) -> bool {
		self.health == 0
	}
}
