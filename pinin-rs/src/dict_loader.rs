pub trait DictLoader {
    fn load(&self, feed: &mut dyn FnMut(char, &[&str]));
    fn load_code_points(&self, _feed: &mut dyn FnMut(u32, &[&str])) {}
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultLoader;

impl DictLoader for DefaultLoader {
    fn load(&self, feed: &mut dyn FnMut(char, &[&str])) {
        for line in include_str!("data.txt").lines() {
            if line.is_empty() {
                continue;
            }
            let Some((lhs, rhs)) = line.split_once(": ") else { continue; };
            let records: Vec<&str> = rhs.split(", ").collect();
            if let Some(ch) = lhs.chars().next() {
                feed(ch, &records);
            }
        }
    }

    fn load_code_points(&self, feed: &mut dyn FnMut(u32, &[&str])) {
        for line in include_str!("extra.txt").lines() {
            if line.is_empty() {
                continue;
            }
            let Some((lhs, rhs)) = line.split_once(": ") else { continue; };
            let records: Vec<&str> = rhs.split(", ").collect();
            if let Some(ch) = lhs.chars().next() {
                feed(ch as u32, &records);
            }
        }
    }
}
