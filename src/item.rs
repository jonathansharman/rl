use ggez::graphics::{Canvas, DrawParam};

use crate::{coordinates::TilePoint, level::Layout, meshes::Meshes};

#[derive(Debug)]
pub struct Item {
	coords: TilePoint,
}

impl Item {
	pub fn draw(&self, canvas: &mut Canvas, meshes: &Meshes, layout: &Layout) {
		let tile_layout = layout.tile_layout(self.coords);
		let mesh = &meshes.item;
		canvas.draw(
			mesh,
			DrawParam::new()
				.dest(tile_layout.pos + tile_layout.size / 2.0)
				.scale(tile_layout.size),
		);
	}
}
