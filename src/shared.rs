use std::{cell::RefCell, rc::Rc};

pub type Shared<T> = Rc<RefCell<T>>;

pub fn share<T>(t: T) -> Shared<T> {
	Rc::new(RefCell::new(t))
}
