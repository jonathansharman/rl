/// A forest of disjoint sets.
pub struct DisjointSets {
	nodes: Vec<Node>,
}

impl DisjointSets {
	/// Initializes a forest of disjoint sets with `len` singleton sets.
	pub fn new(len: usize) -> DisjointSets {
		DisjointSets {
			nodes: (0..len).map(|i| Node { parent: i, size: 1 }).collect(),
		}
	}

	/// Finds the representative element for the set containing element `i`.
	pub fn find(&mut self, i: usize) -> usize {
		if self.nodes[i].parent == i {
			i
		} else {
			self.nodes[i].parent = self.find(self.nodes[i].parent);
			self.nodes[i].parent
		}
	}

	/// Merges the sets containing elements `i` and `j`, returning the size of
	/// the merged set.
	pub fn merge(&mut self, i: usize, j: usize) -> usize {
		let mut i = self.find(i);
		let mut j = self.find(j);

		if i != j {
			// Ensure i has no fewer descendants than j.
			if self.nodes[i].size < self.nodes[j].size {
				(i, j) = (j, i);
			}
			// Set i as j's new parent.
			self.nodes[j].parent = i;
			// Set i's size to the sum of the two sets' sizes.
			self.nodes[i].size += self.nodes[j].size;
		}

		self.nodes[i].size
	}
}

struct Node {
	parent: usize,
	size: usize,
}
