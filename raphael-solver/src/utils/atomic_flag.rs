use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Default)]
pub struct AtomicFlag {
    flag: Arc<AtomicBool>,
}

// https://users.rust-lang.org/t/compiler-hint-for-unlikely-likely-for-if-branches/62102/3
#[inline]
#[cold]
fn cold() {}

#[inline]
fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}

impl AtomicFlag {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn as_ptr(&self) -> *mut bool {
        self.flag.as_ptr()
    }

    pub fn set(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    #[inline]
    pub fn is_set(&self) -> bool {
        unlikely(self.flag.load(Ordering::Relaxed))
    }

    pub fn clear(&self) {
        self.flag.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::AtomicFlag;

    #[test]
    fn test_atomic_flag() {
        let flag = AtomicFlag::new();
        assert!(!flag.is_set());

        flag.set();
        assert!(flag.is_set());

        flag.clear();
        assert!(!flag.is_set());
    }
}
