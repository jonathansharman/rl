use ggez::graphics::{Canvas, DrawParam};
use rand_pcg::Pcg32;

use crate::{
	geometry::{random_neighbor_offset_four, TilePoint},
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
	fn base_stats(&self) -> Stats {
		match self {
			Species::Human => Stats {
				health: 10,
				strength: 2,
			},
			Species::Goblin => Stats {
				health: 5,
				strength: 1,
			},
			Species::Ogre => Stats {
				health: 15,
				strength: 3,
			},
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Faction {
	Ally,
	Enemy,
}

#[derive(Debug)]
pub enum Behavior {
	Idle,
	Patrolling,
}

#[derive(Debug)]
pub struct Stats {
	health: u32,
	pub strength: u32,
}

/// An animate being.
#[derive(Debug)]
pub struct Creature {
	pub species: Species,
	pub faction: Faction,
	pub behavior: Behavior,
	pub coords: TilePoint,
	pub stats: Stats,
}

impl Creature {
	pub fn new(
		faction: Faction,
		species: Species,
		behavior: Behavior,
		coords: TilePoint,
	) -> Creature {
		Creature {
			species,
			faction,
			behavior,
			coords,
			stats: species.base_stats(),
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
			Behavior::Patrolling => {
				if let Some(map) =
					level.dijkstra_maps().enemies.get(&self.faction)
				{
					// TODO: Statify range and do a LOS check. The LOS check
					// will require getting the target tile, not just the next
					// step. Ideally, I'd get all possible targets in range -
					// not just the single closest - and choose the closest with
					// LOS. Otherwise, if there are two targets but the creature
					// doesn't have LOS to the closest one, it would be blind to
					// the second target.
					let step = if self.stats.health == 1 {
						// Retreat when low on health.
						map.step_away(self.coords, rng)
					} else {
						map.step_towards(self.coords, rng)
					};
					if let Some(offset) = step {
						let target = self.coords + offset;
						if map.distance(target).unwrap() < 10 {
							return level.translate_creature(self, offset);
						}
					} else {
						// Already at a locally optimal location - do nothing.
						return;
					}
				}
				// Wander in a random direction.
				level.translate_creature(self, random_neighbor_offset_four(rng))
			}
		}
	}

	pub fn take_damage(&mut self, damage: u32) {
		self.stats.health = self.stats.health.saturating_sub(damage);
	}

	pub fn dead(&self) -> bool {
		self.stats.health == 0
	}
}
