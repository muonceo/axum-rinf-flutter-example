use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Counter {
    pub number: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { number: 0 }
    }

    pub fn increment(&mut self) {
        self.number += 1;
    }

    pub fn get(&self) -> i32 {
        self.number
    }

    pub fn set(&mut self, number: i32) {
        self.number = number;
    }
}

#[cfg(test)]
mod tests {
    use crate::Counter;

    #[test]
    fn test_counter() {
        let mut counter = Counter::new();
        assert_eq!(counter.get(), 0);
        counter.increment();
        assert_eq!(counter.get(), 1);
        counter.increment();
        assert_eq!(counter.get(), 2);
        counter.set(0);
        assert_eq!(counter.get(), 0);
    }
}
