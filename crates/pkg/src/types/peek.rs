use std::{cell::Cell, rc::Rc};

pub struct Peek<T: Copy>(Rc<Cell<T>>);

impl<T: Copy> Peek<T> {
    pub fn new(v: Rc<Cell<T>>) -> Self {
        Self(v)
    }

    pub fn get(&self) -> T {
        self.0.get()
    }
}

// pub type Immutable<'a, X> = &'a X;
