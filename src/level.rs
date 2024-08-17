use std::collections::{HashMap, HashSet};

use ggez::graphics::{Canvas, Color, DrawParam};
use rand::seq::SliceRandom;
use rand::Rng;
use rand_pcg::Pcg32;

use crate::{
	creature::{Behavior, Creature, Faction, Species},
	dijkstra_map::DijkstraMap,
	disjoint_sets::DisjointSets,
	geometry::{
		random_neighbor_offset_eight, RectangleIntersection, ScreenPoint,
		ScreenRectangle, ScreenVector, TileIntersection, TilePoint,
		TileRectangle, TileVector,
	},
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
	Grass,
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
			Tile::Floor(Floor::Grass) => &meshes.grass_floor,
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

/// A set of Dijkstra maps for points of interest within a [`Level`].
#[derive(Default)]
pub struct DijkstraMaps {
	pub enemies: HashMap<Faction, DijkstraMap>,
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
	dijkstra_maps: DijkstraMaps,
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
			let nudge = random_neighbor_offset_eight(rng);
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
			if let RectangleIntersection::Real(intersection) =
				new_room.floor.intersection(floor)
			{
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

		// Open the floor of each room.
		for room in rooms.iter() {
			let floor = if rng.gen() { Floor::Wood } else { Floor::Grass };
			for x in room.floor.pos.x..room.floor.pos.x + room.floor.size.x {
				for y in room.floor.pos.y..room.floor.pos.y + room.floor.size.y
				{
					make_floor(&mut terrain, TilePoint::new(x, y), floor);
				}
			}
		}

		// Build a forest of disjoint sets of connected rooms. Initially, each
		// room is in its own singleton set.
		let mut connected_rooms = DisjointSets::new(rooms.len());

		// Build an ordered queue of distances between rooms.
		struct Edge {
			i: usize,
			j: usize,
			intersection: TileIntersection,
		}
		let mut edges = Vec::new();
		for (i, room1) in rooms.iter().enumerate() {
			for (j, room2) in rooms.iter().enumerate().skip(i) {
				edges.push(Edge {
					i,
					j,
					intersection: room1.floor.intersection(room2.floor),
				});
			}
		}
		edges.sort_by(|e1, e2| {
			e2.intersection.distance().cmp(&e1.intersection.distance())
		});

		// Connect the closest two rooms until all rooms are connected.
		while let Some(Edge { i, j, intersection }) = edges.pop() {
			// Connect the rooms.
			let waypoints = match intersection {
				RectangleIntersection::Real(_) => {
					// At this point all rooms should be nonintersecting, so
					// this should be unreachable, but there wouldn't be any
					// work to do anyway.
					vec![]
				}
				RectangleIntersection::Horizontal(rect) => {
					// Connect vertically.
					let x = rng.gen_range(rect.pos.x..rect.pos.x + rect.size.x);
					vec![
						TilePoint::new(x, rect.pos.y),
						TilePoint::new(x, rect.pos.y + rect.size.y - 1),
					]
				}
				RectangleIntersection::Vertical(rect) => {
					// Connect horizontally.
					let y = rng.gen_range(rect.pos.y..rect.pos.y + rect.size.y);
					vec![
						TilePoint::new(rect.pos.x, y),
						TilePoint::new(rect.pos.x + rect.size.x - 1, y),
					]
				}
				RectangleIntersection::None(rect) => {
					// Expand the rectangle one tile up and left so that [pos,
					// pos + size] includes the rooms' corners.
					let rect = TileRectangle {
						pos: rect.pos - TileVector::new(1, 1),
						size: rect.size + TileVector::new(1, 1),
					};
					// Determine if the rooms are diagonal "/" or "\".
					let (room1, room2) = (&rooms[i], &rooms[j]);
					let (p1, p3) = if (room1.floor.pos.x <= room2.floor.pos.x)
						== (room1.floor.pos.y <= room2.floor.pos.y)
					{
						(rect.pos, rect.pos + rect.size)
					} else {
						let (mut p1, mut p3) = (rect.pos, rect.pos + rect.size);
						(p1.y, p3.y) = (p3.y, p1.y);
						(p1, p3)
					};
					// Connect the start and end via a random elbow.
					let p2 = if rng.gen() {
						TilePoint::new(p1.x, p3.y)
					} else {
						TilePoint::new(p3.x, p1.y)
					};
					vec![p1, p2, p3]
				}
			};
			for waypoints in waypoints.windows(2) {
				for coords in vision::line_between(waypoints[0], waypoints[1]) {
					make_floor(&mut terrain, coords, Floor::Stone);
				}
			}

			// Merge the two connection sets. Stop if all rooms are connected
			// and the most recently merged rooms were far enough apart.
			if connected_rooms.merge(i, j) == rooms.len()
				&& intersection.distance() > 2
			{
				break;
			}
		}

		let mut level = Level {
			tile_layout: TileLayout::new(config.viewport, config.tileport),
			terrain,
			creatures: HashMap::new(),
			items: HashMap::new(),
			vision: HashSet::new(),
			memory: HashMap::new(),
			dijkstra_maps: DijkstraMaps::default(),
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
				Faction::Enemy,
				species,
				Behavior::Patrolling,
				coords,
			)));
		}

		level
	}

	fn update_enemies_dijkstra_map(&mut self, faction: Faction) {
		self.dijkstra_maps.enemies.insert(
			faction,
			DijkstraMap::new(
				self.terrain.keys().copied(),
				|coords| {
					self.creatures.get(coords).is_some_and(|creature| {
						creature.borrow().faction != faction
					})
				},
				|coords| {
					self.collision(coords).is_some_and(|collision| {
						if let Collision::Object(other) = collision {
							// Creatures can't actually pass through allies, but
							// we'll act as though they can for the purpose of
							// pathfinding. This will allow enemies to pile up
							// at choke points when attempting to reach goals.
							other.borrow().faction != faction
						} else {
							// Hard collision.
							true
						}
					})
				},
			),
		);
	}

	/// Builds or rebuilds the level's Dijkstra maps.
	pub fn update_dijkstra_maps(&mut self) {
		self.update_enemies_dijkstra_map(Faction::Ally);
		self.update_enemies_dijkstra_map(Faction::Enemy);
	}

	/// A set of Dijkstra maps for points of interest within the level.
	pub fn dijkstra_maps(&self) -> &DijkstraMaps {
		&self.dijkstra_maps
	}

	/// Updates vision and memory using the given viewer `origin`.
	pub fn update_vision(&mut self, origin: TilePoint) {
		self.vision = vision::get_vision(origin, |coords: &TilePoint| {
			!matches!(self.terrain.get(coords), Some(Tile::Floor(_)))
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
			let mut creature = creature.borrow_mut();
			// The creature may have died during iteration.
			if creature.dead() {
				continue;
			}
			creature.act(self, rng);
		}
	}

	/// Spawns the player character at an arbitrary open tile. Panics if a spot
	/// can't be found.
	pub fn spawn_player(&mut self, rng: &mut Pcg32) -> Shared<Creature> {
		self.spawn(share(Creature::new(
			Faction::Ally,
			Species::Human,
			// The player's creature is controlled separately, so just idle
			// during level updates.
			Behavior::Idle,
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
		if from == to {
			return;
		}
		if let Some(collision) = self.collision(&to) {
			let Collision::Object(other) = collision else {
				return;
			};
			let other = &mut other.borrow_mut();
			if creature.faction != other.faction {
				self.attack(creature, other);
			}
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
			.filter(|coords| self.collision(coords).is_none())
			.collect()
	}

	fn spawn(
		&mut self,
		creature: Shared<Creature>,
	) -> Result<Shared<Creature>, Collision> {
		let coords = creature.borrow().coords;
		if let Some(collision) = self.collision(&coords) {
			return Err(collision);
		}
		self.creatures.insert(coords, creature.clone());
		Ok(creature)
	}

	/// The collision, if any, that would occur at `coords`.
	fn collision(&self, coords: &TilePoint) -> Option<Collision> {
		let Some(tile) = self.terrain.get(coords) else {
			return Some(Collision::OutOfBounds);
		};
		if let Tile::Wall = tile {
			return Some(Collision::Tile(*tile));
		}
		self.creatures
			.get(coords)
			.map(|level_object| Collision::Object(level_object.clone()))
	}

	fn attack(&mut self, attacker: &mut Creature, defender: &mut Creature) {
		defender.take_damage(attacker.stats.strength);
		println!(
			"A {:?} hit a {:?} for {:?} damage!",
			attacker.species, defender.species, attacker.stats.strength
		);
		if defender.dead() {
			println!("The {:?} died!", defender.species);
			self.creatures.remove(&defender.coords);
		}
	}
}
