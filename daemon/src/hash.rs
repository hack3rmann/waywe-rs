use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hasher},
};

pub type TfHashMap<K, V> = HashMap<K, V, TakeFirstBuildHasher>;
pub type TfHashSet<T> = HashSet<T, TakeFirstBuildHasher>;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct TakeFirstBuildHasher;

impl BuildHasher for TakeFirstBuildHasher {
    type Hasher = TakeFirstHasher;

    fn build_hasher(&self) -> Self::Hasher {
        TakeFirstHasher::default()
    }
}

#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct TakeFirstHasher {
    pub value: u64,
}

impl Hasher for TakeFirstHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, bytes: &[u8]) {
        self.value ^= bytes
            .iter()
            .enumerate()
            .map(|(index, &byte)| (byte as u64).wrapping_shl(8 * index as u32))
            .take(8)
            .reduce(|acc, elem| acc | elem)
            .unwrap_or(0);
    }

    fn write_u8(&mut self, i: u8) {
        self.value ^= i as u64;
    }

    fn write_u16(&mut self, i: u16) {
        self.value ^= i as u64;
    }

    fn write_u32(&mut self, i: u32) {
        self.value ^= i as u64;
    }

    fn write_u64(&mut self, i: u64) {
        self.value ^= i;
    }

    fn write_u128(&mut self, i: u128) {
        self.value ^= i as u64;
    }

    fn write_usize(&mut self, i: usize) {
        self.value ^= i as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_trivial_collisions() {
        let hash = |s: &str| -> u64 {
            let mut hasher = TakeFirstHasher::default();
            hasher.write(s.as_bytes());
            hasher.finish()
        };

        for i in 0..10 {
            for j in 0..10 {
                let edp_i = format!("eDP-{i}");
                let edp_j = format!("eDP-{j}");
                let hdmi_i = format!("HDMI-A-{i}");
                let hdmi_j = format!("HDMI-A-{j}");

                assert_ne!(hash(&edp_i), hash(&hdmi_j));

                if j > i {
                    assert_ne!(hash(&edp_i), hash(&edp_j));
                    assert_ne!(hash(&hdmi_i), hash(&hdmi_j));
                }
            }
        }
    }
}
