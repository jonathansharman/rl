/// A type of [`Creature`].
#[derive(Debug)]
pub enum Species {
	Human,
	Goblin,
}

/// An animate being.
#[derive(Debug)]
pub struct Creature {
	pub species: Species,
	pub health: i32,
	pub strength: i32,
}
