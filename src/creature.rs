#[derive(Debug)]
pub enum Species {
	Human,
	Goblin,
}

#[derive(Debug)]
pub struct Creature {
	pub species: Species,
	pub health: i32,
}
