use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(i32);

impl Id {
    pub const LOCALHOST: Self = Self(-1);
    pub fn replay(index: usize) -> Self {
        Self(-(index as i32 + 2))
    }
}

pub struct IdGen {
    next: i32,
}

impl IdGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }
    pub fn gen(&mut self) -> Id {
        let id = Id(self.next);
        self.next += 1;
        id
    }
}

impl Default for IdGen {
    fn default() -> Self {
        Self::new()
    }
}
