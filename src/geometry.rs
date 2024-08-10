use std::ops::{
	Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

use ggez::{
	glam::Vec2,
	mint::{Point2, Vector2},
};
use num_traits::Zero;
use rand::seq::SliceRandom;
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
pub type TileIntersection = RectangleIntersection<i32>;

pub const TILE_UP: TileVector = TileVector::new(0, -1);
pub const TILE_DOWN: TileVector = TileVector::new(0, 1);
pub const TILE_LEFT: TileVector = TileVector::new(-1, 0);
pub const TILE_RIGHT: TileVector = TileVector::new(1, 0);
pub const TILE_UP_LEFT: TileVector = TileVector::new(-1, -1);
pub const TILE_UP_RIGHT: TileVector = TileVector::new(1, -1);
pub const TILE_DOWN_LEFT: TileVector = TileVector::new(-1, 1);
pub const TILE_DOWN_RIGHT: TileVector = TileVector::new(1, 1);

/// Offset to a random adjacent tile, excluding diagonals.
pub fn random_neighbor_four(rng: &mut Pcg32) -> TileVector {
	*[TILE_UP, TILE_DOWN, TILE_LEFT, TILE_RIGHT]
		.choose(rng)
		.unwrap()
}

/// Offset to a random adjacent tile, including diagonals.
pub fn random_neighbor_eight(rng: &mut Pcg32) -> TileVector {
	*[
		TILE_UP,
		TILE_DOWN,
		TILE_LEFT,
		TILE_RIGHT,
		TILE_UP_LEFT,
		TILE_UP_RIGHT,
		TILE_DOWN_LEFT,
		TILE_DOWN_RIGHT,
	]
	.choose(rng)
	.unwrap()
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

#[derive(Clone, Copy)]
pub enum RectangleIntersection<T> {
	/// The rectangles actually intersect.
	Real(Rectangle<T>),
	/// The rectangles share x-coordinates but not y-coordinates. The contained
	/// rectangle represents the empty space between the horizontally
	/// overlapping regions.
	Horizontal(Rectangle<T>),
	/// The rectangles share y-coordinates but not x-coordinates. The contained
	/// rectangle represents the empty space between the vertically overlapping
	/// regions.
	Vertical(Rectangle<T>),
	/// The rectangles share neither x- nor y-coordinates. The contained
	/// rectangle represents the empty space between the rectangles' nearest
	/// corners.
	None(Rectangle<T>),
}

impl<T> RectangleIntersection<T> {
	/// The Manhattan distance between the rectangles. If the rectangles
	/// overlap, their distance is zero. Note that this function measures from
	/// edges/corners to edges/corners. For instance, nonintersecting rectangles
	/// that share a corner are considered to be zero distance apart.
	pub fn distance(self) -> T
	where
		T: Copy + Add<Output = T> + Zero,
	{
		// Overlapping x- or y-coordinates contribute zero distance.
		match self {
			RectangleIntersection::Real(_) => T::zero(),
			RectangleIntersection::Horizontal(i) => i.size.y,
			RectangleIntersection::Vertical(i) => i.size.x,
			RectangleIntersection::None(i) => i.size.x + i.size.y,
		}
	}
}

impl<T> Rectangle<T> {
	/// The nonempty overlapping region of `self` and `other`, if any.
	pub fn intersection(self, other: Self) -> RectangleIntersection<T>
	where
		T: Copy + Ord + Add<Output = T> + Sub<Output = T>,
	{
		let start_x = self.pos.x.max(other.pos.x);
		let start_y = self.pos.y.max(other.pos.y);
		let end_x = (self.pos.x + self.size.x).min(other.pos.x + other.size.x);
		let end_y = (self.pos.y + self.size.y).min(other.pos.y + other.size.y);
		match (start_x < end_x, start_y < end_y) {
			(true, true) => RectangleIntersection::Real(Rectangle {
				pos: Point::new(start_x, start_y),
				size: Vector::new(end_x - start_x, end_y - start_y),
			}),
			(true, false) => RectangleIntersection::Horizontal(Rectangle {
				pos: Point::new(start_x, end_y),
				size: Vector::new(end_x - start_x, start_y - end_y),
			}),
			(false, true) => RectangleIntersection::Vertical(Rectangle {
				pos: Point::new(end_x, start_y),
				size: Vector::new(start_x - end_x, end_y - start_y),
			}),
			(false, false) => RectangleIntersection::None(Rectangle {
				pos: Point::new(end_x, end_y),
				size: Vector::new(start_x - end_x, start_y - end_y),
			}),
		}
	}

	/// Whether `self` and `other` are overlapping or adjacent.
	pub fn touching(self, other: Self) -> bool
	where
		T: Copy + Ord + Add<Output = T> + Sub<Output = T>,
	{
		let start_x = self.pos.x.max(other.pos.x);
		let start_y = self.pos.y.max(other.pos.y);
		let end_x = (self.pos.x + self.size.x).min(other.pos.x + other.size.x);
		let end_y = (self.pos.y + self.size.y).min(other.pos.y + other.size.y);
		start_x <= end_x && start_y <= end_y
	}

	/// The rectangle's width times height.
	pub fn area(self) -> T
	where
		T: Mul<Output = T>,
	{
		self.size.x * self.size.y
	}
}
