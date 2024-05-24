use std::{
	collections::{hash_map::Entry, HashMap},
	hash::Hash,
};

use macroquad::{input, prelude::*};

const TILE_SIZE: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Coords {
	row: i32,
	col: i32,
}

impl Coords {
	fn new(x: i32, y: i32) -> Coords {
		Coords { row: x, col: y }
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
	Floor,
	Wall,
}

impl Tile {
	fn draw(&self, coords: Coords) {
		let x = (TILE_SIZE + 1.0) * coords.col as f32;
		let y = (TILE_SIZE + 1.0) * coords.row as f32;
		match self {
			Tile::Floor => {
				draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, DARKGRAY);
			}
			Tile::Wall => {
				draw_rectangle(x, y, TILE_SIZE, TILE_SIZE, MAROON);
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
	coords: Coords,
}

impl LevelObject {
	fn draw(&self) {
		let x = (TILE_SIZE + 1.0) * self.coords.col as f32 + (0.5 * TILE_SIZE);
		let y = (TILE_SIZE + 1.0) * self.coords.row as f32 + (0.5 * TILE_SIZE);
		match self.object {
			Object::Player => draw_circle(x, y, 0.5 * TILE_SIZE, YELLOW),
		}
	}
}

enum Collision {
	Tile(Tile),
	Object(Id),
}

struct Level {
	terrain: HashMap<Coords, Tile>,
	objects_by_id: HashMap<Id, LevelObject>,
	object_ids_by_coords: HashMap<Coords, Id>,
	next_object_id: Id,
}

impl Level {
	fn generate() -> Level {
		let mut terrain = HashMap::new();
		for row in 0..5 {
			for col in 0..5 {
				terrain.insert(Coords::new(row, col), Tile::Floor);
			}
		}
		terrain.insert(Coords::new(3, 3), Tile::Wall);
		Level {
			terrain,
			objects_by_id: HashMap::new(),
			object_ids_by_coords: HashMap::new(),
			next_object_id: Id(0),
		}
	}

	fn draw(&self) {
		for (&coords, tile) in self.terrain.iter() {
			tile.draw(coords);
		}
		for object in self.objects_by_id.values() {
			object.draw();
		}
	}

	fn spawn(&mut self, object: Object, coords: Coords) -> Id {
		let id = self.next_object_id;
		self.next_object_id.0 += 1;
		self.objects_by_id
			.insert(id, LevelObject { id, object, coords });
		self.object_ids_by_coords.insert(coords, id);
		id
	}

	// Translate's the position of the object identified by `id` by the given
	// `row_offset` and `col_offset` if the tile at those coordinates is
	// unoccupied. If the tile is occupied, this returns a collision.
	fn translate_object(
		&mut self,
		id: Id,
		row_offset: i32,
		col_offset: i32,
	) -> Option<Collision> {
		let level_object = &self.objects_by_id[&id];
		self.move_object_to(
			level_object.id,
			Coords {
				row: level_object.coords.row + row_offset,
				col: level_object.coords.col + col_offset,
			},
		)
	}

	// Moves the object identified by `id` to `coords` if there's an unoccupied,
	// passable tile there. If the tile is occupied, this returns a collision.
	fn move_object_to(&mut self, id: Id, coords: Coords) -> Option<Collision> {
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
	let mut level = Level::generate();
	let player_id = level.spawn(Object::Player, Coords::new(2, 2));

	loop {
		clear_background(BLACK);

		if input::is_key_pressed(KeyCode::Up) {
			level.translate_object(player_id, -1, 0);
		} else if input::is_key_pressed(KeyCode::Down) {
			level.translate_object(player_id, 1, 0);
		} else if input::is_key_pressed(KeyCode::Left) {
			level.translate_object(player_id, 0, -1);
		} else if input::is_key_pressed(KeyCode::Right) {
			level.translate_object(player_id, 0, 1);
		}

		level.draw();

		next_frame().await
	}
}
