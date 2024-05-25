mod coordinates;

use std::{
	collections::{hash_map::Entry, HashMap},
	hash::Hash,
};

use coordinates::{Point, Vector};
use macroquad::{
	color,
	input::{self, KeyCode},
	shapes, window,
};
use rand::prelude::*;
use rand_pcg::Pcg32;

const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
	Floor,
	Wall,
}

impl Tile {
	fn draw(&self, coords: Point) {
		let x = (TILE_SIZE + 1.0) * coords.x as f32;
		let y = (TILE_SIZE + 1.0) * coords.y as f32;
		match self {
			Tile::Floor => {
				shapes::draw_rectangle(
					x,
					y,
					TILE_SIZE,
					TILE_SIZE,
					color::DARKGRAY,
				);
			}
			Tile::Wall => {
				shapes::draw_rectangle(
					x,
					y,
					TILE_SIZE,
					TILE_SIZE,
					color::MAROON,
				);
			}
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Id(u32);

enum Object {
	Player,
}

struct LevelObject {
	id: Id,
	object: Object,
	coords: Point,
}

impl LevelObject {
	fn draw(&self) {
		let x = (TILE_SIZE + 1.0) * self.coords.x as f32 + (0.5 * TILE_SIZE);
		let y = (TILE_SIZE + 1.0) * self.coords.y as f32 + (0.5 * TILE_SIZE);
		match self.object {
			Object::Player => {
				shapes::draw_circle(x, y, 0.5 * TILE_SIZE, color::YELLOW);
			}
		}
	}
}

enum Collision {
	Tile(Tile),
	Object(Id),
}

struct Level {
	terrain: HashMap<Point, Tile>,
	objects_by_id: HashMap<Id, LevelObject>,
	object_ids_by_coords: HashMap<Point, Id>,
	next_object_id: Id,
}

impl Level {
	fn generate(rng: &mut Pcg32) -> Level {
		struct Room {
			center: Point,
			radius: Vector,
		}

		let rooms = std::iter::from_fn(|| {
			Some(Room {
				center: Point::new(rng.gen_range(0..30), rng.gen_range(0..30)),
				radius: Vector::new(rng.gen_range(3..5), rng.gen_range(3..5)),
			})
		})
		.take(5)
		.collect::<Vec<_>>();

		let mut terrain = HashMap::new();
		// Connect each room to each other.
		for (i, room1) in rooms.iter().enumerate() {
			for room2 in rooms.iter().skip(i) {
				let mut coords = room1.center;
				while coords.x < room2.center.x {
					terrain.insert(coords, Tile::Floor);
					coords.x += 1;
				}
				while coords.x > room2.center.x {
					terrain.insert(coords, Tile::Floor);
					coords.x -= 1;
				}
				while coords.y < room2.center.y {
					terrain.insert(coords, Tile::Floor);
					coords.y += 1;
				}
				while coords.y > room2.center.y {
					terrain.insert(coords, Tile::Floor);
					coords.y -= 1;
				}
			}
		}
		// Open the floor of each room.
		for room in rooms {
			let x_min = room.center.x - room.radius.x;
			let x_max = room.center.x + room.radius.x;
			let y_min = room.center.y - room.radius.y;
			let y_max = room.center.y + room.radius.y;
			for x in x_min..=x_max {
				for y in y_min..=y_max {
					terrain.insert(Point::new(x, y), Tile::Floor);
				}
			}
		}

		Level {
			terrain,
			objects_by_id: HashMap::new(),
			object_ids_by_coords: HashMap::new(),
			next_object_id: Id(0),
		}
	}

	fn draw(&self) {
		for (&coords, tile) in &self.terrain {
			tile.draw(coords);
		}
		for object in self.objects_by_id.values() {
			object.draw();
		}
	}

	fn spawn(&mut self, object: Object, coords: Point) -> Id {
		let id = self.next_object_id;
		self.next_object_id.0 += 1;
		self.objects_by_id
			.insert(id, LevelObject { id, object, coords });
		self.object_ids_by_coords.insert(coords, id);
		id
	}

	// Translate's the position of the object identified by `id` by the given
	// `offset` if the tile at those coordinates is unoccupied. If the tile is
	// occupied, this returns a collision.
	fn translate_object(
		&mut self,
		id: Id,
		offset: Vector,
	) -> Option<Collision> {
		let level_object = &self.objects_by_id[&id];
		self.move_object_to(level_object.id, level_object.coords + offset)
	}

	// Moves the object identified by `id` to `coords` if there's an unoccupied,
	// passable tile there. If the tile is occupied, this returns a collision.
	fn move_object_to(&mut self, id: Id, coords: Point) -> Option<Collision> {
		let tile = self.terrain.get(&coords)?;
		if let Tile::Wall = tile {
			return Some(Collision::Tile(*tile));
		}

		let level_object = self.objects_by_id.get_mut(&id).unwrap();
		match self.object_ids_by_coords.entry(coords) {
			Entry::Occupied(entry) => Some(Collision::Object(*entry.get())),
			Entry::Vacant(entry) => {
				entry.insert(level_object.id);
				self.object_ids_by_coords.remove(&level_object.coords);
				level_object.coords = coords;
				None
			}
		}
	}
}

#[macroquad::main("RL")]
async fn main() {
	let mut rng: Pcg32 = Pcg32::from_entropy();
	let mut level = Level::generate(&mut rng);
	let player_coords = level
		.terrain
		.iter()
		.find_map(|(&coords, &tile)| (tile == Tile::Floor).then_some(coords))
		.unwrap();
	let player_id = level.spawn(Object::Player, player_coords);

	loop {
		window::clear_background(color::BLACK);

		if input::is_key_pressed(KeyCode::Up) {
			level.translate_object(player_id, Vector::new(0, -1));
		} else if input::is_key_pressed(KeyCode::Down) {
			level.translate_object(player_id, Vector::new(0, 1));
		} else if input::is_key_pressed(KeyCode::Left) {
			level.translate_object(player_id, Vector::new(-1, 0));
		} else if input::is_key_pressed(KeyCode::Right) {
			level.translate_object(player_id, Vector::new(1, 0));
		}

		level.draw();

		window::next_frame().await
	}
}
