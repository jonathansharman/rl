use std::collections::{HashMap, HashSet};

use ggez::graphics::{Canvas, Color, DrawParam};
use rand::Rng;
use rand_pcg::Pcg32;

use crate::{
	coordinates::{
		ScreenPoint, ScreenRectangle, ScreenVector, TilePoint, TileRectangle,
		TileVector,
	},
	creature::{Creature, Species},
	item::Item,
	meshes::Meshes,
	shared::{share, Shared},
	vision,
};

pub struct Layout {
	// The region of the screen to map this layout to.
	viewport: ScreenRectangle,
	// Tile rectangle containing all the tiles that may need to be displayed.
	tileport: TileRectangle,
	// Tile width and height on-screen.
	tile_size: ScreenVector,
}

impl Layout {
	fn new(viewport: ScreenRectangle, tileport: TileRectangle) -> Layout {
		let tile_size = ScreenVector::new(
			viewport.size.x / tileport.size.x as f32,
			viewport.size.y / tileport.size.y as f32,
		);
		Layout {
			viewport,
			tileport,
			tile_size,
		}
	}

	pub fn tile_layout(&self, coords: TilePoint) -> ScreenRectangle {
		let pos = ScreenPoint::new(
			self.viewport.pos.x
				+ self.tile_size.x * (coords.x - self.tileport.pos.x) as f32,
			self.viewport.pos.y
				+ self.tile_size.y * (coords.y - self.tileport.pos.y) as f32,
		);
		ScreenRectangle {
			pos,
			size: self.tile_size - ScreenVector::new(1.0, 1.0),
		}
	}
}

enum Perception {
	Seen,
	Remembered,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
	Floor,
	Wall,
}

impl Tile {
	fn draw(
		&self,
		canvas: &mut Canvas,
		meshes: &Meshes,
		layout: &Layout,
		coords: TilePoint,
		perception: Perception,
	) {
		let color = match perception {
			Perception::Seen => Color::WHITE,
			Perception::Remembered => Color::from_rgba(255, 255, 255, 64),
		};
		let tile_layout = layout.tile_layout(coords);
		match self {
			Tile::Floor => {
				canvas.draw(
					&meshes.floor,
					DrawParam::new()
						.dest(tile_layout.pos)
						.scale(tile_layout.size)
						.color(color),
				);
			}
			Tile::Wall => {
				canvas.draw(
					&meshes.wall,
					DrawParam::new()
						.dest(tile_layout.pos)
						.scale(tile_layout.size)
						.color(color),
				);
			}
		}
	}
}

#[derive(Debug)]
pub enum Collision {
	OutOfBounds,
	Tile(Tile),
	Object(Shared<Creature>),
}

pub struct Level {
	layout: Layout,
	terrain: HashMap<TilePoint, Tile>,
	creatures: HashMap<TilePoint, Shared<Creature>>,
	items: HashMap<TilePoint, Shared<Item>>,
	/// Points the player can currently see.
	vision: HashSet<TilePoint>,
	/// Tiles the player remembers seeing.
	memory: HashMap<TilePoint, Tile>,
}

impl Level {
	pub fn generate(viewport: ScreenRectangle, rng: &mut Pcg32) -> Level {
		struct Room {
			center: TilePoint,
			radius: TileVector,
		}

		let rooms = std::iter::from_fn(|| {
			Some(Room {
				center: TilePoint::new(
					rng.gen_range(0..64),
					rng.gen_range(0..36),
				),
				radius: TileVector::new(
					rng.gen_range(3..5),
					rng.gen_range(3..5),
				),
			})
		})
		.take(5)
		.collect::<Vec<_>>();

		let mut terrain = HashMap::new();
		let make_floor = |terrain: &mut HashMap<TilePoint, Tile>,
		                  coords: TilePoint| {
			for x in coords.x - 1..=coords.x + 1 {
				for y in coords.y - 1..=coords.y + 1 {
					if x == coords.x && y == coords.y {
						terrain.insert(coords, Tile::Floor);
					} else {
						terrain
							.entry(TilePoint::new(x, y))
							.or_insert(Tile::Wall);
					}
				}
			}
		};
		// Open the floor of each room.
		for room in rooms.iter() {
			let x_min = room.center.x - room.radius.x;
			let x_max = room.center.x + room.radius.x;
			let y_min = room.center.y - room.radius.y;
			let y_max = room.center.y + room.radius.y;
			for x in x_min..=x_max {
				for y in y_min..=y_max {
					make_floor(&mut terrain, TilePoint::new(x, y));
				}
			}
		}
		// Connect each room to each other.
		for (i, room1) in rooms.iter().enumerate() {
			for room2 in rooms.iter().skip(i) {
				let mut coords = room1.center;
				while coords.x < room2.center.x {
					make_floor(&mut terrain, coords);
					coords.x += 1;
				}
				while coords.x > room2.center.x {
					make_floor(&mut terrain, coords);
					coords.x -= 1;
				}
				while coords.y < room2.center.y {
					make_floor(&mut terrain, coords);
					coords.y += 1;
				}
				while coords.y > room2.center.y {
					make_floor(&mut terrain, coords);
					coords.y -= 1;
				}
			}
		}

		let min_tile_x = terrain.keys().map(|coords| coords.x).min().unwrap();
		let min_tile_y = terrain.keys().map(|coords| coords.y).min().unwrap();
		let max_tile_x = terrain.keys().map(|coords| coords.x).max().unwrap();
		let max_tile_y = terrain.keys().map(|coords| coords.y).max().unwrap();
		let tileport = TileRectangle {
			pos: TilePoint::new(min_tile_x, min_tile_y),
			size: TileVector::new(
				max_tile_x - min_tile_x + 1,
				max_tile_y - min_tile_y + 1,
			),
		};
		let mut level = Level {
			layout: Layout::new(viewport, tileport),
			terrain,
			creatures: HashMap::new(),
			items: HashMap::new(),
			vision: HashSet::new(),
			memory: HashMap::new(),
		};

		// Spawn some creatures.
		for room in rooms {
			level
				.spawn(share(Creature {
					species: Species::Goblin,
					coords: room.center,
					health: 3,
					strength: 1,
				}))
				.unwrap();
		}

		level
	}

