use std::ops::Deref;
use std::rc::Rc;

pub struct HashRc<T>(Rc<T>);

impl<T> std::borrow::Borrow<Rc<T>> for HashRc<T> {
    fn borrow(&self) -> &Rc<T> {
        &self.0
    }
}

impl<T> From<Rc<T>> for HashRc<T> {
    fn from(value: Rc<T>) -> Self {
        Self(value)
    }
}

impl<T> Deref for HashRc<T> {
    type Target = Rc<T>;
    fn deref(&self) -> &Rc<T> {
        &self.0
    }
}

impl<T> std::hash::Hash for HashRc<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0.as_ref() as *const T).hash(state)
    }
}

impl<T> PartialEq for HashRc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for HashRc<T> {}
