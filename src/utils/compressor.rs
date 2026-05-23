#[derive(Clone, Default, Debug)]
pub struct Compressor {
    data: String,
    offsets: Vec<usize>,
}

impl Compressor {
    pub fn put(&mut self, s: &str) -> usize {
        let offset = self.data.len();
        self.offsets.push(offset);
        self.data.push_str(s);
        self.data.push('\0');
        offset
    }

    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }

    pub fn end(&self, i: usize) -> bool {
        self.data.as_bytes().get(i).copied() == Some(0)
    }

    pub fn get_char(&self, i: usize) -> char {
        self.data[i..].chars().next().unwrap()
    }

    pub fn get(&self, i: usize) -> u32 {
        self.get_char(i) as u32
    }

    pub fn char_len(&self, i: usize) -> usize {
        self.get_char(i).len_utf8()
    }
}
