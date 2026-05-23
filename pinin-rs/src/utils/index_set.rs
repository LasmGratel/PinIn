#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct IndexSet(pub u32);

impl IndexSet {
    pub const ZERO: IndexSet = IndexSet(0x1);
    pub const ONE: IndexSet = IndexSet(0x2);
    pub const NONE: IndexSet = IndexSet(0x0);

    pub fn set(&mut self, index: u32) {
        if index < 32 {
            self.0 |= 1 << index;
        }
    }

    pub fn get(self, index: u32) -> bool {
        index < 32 && ((self.0 >> index) & 1) == 1
    }

    pub fn merge(&mut self, other: IndexSet) {
        self.0 = if self.0 == 0x1 { other.0 } else { self.0 | other.0 };
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn offset(&mut self, i: u32) {
        self.0 <<= i;
    }

    pub fn foreach(self, mut f: impl FnMut(u32)) {
        let mut v = self.0;
        let mut i = 0u32;
        while v != 0 && i < 32 {
            if v & 1 == 1 {
                f(i);
            }
            v >>= 1;
            i += 1;
        }
    }

    pub fn traverse(self, mut f: impl FnMut(u32) -> bool) -> bool {
        let mut v = self.0;
        let mut i = 0u32;
        while i < 32 {
            if v == 0 {
                return false;
            }
            if v & 1 == 1 && f(i) {
                return true;
            }
            v >>= 1;
            i += 1;
        }
        false
    }
}
