mod coordinates;

use std::{
	collections::{hash_map::Entry, HashMap},
	hash::Hash,
};

use coordinates::{
	ScreenPoint, ScreenRectangle, ScreenVector, TilePoint, TileRectangle,
	TileVector,
};
use macroquad::{
	color,
	input::{self, KeyCode},
	shapes, window,
};
use rand::prelude::*;
use rand_pcg::Pcg32;

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
	Floor,
	Wall,
}

impl Tile {
	fn draw(&self, layout: &Layout, coords: TilePoint) {
		let tile_layout = layout.tile_layout(coords);
		match self {
			Tile::Floor => {
				shapes::draw_rectangle(
					tile_layout.pos.x,
					tile_layout.pos.y,
					tile_layout.size.x,
					tile_layout.size.y,
					color::DARKGRAY,
				);
			}
			Tile::Wall => {
				shapes::draw_rectangle(
					tile_layout.pos.x,
					tile_layout.pos.y,
					tile_layout.size.x,
					tile_layout.size.y,
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
	coords: TilePoint,
}

impl LevelObject {
	fn draw(&self, layout: &Layout) {
		let tile_layout = layout.tile_layout(self.coords);
		match self.object {
			Object::Player => {
				shapes::draw_ellipse(
					tile_layout.pos.x + 0.5 * tile_layout.size.x,
					tile_layout.pos.y + 0.5 * tile_layout.size.y,
					0.5 * tile_layout.size.x,
					0.5 * tile_layout.size.y,
					0.0,
					color::YELLOW,
				);
			}
		}
	}
}

enum Collision {
	Tile(Tile),
	Object(Id),
}

struct Level {
	layout: Layout,
	terrain: HashMap<TilePoint, Tile>,
	objects_by_id: HashMap<Id, LevelObject>,
	object_ids_by_coords: HashMap<TilePoint, Id>,
	next_object_id: Id,
}

impl Level {
	fn generate(rng: &mut Pcg32) -> Level {
		struct Room {
			center: TilePoint,
			radius: TileVector,
		}

		let rooms = std::iter::from_fn(|| {
			Some(Room {
				center: TilePoint::new(
					rng.gen_range(0..30),
					rng.gen_range(0..30),
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
					terrain.insert(TilePoint::new(x, y), Tile::Floor);
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
		Level {
			layout: Layout::new(
				ScreenRectangle {
					pos: ScreenPoint::new(0.0, 0.0),
					size: ScreenVector::new(
						window::screen_width(),
						window::screen_height(),
					),
				},
				tileport,
			),
			terrain,
			objects_by_id: HashMap::new(),
			object_ids_by_coords: HashMap::new(),
			next_object_id: Id(0),
		}
	}

	fn draw(&self) {
		for (&coords, tile) in &self.terrain {
			tile.draw(&self.layout, coords);
		}
		for object in self.objects_by_id.values() {
			object.draw(&self.layout);
		}
	}

	fn spawn(&mut self, object: Object, coords: TilePoint) -> Id {
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
		offset: TileVector,
	) -> Option<Collision> {
		let level_object = &self.objects_by_id[&id];
		self.move_object_to(level_object.id, level_object.coords + offset)
	}

	// Moves the object identified by `id` to `coords` if there's an unoccupied,
	// passable tile there. If the tile is occupied, this returns a collision.
	fn move_object_to(
		&mut self,
		id: Id,
		coords: TilePoint,
	) -> Option<Collision> {
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

fn window_conf() -> window::Conf {
	window::Conf {
		window_title: "RL".to_string(),
		window_width: 1280,
		window_height: 720,
		fullscreen: false,
		window_resizable: true,
		// TODO: icon
		..Default::default()
	}
}

#[macroquad::main(window_conf)]
async fn main() {
	let mut rng: Pcg32 = Pcg32::from_entropy();
	let mut level = Level::generate(&mut rng);
	let player_coords = level
		.terrain
		.iter()
		.find_map(|(&coords, &tile)| (tile == Tile::Floor).then_some(coords))
		.unwrap();
	let player_id = level.spawn(Object::Player, player_coords);

	let mut fullscreen = false;
	loop {
		window::clear_background(color::BLACK);

		// Control fullscreen setting.
		if (input::is_key_down(KeyCode::LeftAlt)
			|| input::is_key_down(KeyCode::RightAlt))
			&& input::is_key_pressed(KeyCode::Enter)
		{
			fullscreen = !fullscreen;
			window::set_fullscreen(fullscreen);
		}
		if fullscreen && input::is_key_pressed(KeyCode::Escape) {
			fullscreen = false;
			window::set_fullscreen(false);
		}

		if input::is_key_pressed(KeyCode::Up) {
			level.translate_object(player_id, TileVector::new(0, -1));
		} else if input::is_key_pressed(KeyCode::Down) {
			level.translate_object(player_id, TileVector::new(0, 1));
		} else if input::is_key_pressed(KeyCode::Left) {
			level.translate_object(player_id, TileVector::new(-1, 0));
		} else if input::is_key_pressed(KeyCode::Right) {
			level.translate_object(player_id, TileVector::new(1, 0));
		}

		level.draw();

		window::next_frame().await
	}
}
