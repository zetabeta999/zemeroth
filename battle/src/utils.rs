use std::fmt::Debug;

/// Remove an element from a vector.
pub fn try_remove_item<T: Debug + PartialEq>(vec: &mut Vec<T>, e: &T) -> bool {
    vec.iter()
        .position(|current| current == e)
        .map(|e| vec.remove(e))
        .is_some()
}

pub fn clamp_min<T: PartialOrd>(value: T, min: T) -> T {
    if value < min {
        min
    } else {
        value
    }
}

pub fn clamp_max<T: PartialOrd>(value: T, max: T) -> T {
    if value > max {
        max
    } else {
        value
    }
}

pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleRng {
    seed: u32,
    a: u32,
    c: u32,
}

impl SimpleRng {
    pub fn seed_from_u32(seed: u32) -> Self {
        SimpleRng {
            seed,
            a: 22695477,
            c: 1,
        }
    }

    fn next(&mut self) -> u32 {
        self.seed = self.a.wrapping_mul(self.seed).wrapping_add(self.c);
        self.seed
    }

    pub fn gen_range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next() % (max - min) as u32) as i32
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clamp_min() {
        assert_eq!(super::clamp_min(1, 0), 1);
        assert_eq!(super::clamp_min(0, 0), 0);
        assert_eq!(super::clamp_min(-1, 0), 0);
    }

    #[test]
    fn test_clamp_max() {
        assert_eq!(super::clamp_max(1, 2), 1);
        assert_eq!(super::clamp_max(2, 2), 2);
        assert_eq!(super::clamp_max(3, 2), 2);
    }

    #[test]
    fn test_clamp() {
        let min = 0;
        let max = 2;
        assert_eq!(super::clamp(1, min, max), 1);
        assert_eq!(super::clamp(0, min, max), 0);
        assert_eq!(super::clamp(-1, min, max), 0);
        assert_eq!(super::clamp(1, min, max), 1);
        assert_eq!(super::clamp(2, min, max), 2);
        assert_eq!(super::clamp(3, min, max), 2);
    }

    #[test]
    fn test_try_remove_item() {
        let mut a = vec![1, 2, 3];
        assert!(super::try_remove_item(&mut a, &1));
        assert_eq!(&a, &[2, 3]);
        assert!(!super::try_remove_item(&mut a, &666));
    }
}
