use ggez::graphics::{Canvas, DrawParam};
use rand_pcg::Pcg32;

use crate::{
	coordinates::{random_neighbor_four, TilePoint},
	level::{Level, TileLayout},
	meshes::Meshes,
};

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
	Idle,
	Wandering,
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

	pub fn act(&mut self, level: &mut Level, rng: &mut Pcg32) {
		match self.behavior {
			Behavior::Idle => {}
			Behavior::Wandering => {
				// Move in a random direction.
				level.translate_creature(self, random_neighbor_four(rng));
			}
		}
	}

	pub fn take_damage(&mut self, damage: u32) {
		self.health = self.health.saturating_sub(damage);
	}

	pub fn dead(&self) -> bool {
		self.health == 0
	}
}
