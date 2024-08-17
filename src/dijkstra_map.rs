use std::collections::{hash_map::Entry, HashMap, VecDeque};

use rand::seq::IteratorRandom;
use rand_pcg::Pcg32;

use crate::geometry::{TilePoint, TileVector, NEIGHBOR_OFFSETS_FOUR};

/// Allows quickly pathfinding from any tile to the nearest tile of interest.
/// Based on [Dijkstra Maps Visualized][1] and [The Incredible Power of Dijkstra
/// Maps].
///
/// [1]: https://www.roguebasin.com/index.php/Dijkstra_Maps_Visualized
/// [2]: https://www.roguebasin.com/index.php/The_Incredible_Power_of_Dijkstra_Maps
#[derive(Debug)]
pub struct DijkstraMap {
	distances: HashMap<TilePoint, isize>,
}

impl DijkstraMap {
	/// Generates a Dijkstra map over `tiles` using `is_goal` to identify target
	/// tiles and `is_blocking` to identify blocking tiles.
	pub fn new(
		tiles: impl Iterator<Item = TilePoint>,
		is_goal: impl Fn(&TilePoint) -> bool,
		is_blocking: impl Fn(&TilePoint) -> bool,
	) -> DijkstraMap {
		let mut distances = HashMap::new();
		let mut queue = VecDeque::new();
		for coords in tiles.filter(is_goal) {
			distances.insert(coords, 0);
			queue.push_front((coords, 0));
		}
		while let Some((coords, distance)) = queue.pop_front() {
			for offset in NEIGHBOR_OFFSETS_FOUR {
				let neighbor = coords + offset;
				if is_blocking(&neighbor) {
					continue;
				}
				match distances.entry(neighbor) {
					Entry::Occupied(_) => {
						// This tile has already been visited.
						continue;
					}
					Entry::Vacant(entry) => {
						// Since this is a breadth-first search, we visit nodes
						// in ascending order of distance.
						let neighbor_distance = distance + 1;
						entry.insert(neighbor_distance);
						queue.push_back((neighbor, neighbor_distance));
					}
				}
			}
		}
		DijkstraMap { distances }
	}

	/// The distance from `coords` to the nearest tile of interest or `None` if
	/// there is no path from `coords` to a tile of interest.
	pub fn distance(&self, coords: TilePoint) -> Option<isize> {
		self.distances.get(&coords).copied()
	}

	/// Offset to a random neighbor of `coords` that is one tile closer to a
	/// tile of interest, if there is such a neighbor.
	pub fn step_towards(
		&self,
		coords: TilePoint,
		rng: &mut Pcg32,
	) -> Option<TileVector> {
		self.step(coords, rng, false)
	}

	/// Offset to a random neighbor of `coords` that is one tile farther away
	/// from a tile of interest, if there is such a neighbor.
	pub fn step_away(
		&self,
		coords: TilePoint,
		rng: &mut Pcg32,
	) -> Option<TileVector> {
		self.step(coords, rng, true)
	}

	/// Common implementation for `step_towards` and `step_away`.
	fn step(
		&self,
		coords: TilePoint,
		rng: &mut Pcg32,
		reverse: bool,
	) -> Option<TileVector> {
		let mut best_offsets = Vec::with_capacity(4);
		let mut best_distance = self.distance(coords).unwrap_or(if reverse {
			isize::MIN
		} else {
			isize::MAX
		});
		for offset in NEIGHBOR_OFFSETS_FOUR {
			if let Some(&distance) = self.distances.get(&(coords + offset)) {
				match distance.cmp(&best_distance) {
					std::cmp::Ordering::Less => {
						if !reverse {
							best_distance = distance;
							best_offsets = vec![offset];
						}
					}
					std::cmp::Ordering::Equal => best_offsets.push(offset),
					std::cmp::Ordering::Greater => {
						if reverse {
							best_distance = distance;
							best_offsets = vec![offset];
						}
					}
				}
			}
		}
		best_offsets.into_iter().choose(rng)
	}
}
