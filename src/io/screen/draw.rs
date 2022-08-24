use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Draw {
    pub scans: Vec<(usize, String)>,
    pub collision: bool,
}

impl Draw {
    pub fn new(capacity: usize) -> Self {
        Self {
            scans: Vec::with_capacity(capacity),
            collision: false,
        }
    }
}
