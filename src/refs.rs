
use std::{
    rc::Rc, cell::{Cell/*, RefCell, Ref, RefMut*/}, ops::Deref,
    convert::From, hash::{Hash, Hasher}, cmp::{PartialEq, Eq}, clone::Clone,
};


// RcCell wrapper type
pub struct RcCell<T>(Rc<Cell<T>>);

impl<T> RcCell<T> {
    pub fn new(data: T) -> Self { Self(Cell::new(data).into()) }
}

impl<T> Deref for RcCell<T> { type Target = Cell<T>; fn deref(&self) -> &Self::Target { &self.0 } }

impl<T> From<T> for RcCell<T> { fn from(other: T) -> Self { Self(Cell::from(other).into()) } }

impl<T> Hash for RcCell<T> { fn hash<H: Hasher>(&self, state: &mut H) { self.0.as_ptr().hash(state) } }
impl<T> PartialEq<RcCell<T>> for RcCell<T> { fn eq(&self, other: &Self) -> bool { self.0.as_ptr() == other.0.as_ptr() } }
impl<T> Eq for RcCell<T> {}

impl<T> Clone for RcCell<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }



/*// RcRfC wrapper type
pub struct RcRfC<T>(Rc<RefCell<T>>);

impl<T> RcRfC<T> {
    pub fn new(data: T) -> Self { Self(RefCell::new(data).into()) }
    pub fn get(&self) -> Ref<T> { self.0.borrow() }
    pub fn get_mut(&self) -> RefMut<T> { self.0.borrow_mut() }
}

impl<T> From<T> for RcRfC<T> { fn from(other: T) -> Self { Self(RefCell::from(other).into()) } }

impl<T> Hash for RcRfC<T> { fn hash<H: Hasher>(&self, state: &mut H) { self.0.as_ptr().hash(state) } }
impl<T> PartialEq<RcRfC<T>> for RcRfC<T> { fn eq(&self, other: &Self) -> bool { self.0.as_ptr() == other.0.as_ptr() } }
impl<T> Eq for RcRfC<T> {}

impl<T> Clone for RcRfC<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }*/