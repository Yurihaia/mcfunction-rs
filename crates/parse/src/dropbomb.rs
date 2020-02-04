use std::fmt::Display;

#[derive(Debug)]
pub struct DropBomb<T: Display>(bool, T);

impl<T: Display> Drop for DropBomb<T> {
    fn drop(&mut self) {
        if !self.0 {
            panic!("{}", self.1);
        }
    }
}

impl<T: Display> DropBomb<T> {
    pub fn new(msg: T) -> Self {
        DropBomb(false, msg)
    }

    pub fn defuse(&mut self) {
        self.0 = true;
    }
}
