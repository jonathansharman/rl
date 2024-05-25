use std::ops::{Add, AddAssign, Sub, SubAssign};

pub struct Vector {
	pub x: i32,
	pub y: i32,
}

impl Vector {
	pub fn new(x: i32, y: i32) -> Vector {
		Vector { x, y }
	}
}

impl AddAssign for Vector {
	fn add_assign(&mut self, rhs: Vector) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl Add for Vector {
	type Output = Vector;

	fn add(mut self, rhs: Self) -> Vector {
		self += rhs;
		self
	}
}

impl SubAssign<Vector> for Vector {
	fn sub_assign(&mut self, rhs: Vector) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl Sub for Vector {
	type Output = Vector;

	fn sub(mut self, rhs: Self) -> Vector {
		self -= rhs;
		self
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
	pub x: i32,
	pub y: i32,
}

impl Point {
	pub fn new(x: i32, y: i32) -> Point {
		Point { x, y }
	}
}

impl AddAssign<Vector> for Point {
	fn add_assign(&mut self, rhs: Vector) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl Add<Vector> for Point {
	type Output = Point;

	fn add(mut self, rhs: Vector) -> Point {
		self += rhs;
		self
	}
}

impl SubAssign<Vector> for Point {
	fn sub_assign(&mut self, rhs: Vector) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl Sub<Vector> for Point {
	type Output = Point;

	fn sub(mut self, rhs: Vector) -> Point {
		self -= rhs;
		self
	}
}
