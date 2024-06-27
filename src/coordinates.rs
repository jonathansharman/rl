use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

use ggez::{
	glam::Vec2,
	mint::{Point2, Vector2},
};
use rand::Rng;
use rand_pcg::Pcg32;

pub type ScreenVector = Vector<f32>;
pub type ScreenPoint = Point<f32>;
pub type ScreenRectangle = Rectangle<f32>;

impl From<ScreenVector> for Vec2 {
	fn from(value: ScreenVector) -> Self {
		Vec2::new(value.x, value.y)
	}
}

impl From<ScreenPoint> for Vec2 {
	fn from(value: ScreenPoint) -> Self {
		Vec2::new(value.x, value.y)
	}
}

pub type TileVector = Vector<i32>;
pub type TilePoint = Point<i32>;
pub type TileRectangle = Rectangle<i32>;

pub const TILE_UP: TileVector = TileVector::new(0, -1);
pub const TILE_DOWN: TileVector = TileVector::new(0, 1);
pub const TILE_LEFT: TileVector = TileVector::new(-1, 0);
pub const TILE_RIGHT: TileVector = TileVector::new(1, 0);

/// Offset to a random adjacent tile.
pub fn random_neighbor(rng: &mut Pcg32) -> TileVector {
	match rng.gen_range(0..4) {
		0 => TILE_UP,
		1 => TILE_DOWN,
		2 => TILE_LEFT,
		_ => TILE_RIGHT,
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Vector<T> {
	pub x: T,
	pub y: T,
}

impl<T> Vector<T> {
	pub const fn new(x: T, y: T) -> Self {
		Vector { x, y }
	}
}

impl<T: AddAssign<T>> AddAssign for Vector<T> {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl<T: Add<Output = T>> Add for Vector<T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl<T: SubAssign<T>> SubAssign<Vector<T>> for Vector<T> {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl<T: Sub<Output = T>> Sub for Vector<T> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl<T: Clone + MulAssign<T>> MulAssign<T> for Vector<T> {
	fn mul_assign(&mut self, rhs: T) {
		self.x *= rhs.clone();
		self.y *= rhs;
	}
}

impl<T: Clone + Mul<Output = T>> Mul<T> for Vector<T> {
	type Output = Self;

	fn mul(self, rhs: T) -> Self {
		Self {
			x: self.x * rhs.clone(),
			y: self.y * rhs,
		}
	}
}

impl<T: Clone + DivAssign<T>> DivAssign<T> for Vector<T> {
	fn div_assign(&mut self, rhs: T) {
		self.x /= rhs.clone();
		self.y /= rhs;
	}
}

impl<T: Clone + Div<Output = T>> Div<T> for Vector<T> {
	type Output = Self;

	fn div(self, rhs: T) -> Self {
		Self {
			x: self.x / rhs.clone(),
			y: self.y / rhs,
		}
	}
}

impl<T> From<Vector<T>> for Vector2<T> {
	fn from(value: Vector<T>) -> Self {
		Self {
			x: value.x,
			y: value.y,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Point<T> {
	pub x: T,
	pub y: T,
}

impl<T> Point<T> {
	pub fn new(x: T, y: T) -> Self {
		Point { x, y }
	}
}

impl<T: AddAssign<T>> AddAssign<Vector<T>> for Point<T> {
	fn add_assign(&mut self, rhs: Vector<T>) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl<T: Add<Output = T>> Add<Vector<T>> for Point<T> {
	type Output = Self;

	fn add(self, rhs: Vector<T>) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl<T: SubAssign<T>> SubAssign<Vector<T>> for Point<T> {
	fn sub_assign(&mut self, rhs: Vector<T>) {
		self.x -= rhs.x;
		self.y -= rhs.y;
	}
}

impl<T: Sub<Output = T>> Sub<Vector<T>> for Point<T> {
	type Output = Self;

	fn sub(self, rhs: Vector<T>) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl<T: Sub<Output = T>> Sub for Point<T> {
	type Output = Vector<T>;

	fn sub(self, rhs: Self) -> Self::Output {
		Self::Output {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl<T> From<Point<T>> for Point2<T> {
	fn from(value: Point<T>) -> Self {
		Self {
			x: value.x,
			y: value.y,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rectangle<T> {
	pub pos: Point<T>,
	pub size: Vector<T>,
}
