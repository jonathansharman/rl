use std::{
	cmp::Ordering,
	collections::{BTreeSet, HashMap, HashSet},
};

use ggez::graphics::{Canvas, Color, DrawParam};
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg32;

use crate::{
	coordinates::{
		random_neighbor_eight, random_neighbor_four, ScreenPoint,
		ScreenRectangle, ScreenVector, TilePoint, TileRectangle, TileVector,
	},
	creature::{Behavior, Creature, Species},
	item::Item,
	meshes::Meshes,
	shared::{share, Shared},
	vision,
};

/// Maps a region in tile space (the tileport) to a region in screen space (the
/// viewport), filling the viewport while maintaining the tileport's original
/// aspect ratio, i.e. ensuring tiles appear square.
pub struct TileLayout {
	// The region of the screen to map this layout to.
	viewport: ScreenRectangle,
	// Tile rectangle containing all the tiles that may need to be displayed.
	tileport: TileRectangle,
	// Tile width and height on-screen.
	tile_size: ScreenVector,
}

impl TileLayout {
	fn new(viewport: ScreenRectangle, tileport: TileRectangle) -> TileLayout {
		// Shrink the viewport as needed so that its aspect ratio matches the
		// tileport's.
		let tileport_ar = tileport.size.x as f32 / tileport.size.y as f32;
		let viewport_ar = viewport.size.x / viewport.size.y;
		let viewport = if viewport_ar <= tileport_ar {
			// The viewport is possibly too tall.
			let new_height = viewport.size.x / tileport_ar;
			ScreenRectangle {
				pos: ScreenPoint::new(
					viewport.pos.x,
					viewport.pos.y + 0.5 * (viewport.size.y - new_height),
				),
				size: ScreenVector::new(viewport.size.x, new_height),
			}
		} else {
			// The viewport is too wide.
			let new_width = viewport.size.y * tileport_ar;
			ScreenRectangle {
				pos: ScreenPoint::new(
					viewport.pos.x + 0.5 * (viewport.size.x - new_width),
					viewport.pos.y,
				), // TODO: Center
				size: ScreenVector::new(new_width, viewport.size.y),
			}
		};
		let tile_size = ScreenVector::new(
			viewport.size.x / tileport.size.x as f32,
			viewport.size.y / tileport.size.y as f32,
		);
		TileLayout {
			viewport,
			tileport,
			tile_size,
		}
	}