	/// Updates vision and memory using the given viewer `origin`.
	pub fn update_vision(&mut self, origin: TilePoint) {
		self.vision = vision::get_vision(origin, |coords: TilePoint| {
			!matches!(self.terrain.get(&coords), Some(Tile::Floor))
		});
		for coords in &self.vision {
			if let Some(tile) = self.terrain.get(coords) {
				self.memory.insert(*coords, *tile);
			}
		}
	}

	/// Draw everything in the level.
	pub fn draw(&self, canvas: &mut Canvas, meshes: &Meshes) {
		// Draw all remembered tiles that are not currently visible.
		for (coords, tile) in &self.memory {
			if !self.vision.contains(coords) {
				tile.draw(
					canvas,
					meshes,
					&self.layout,
					*coords,
					Perception::Remembered,
				);
			}
		}
		// Draw visible tiles and objects.
		for (coords, tile) in &self.terrain {
			if self.vision.contains(coords) {
				tile.draw(
					canvas,
					meshes,
					&self.layout,
					*coords,
					Perception::Seen,
				);
			}
		}
		for creature in self.creatures.values() {
			let creature = creature.borrow();
			if self.vision.contains(&creature.coords) {
				creature.draw(canvas, meshes, &self.layout);
			}
		}
	}

	fn spawn(
		&mut self,
		creature: Shared<Creature>,
	) -> Result<Shared<Creature>, Collision> {
		let coords = creature.borrow().coords;
		if let Some(collision) = self.collision(coords) {
			return Err(collision);
		}
		self.creatures.insert(coords, creature.clone());
		Ok(creature)
	}

	/// Spawns the player character at an arbitrary open tile. If a spot can't
	/// be found, this panics.
	pub fn spawn_player(&mut self) -> Shared<Creature> {
		let player_coords = self
			.terrain
			.iter()
			.find_map(|(&coords, &tile)| {
				(tile == Tile::Floor && !self.creatures.contains_key(&coords))
					.then_some(coords)
			})
			.unwrap();
		self.spawn(share(Creature {
			species: Species::Human,
			coords: player_coords,
			health: 10,
			strength: 2,
		}))
		.unwrap()
	}

	/// Attempts to translate `creature`'s position by `offset`, handling any
	/// resulting collisions. The creature must exist in the level, or this
	/// panics.
	pub fn translate_creature(
		&mut self,
		creature: &mut Creature,
		offset: TileVector,
	) {
		self.move_creature(creature, creature.coords + offset)
	}

	/// Attempts to move `creature` to `to`, handling any resulting collisions.
	/// The creature must exist in the level, or this panics.
	pub fn move_creature(&mut self, creature: &mut Creature, to: TilePoint) {
		let from = creature.coords;
		if let Some(collision) = self.collision(to) {
			let Collision::Object(other) = collision else {
				return;
			};
			self.attack(creature, &mut other.borrow_mut());
			return;
		}
		// The removed creature should be a reference to the creature parameter.
		let removed = self.creatures.remove(&from).unwrap();
		creature.coords = to;
		self.creatures.insert(to, removed);
	}

	/// The collision, if any, that would occur at `coords`.
	fn collision(&self, coords: TilePoint) -> Option<Collision> {
		let Some(tile) = self.terrain.get(&coords) else {
			return Some(Collision::OutOfBounds);
		};
		if let Tile::Wall = tile {
			return Some(Collision::Tile(*tile));
		}
		self.creatures
			.get(&coords)
			.map(|level_object| Collision::Object(level_object.clone()))
	}

	fn attack(&mut self, attacker: &mut Creature, defender: &mut Creature) {
		defender.health -= attacker.strength;
		println!(
			"A {:?} hit a {:?} for {:?} damage!",
			attacker.species, defender.species, attacker.strength
		);
		if defender.health <= 0 {
			println!("The {:?} died!", defender.species);
			self.creatures.remove(&defender.coords);
		}
	}
}
