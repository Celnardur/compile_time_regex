use std::{rc::Rc, cell::RefCell};

pub type Pointer<T> = Rc<RefCell<T>>;

#[macro_export]
macro_rules! ptr {
    ($e:expr) => { Rc::new(RefCell::new($e)) };
}

#[macro_export]
macro_rules! ptrcp {
    ($e:expr) => { Rc::clone(&$e) };
}

#[macro_export]
macro_rules! deref {
    ($e:expr) => { *$e.borrow_mut() };
}
