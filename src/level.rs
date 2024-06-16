use std::{
	collections::{hash_map::Entry, HashMap, HashSet},
	hash::Hash,
};

use ggez::{
	graphics::{Canvas, Color, DrawParam},
	GameResult,
};
use rand::Rng;
use rand_pcg::Pcg32;

use crate::{
	coordinates::{
		ScreenPoint, ScreenRectangle, ScreenVector, TilePoint, TileRectangle,
		TileVector,
	},
	creature::{Creature, Species},
	meshes::Meshes,
	vision,
};

struct Layout {
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

	fn tile_layout(&self, coords: TilePoint) -> ScreenRectangle {
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
	) -> GameResult {
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
		Ok(())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id(u32);

enum Object {
	Creature(Creature),
}

struct LevelObject {
	id: Id,
	object: Object,
	coords: TilePoint,
}

impl LevelObject {
	fn draw(
		&self,
		canvas: &mut Canvas,
		meshes: &Meshes,
		layout: &Layout,
	) -> GameResult {
		let tile_layout = layout.tile_layout(self.coords);
		let mesh = match &self.object {
			Object::Creature(creature) => match creature.species {
				Species::Human => &meshes.human,
				Species::Goblin => &meshes.goblin,
			},
		};
		canvas.draw(
			mesh,
			DrawParam::new()
				.dest(tile_layout.pos + tile_layout.size / 2.0)
				.scale(tile_layout.size),
		);
		Ok(())
	}
}

#[derive(Debug)]
pub enum Collision {
	OutOfBounds,
	Tile(Tile),
	Object(Id),
}

pub struct Level {
	layout: Layout,
	terrain: HashMap<TilePoint, Tile>,
	objects_by_id: HashMap<Id, LevelObject>,
	object_ids_by_coords: HashMap<TilePoint, Id>,
	next_object_id: Id,
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
			objects_by_id: HashMap::new(),
			object_ids_by_coords: HashMap::new(),
			next_object_id: Id(0),
			vision: HashSet::new(),
			memory: HashMap::new(),
		};

		// Spawn some creatures.
		for room in rooms {
			level
				.spawn(
					Object::Creature(Creature {
						species: Species::Goblin,
						health: 3,
					}),
					room.center,
				)
				.unwrap();
		}

		level
	}

	/// Updates vision and memory according to the given player's field of view.
	pub fn update_vision(&mut self, player_id: Id) {
		let origin = self.objects_by_id[&player_id].coords;
		self.vision = vision::get_vision(origin, |coords: TilePoint| {
			!matches!(self.terrain.get(&coords), Some(Tile::Floor))
		});
		for coords in &self.vision {
			if let Some(tile) = self.terrain.get(coords) {
				self.memory.insert(*coords, *tile);
			}
		}
	}

	pub fn draw(&self, canvas: &mut Canvas, meshes: &Meshes) -> GameResult {
		// Draw all remembered tiles that are not currently visible.
		for (coords, tile) in &self.memory {
			if !self.vision.contains(coords) {
				tile.draw(
					canvas,
					meshes,
					&self.layout,
					*coords,
					Perception::Remembered,
				)?;
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
				)?;
			}
		}
		for object in self.objects_by_id.values() {
			if self.vision.contains(&object.coords) {
				object.draw(canvas, meshes, &self.layout)?;
			}
		}
		Ok(())
	}

	fn spawn(
		&mut self,
		object: Object,
		coords: TilePoint,
	) -> Result<Id, Collision> {
		if let Some(collision) = self.collide(coords) {
			return Err(collision);
		}
		let id = self.next_object_id;
		self.next_object_id.0 += 1;
		self.object_ids_by_coords.insert(coords, id);
		self.objects_by_id
			.insert(id, LevelObject { id, object, coords });
		Ok(id)
	}

	pub fn spawn_player(&mut self) -> Result<Id, Collision> {
		let player_coords = self
			.terrain
			.iter()
			.find_map(|(&coords, &tile)| {
				(tile == Tile::Floor
					&& !self.object_ids_by_coords.contains_key(&coords))
				.then_some(coords)
			})
			.unwrap();
		self.spawn(
			Object::Creature(Creature {
				species: Species::Human,
				health: 10,
			}),
			player_coords,
		)
	}

	/// Translate's the position of the object identified by `id` by the given
	/// `offset` if the tile at those coordinates is unoccupied. If the tile is
	/// occupied, this returns a collision.
	pub fn translate_object(
		&mut self,
		id: Id,
		offset: TileVector,
	) -> Result<(), Collision> {
		let level_object = &self.objects_by_id[&id];
		self.move_object_to(level_object.id, level_object.coords + offset)
	}

	/// Moves the object identified by `id` to `coords` if there's an
	/// unoccupied, passable tile there. If the tile is occupied, this returns a
	/// collision.
	fn move_object_to(
		&mut self,
		id: Id,
		coords: TilePoint,
	) -> Result<(), Collision> {
		if let Some(collision) = self.collide(coords) {
			return Err(collision);
		}
		let level_object = self.objects_by_id.get_mut(&id).unwrap();
		level_object.coords = coords;
		self.object_ids_by_coords.insert(coords, level_object.id);
		self.object_ids_by_coords.remove(&level_object.coords);
		Ok(())
	}

	/// Gets the collision, if any, that would occur at `coords`.
	fn collide(&self, coords: TilePoint) -> Option<Collision> {
		let Some(tile) = self.terrain.get(&coords) else {
			return Some(Collision::OutOfBounds);
		};
		if let Tile::Wall = tile {
			return Some(Collision::Tile(*tile));
		}
		self.object_ids_by_coords
			.get(&coords)
			.map(|id| Collision::Object(*id))
	}
}
