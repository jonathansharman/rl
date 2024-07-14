use ggez::graphics::{Canvas, DrawParam};

use crate::{coordinates::TilePoint, level::TileLayout, meshes::Meshes};

#[derive(Debug)]
pub struct Item {
	coords: TilePoint,
}

impl Item {
	pub fn draw(
		&self,
		canvas: &mut Canvas,
		meshes: &Meshes,
		tile_layout: &TileLayout,
	) {
		let screen_tile = tile_layout.to_screen(self.coords);
		let mesh = &meshes.item;
		canvas.draw(
			mesh,
			DrawParam::new()
				.dest(screen_tile.pos + screen_tile.size / 2.0)
				.scale(screen_tile.size),
		);
	}
}