	pub fn to_screen(&self, coords: TilePoint) -> ScreenRectangle {
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
pub enum Floor {
	Stone,
	Wood,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
	Floor(Floor),
	Wall,
}

impl Tile {
	fn draw(
		&self,
		canvas: &mut Canvas,
		meshes: &Meshes,
		tile_layout: &TileLayout,
		coords: TilePoint,
		perception: Perception,
	) {
		let color = match perception {
			Perception::Seen => Color::WHITE,
			Perception::Remembered => Color::from_rgba(255, 255, 255, 64),
		};
		let screen_tile = tile_layout.to_screen(coords);
		let mesh = match self {
			Tile::Floor(Floor::Stone) => &meshes.stone_floor,
			Tile::Floor(Floor::Wood) => &meshes.wood_floor,
			Tile::Wall => &meshes.wall,
		};
		canvas.draw(
			mesh,
			DrawParam::new()
				.dest(screen_tile.pos)
				.scale(screen_tile.size)
				.color(color),
		);
	}
}

#[derive(Debug)]
pub enum Collision {
	OutOfBounds,
	Tile(Tile),
	Object(Shared<Creature>),
}

pub struct Level {
	tile_layout: TileLayout,
	terrain: HashMap<TilePoint, Tile>,
	creatures: HashMap<TilePoint, Shared<Creature>>,
	items: HashMap<TilePoint, Shared<Item>>,
	/// Points the player can currently see.
	vision: HashSet<TilePoint>,
	/// Tiles the player remembers seeing.
	memory: HashMap<TilePoint, Tile>,
}

/// Configuration settings for level generation.
pub struct GenerationConfig {
	/// The region in screen space the level should cover.
	pub viewport: ScreenRectangle,
	/// The region in tile space the level should cover.
	pub tileport: TileRectangle,
	/// The minimum allowable proportion of all tiles within `tileport` to be
	/// marked as floors. Additional rooms will be added until this proportion
	/// is reached (up to a retry limit, in case additional rooms can't fit).
	pub min_floor_ratio: f32,
	/// Minimum width of a room's floor.
	pub min_room_size: i32,
	/// Maximum length of a room's floor.
	pub max_room_size: i32,
}

struct Room {
	floor: TileRectangle,
}

impl Room {
	fn center(&self) -> TilePoint {
		self.floor.pos + self.floor.size / 2
	}
}

#[derive(PartialEq, Eq)]
struct Neighbor {
	index: usize,
	distance: i32,
}

impl Ord for Neighbor {
	fn cmp(&self, other: &Self) -> Ordering {
		self.distance
			.cmp(&other.distance)
			.then(self.index.cmp(&other.index))
	}
}

impl PartialOrd for Neighbor {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

const MAX_ROOM_PLACEMENT_RETRIES: u32 = 100;

impl Level {
	pub fn generate(config: GenerationConfig, rng: &mut Pcg32) -> Level {
		// Leave a one-tile border around the floor for outer walls.
		let floor = TileRectangle {
			pos: TilePoint {
				x: config.tileport.pos.x + 1,
				y: config.tileport.pos.y + 1,
			},
			size: TileVector::new(
				config.tileport.size.x - 2,
				config.tileport.size.y - 2,
			),
		};

		let mut rooms: Vec<Room> = Vec::new();

		// Add rooms until the target floor coverage is reached.
		let total_area = floor.area();
		let mut floor_area = 0;
		let mut retries = 0;
		while (floor_area as f32 / total_area as f32) < config.min_floor_ratio {
			let mut new_room = {
				let size = TileVector::new(
					rng.gen_range(config.min_room_size..=config.max_room_size),
					rng.gen_range(config.min_room_size..=config.max_room_size),
				);
				Room {
					floor: TileRectangle {
						pos: TilePoint::new(
							floor.pos.x
								+ rng.gen_range(0..=floor.size.x - size.x),
							floor.pos.y
								+ rng.gen_range(0..=floor.size.y - size.y),
						),
						size,
					},
				}
			};

			// Nudge the room while it touches any existing rooms. (This uses an
			// inefficient O(n^2) collision algorithm, but it should be good
			// enough for the number of rooms we're dealing with.)
			let nudge = random_neighbor_eight(rng);
			let mut nudging = true;
			while nudging {
				nudging = false;
				for room in rooms.iter() {
					if new_room.floor.touching(room.floor) {
						nudging = true;
						new_room.floor.pos += nudge;
					}
				}
			}

			// Crop the room to fit within the tileport.
			if let Some(intersection) = new_room.floor.intersection(floor) {
				new_room.floor = intersection;
			} else {
				// If the room is completely outside the level, set its size to
				// zero so it will be discarded.
				new_room.floor.size = TileVector::new(0, 0);
			}

			// If the room is now too small, discard it and try again.
			if new_room.floor.size.x < config.min_room_size
				|| new_room.floor.size.y < config.min_room_size
			{
				retries += 1;
				if retries > MAX_ROOM_PLACEMENT_RETRIES {
					break;
				} else {
					continue;
				}
			}

			retries = 0;
			floor_area += new_room.floor.area();
			rooms.push(new_room);
		}

		let mut terrain = HashMap::new();
		let make_floor = |terrain: &mut HashMap<TilePoint, Tile>,
		                  coords: TilePoint,
		                  floor: Floor| {
			for x in coords.x - 1..=coords.x + 1 {
				for y in coords.y - 1..=coords.y + 1 {
					if x == coords.x && y == coords.y {
						terrain.insert(coords, Tile::Floor(floor));
					} else {
						terrain
							.entry(TilePoint::new(x, y))
							.or_insert(Tile::Wall);
					}
				}
			}
		};

		// TODO: This room connection algorithm (besides being inefficient) does
		// not produce very good results and should be replaced (see
		// docs/level-generation.md).

		// Connect rooms via hallways.
		let mut connected = HashSet::from([0]);
		let mut unconnected = Vec::from_iter(0..rooms.len());
		while let Some(i) = unconnected.pop() {
			let room = &rooms[i];

			// Connect this room to 1-3 of its nearest neighbors in the
			// connected set. (This uses a very inefficient KNN algorithm, but
			// again it suffices for now.)
			let mut neighbors = BTreeSet::new();
			for &j in connected.iter() {
				let offset = room.center() - rooms[j].center();
				let distance = offset.x.abs() + offset.y.abs();
				neighbors.insert(Neighbor { index: j, distance });
			}
			for neighbor in neighbors.iter().take(rng.gen_range(1..=3)) {
				let neighbor = &rooms[neighbor.index];
				let mut coords = room.center();
				while coords.x < neighbor.center().x {
					make_floor(&mut terrain, coords, Floor::Stone);
					coords.x += 1;
				}
				while coords.x > neighbor.center().x {
					make_floor(&mut terrain, coords, Floor::Stone);
					coords.x -= 1;
				}
				while coords.y < neighbor.center().y {
					make_floor(&mut terrain, coords, Floor::Stone);
					coords.y += 1;
				}
				while coords.y > neighbor.center().y {
					make_floor(&mut terrain, coords, Floor::Stone);
					coords.y -= 1;
				}
			}

			connected.insert(i);
		}

		// Open the floor of each room.
		for room in rooms.iter() {
			for x in room.floor.pos.x..room.floor.pos.x + room.floor.size.x {
				for y in room.floor.pos.y..room.floor.pos.y + room.floor.size.y
				{
					make_floor(&mut terrain, TilePoint::new(x, y), Floor::Wood);
				}
			}
		}

		let mut level = Level {
			tile_layout: TileLayout::new(config.viewport, config.tileport),
			terrain,
			creatures: HashMap::new(),
			items: HashMap::new(),
			vision: HashSet::new(),
			memory: HashMap::new(),
		};

		// Spawn creatures.
		let mut unoccupied_coords = level.unoccupied_coords();
		unoccupied_coords.shuffle(rng);
		// TODO: Configure spawning in GenerationConfig.
		for coords in unoccupied_coords.into_iter().take(10) {
			let species = if rng.gen_range(0.0..1.0) < 0.15 {
				Species::Ogre
			} else {
				Species::Goblin
			};
			// Ignore failure to spawn.
			let _ = level.spawn(share(Creature::new(
				species,
				Behavior::AIControlled,
				coords,
			)));
		}

		level
	}

	/// Updates vision and memory using the given viewer `origin`.
	pub fn update_vision(&mut self, origin: TilePoint) {
		self.vision = vision::get_vision(origin, |coords: TilePoint| {
			!matches!(self.terrain.get(&coords), Some(Tile::Floor(_)))
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
					&self.tile_layout,
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
					&self.tile_layout,
					*coords,
					Perception::Seen,
				);
			}
		}
		for creature in self.creatures.values() {
			let creature = creature.borrow();
			if self.vision.contains(&creature.coords) {
				creature.draw(canvas, meshes, &self.tile_layout);
			}
		}
	}

	/// Advance time in the level by one turn, allowing NPCs to take their
	/// turns.
	pub fn update(&mut self, rng: &mut Pcg32) {
		let mut queue = self.creatures.values().cloned().collect::<Vec<_>>();
		while let Some(creature) = queue.pop() {
			// The player's creature is controlled separately.
			if let Behavior::PlayerControlled = creature.borrow().behavior {
				continue;
			}
			// The creature may have died during iteration.
			if creature.borrow().dead() {
				continue;
			}
			// Move in a random direction.
			self.translate_creature(
				&mut creature.borrow_mut(),
				random_neighbor_four(rng),
			);
		}
	}

	/// Spawns the player character at an arbitrary open tile. Panics if a spot
	/// can't be found.
	pub fn spawn_player(&mut self, rng: &mut Pcg32) -> Shared<Creature> {
		self.spawn(share(Creature::new(
			Species::Human,
			Behavior::PlayerControlled,
			*self.unoccupied_coords().choose(rng).unwrap(),
		)))
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

	/// All floor tile coordinates not occupied by a creature.
	fn unoccupied_coords(&self) -> Vec<TilePoint> {
		self.terrain
			.keys()
			.copied()
			.filter(|&coords| self.collision(coords).is_none())
			.collect()
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
		defender.take_damage(attacker.strength);
		println!(
			"A {:?} hit a {:?} for {:?} damage!",
			attacker.species, defender.species, attacker.strength
		);
		if defender.dead() {
			println!("The {:?} died!", defender.species);
			self.creatures.remove(&defender.coords);
		}
	}
}
