use std::collections::HashSet;

use num_rational::Rational32;

use crate::coordinates::TilePoint;

/// Computes the set of tile coordinates visible from the given `origin`,
/// blocked by any tiles where `is_blocking` returns true.
///
/// This function is adapted from https://www.albertford.com/shadowcasting/,
/// which implements symmetric shadowcasting with diamond-shaped walls.
pub fn get_vision(
	origin: TilePoint,
	is_blocking: impl Fn(TilePoint) -> bool,
) -> HashSet<TilePoint> {
	let mut vision = HashSet::from([origin]);

	for quadrant in [
		Quadrant::North,
		Quadrant::South,
		Quadrant::East,
		Quadrant::West,
	] {
		let is_wall = |coords: Option<QuadrantPoint>| -> bool {
			coords.is_some_and(|coords| {
				is_blocking(quadrant.transform(origin, coords))
			})
		};

		let is_floor = |coords: Option<QuadrantPoint>| {
			coords.is_some_and(|coords| {
				!is_blocking(quadrant.transform(origin, coords))
			})
		};

		// Iteratively process the current quadrant by row sections, starting
		// with the entire arc of the first row.
		let mut queue = vec![QuadrantRow {
			distance: 1,
			start_slope: Rational32::from(-1),
			end_slope: Rational32::from(1),
		}];
		while let Some(mut row) = queue.pop() {
			let mut prev_tile = None;

			// A tile is considered to be in a row if "the sector swept out by
			// the row's start and end slopes overlaps with a diamond inscribed
			// in the tile."
			let min_column = round_ties_up(row.start_slope * row.distance);
			let max_column = round_ties_down(row.end_slope * row.distance);
			for column in min_column..=max_column {
				let coords = QuadrantPoint {
					distance: row.distance,
					column,
				};
				// Reveal walls unconditionally and non-walls if they have
				// symmetric vision with the origin. Including walls even if
				// vision is not symmetric provides "expansive walls", where
				// every wall in a convex room is visible when standing in that
				// room.
				if is_wall(Some(coords)) || row.contains_center(coords) {
					vision.insert(quadrant.transform(origin, coords));
				}
				// If we hit a wall, split the current row into (at most) two
				// sections: one before and one after the wall.
				if is_floor(prev_tile) && is_wall(Some(coords)) {
					// Iterate on the row section up to the wall.
					queue.push(QuadrantRow {
						distance: row.distance + 1,
						end_slope: coords.wall_tangent_slope(),
						..row
					});
				}
				if is_wall(prev_tile) && is_floor(Some(coords)) {
					// This means we've scanned to the other side of a wall.
					// Move the start slope of the current row to just beyond
					// the end of the wall and keep going.
					row.start_slope = coords.wall_tangent_slope();
				}
				prev_tile = Some(coords);
			}
			if is_floor(prev_tile) {
				queue.push(QuadrantRow {
					distance: row.distance + 1,
					..row
				});
			}
		}
	}

	vision
}

enum Quadrant {
	North,
	South,
	East,
	West,
}

impl Quadrant {
	/// Transforms `coords` relative to this quadrant and the given `origin` to
	/// tile coordinates relative to the entire level.
	fn transform(&self, origin: TilePoint, coords: QuadrantPoint) -> TilePoint {
		match self {
			Quadrant::North => TilePoint::new(
				origin.x - coords.column,
				origin.y - coords.distance,
			),
			Quadrant::South => TilePoint::new(
				origin.x + coords.column,
				origin.y + coords.distance,
			),
			Quadrant::East => TilePoint::new(
				origin.x + coords.distance,
				origin.y - coords.column,
			),
			Quadrant::West => TilePoint::new(
				origin.x - coords.distance,
				origin.y + coords.column,
			),
		}
	}
}

/// Tile coordinates relative to a [`Quadrant`].
#[derive(Clone, Copy)]
struct QuadrantPoint {
	/// Distance from the quadrant origin.
	distance: i32,
	/// Offset within a [`QuadrantRow`].
	column: i32,
}

impl QuadrantPoint {
	/// The slope from the origin of the quadrant to the "far" tangent line of a
	/// diamond-shaped wall at `self`.
	fn wall_tangent_slope(&self) -> Rational32 {
		Rational32::new(2 * self.column - 1, 2 * self.distance)
	}
}

/// A section of a row in a [`Quadrant`].
struct QuadrantRow {
	/// Distance from the quadrant origin.
	distance: i32,
	start_slope: Rational32,
	end_slope: Rational32,
}

impl QuadrantRow {
	/// Whether the center of the tile at `coords` is within `self`.
	fn contains_center(&self, coords: QuadrantPoint) -> bool {
		(self.start_slope * self.distance..=self.end_slope * self.distance)
			.contains(&coords.column.into())
	}
}

/// Rounds [x, x + 0.5) to x and [x + 0.5, x + 1) to x + 1.
fn round_ties_up(n: Rational32) -> i32 {
	(n + Rational32::new(1, 2)).floor().to_integer()
}

/// Rounds [x, x + 0.5] to x and (x + 0.5, x + 1) to x + 1.
fn round_ties_down(n: Rational32) -> i32 {
	(n - Rational32::new(1, 2)).ceil().to_integer()
}
