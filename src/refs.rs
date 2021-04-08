
use std::{
    rc::Rc, cell::{UnsafeCell},
    ops::{Deref, DerefMut},
    convert::From, hash::{Hash, Hasher}, cmp::{PartialEq, Eq}, clone::Clone,
    fmt, fmt::{Formatter, Debug}
};



// id trait

pub trait Id {
    fn id(&self) -> usize;
}


// flexible reference type

pub struct Flr<T>(Rc<UnsafeCell<T>>);

impl<T> Flr<T> {
    pub fn new(data: T) -> Self { Self(UnsafeCell::new(data).into()) }
    /*pub fn get(&self) -> &T { unsafe { &* self.0.get() } }
    pub fn get_mut(&self) -> &mut T { unsafe { &mut* self.0.get() } }*/
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

impl<T> Clone for Flr<T> { fn clone(&self) -> Self { Self(self.0.clone()) } }


impl<T: Debug> Debug for Flr<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        (**self).fmt(formatter)
    }
}


impl<T> Id for Flr<T> {
    fn id(&self) -> usize { Rc::<UnsafeCell<T>>::as_ptr(&self.0) as *const Self as *const usize as usize }
}




