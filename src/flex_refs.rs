
use std::{
    rc::Rc, cell::{RefCell, Ref, RefMut/*, UnsafeCell, Cell*/},
    /*ops::{Deref, DerefMut},*/
    convert::From, hash::{Hash, Hasher}, cmp::{PartialEq, Eq}, clone::Clone,
};


// usafe refs traits

// pub trait UnsafeRef<'a> { fn unsafe_ref(&self) -> &'a Self { unsafe { &*(self as *const Self) } } }
// pub trait UnsafeMut<'a> { fn unsafe_mut(&mut self) -> &'a mut Self { unsafe { &mut*(self as *mut Self) } } }


// Flr wrapper type
pub struct Flr<T>(Rc<RefCell<T>>);

impl<T> Flr<T> {
    pub fn new(data: T) -> Self { Self(RefCell::new(data).into()) }
    pub fn get(&self) -> Ref<T> { self.0.borrow() }
    pub fn get_mut(&self) -> RefMut<T> { self.0.borrow_mut() }
}

impl<T> From<T> for Flr<T> { fn from(other: T) -> Self { Self(RefCell::from(other).into()) } }

impl<T> Hash for Flr<T> { fn hash<H: Hasher>(&self, state: &mut H) { self.0.as_ptr().hash(state) } }
impl<T> PartialEq<Flr<T>> for Flr<T> { fn eq(&self, other: &Self) -> bool { self.0.as_ptr() == other.0.as_ptr() } }
impl<T> Eq for Flr<T> {}

impl<T> Clone for Flr<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }


// unsafe Flr wrapper type
/*pub struct Flr<T>(Rc<UnsafeCell<T>>);

impl<T> Flr<T> {
    pub fn new(data: T) -> Self { Self(UnsafeCell::new(data).into()) }
    // pub fn get(&self) -> &T { unsafe { &* self.0.get() } }
    // pub fn get_mut(&self) -> &mut T { unsafe { &mut* self.0.get() } }
}

impl<T> Deref for Flr<T> { type Target = T; fn deref(&self) -> &Self::Target {
   unsafe { &*((&*self.0.get()) as *const T) }
} }
impl<T> DerefMut for Flr<T> { fn deref_mut(&mut self) -> &mut Self::Target {
   unsafe { &mut*((&mut*self.0.get()) as *mut T) }
} }

impl<T> From<T> for Flr<T> { fn from(other: T) -> Self { Self(UnsafeCell::from(other).into()) } }

impl<T> Hash for Flr<T> { fn hash<H: Hasher>(&self, state: &mut H) { self.0.get().hash(state) } }
impl<T> PartialEq<Flr<T>> for Flr<T> { fn eq(&self, other: &Self) -> bool { self.0.get() == other.0.get() } }
impl<T> Eq for Flr<T> {}

impl<T> Clone for Flr<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }*/


// FlexBox
/*pub struct FlexBox<T: ?Sized>(Rc<RefCell<Box<T>>>);

impl<T> FlexBox<T> {
    pub fn new(data: T) -> Self { Self(RefCell::new(Box::new(data)).into()) }
    pub fn get(&self) -> Ref<Box<T>> { self.0.borrow() }
    pub fn get_mut(&self) -> RefMut<Box<T>> { self.0.borrow_mut() }
}

impl<T> Deref for FlexBox<T> { type Target = RefCell<Box<T>>; fn deref(&self) -> &Self::Target { &self.0 } }
impl<T> From<T> for FlexBox<T> { fn from(other: T) -> Self { Self(RefCell::from(Box::new(other)).into()) } }

impl<T> Hash for FlexBox<T> { fn hash<H: Hasher>(&self, state: &mut H) { self.0.as_ptr().hash(state) } }
impl<T> PartialEq<FlexBox<T>> for FlexBox<T> { fn eq(&self, other: &Self) -> bool { self.0.as_ptr() == other.0.as_ptr() } }
impl<T> Eq for FlexBox<T> {}

impl<T> Clone for FlexBox<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }*/
