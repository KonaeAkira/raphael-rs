use crate::game::{actions::Action, effects::Effects, state::InProgress};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParetoKey {
    pub last_action: Option<Action>,
    pub durability: i32,
    pub effects: Effects,
}

impl ParetoKey {
    pub fn new(state: &InProgress) -> ParetoKey {
        ParetoKey {
            last_action: state.last_action,
            durability: state.durability,
            effects: state.effects,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParetoValue {
    pub progress: i32,
    pub quality: i32,
}

impl ParetoValue {
    pub fn new(state: &InProgress) -> ParetoValue {
        ParetoValue {
            progress: state.progress,
            quality: state.quality,
        }
    }
}

impl PartialOrd for ParetoValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.progress.cmp(&other.progress),
            self.quality.cmp(&other.quality),
        ) {
            (Ordering::Less, Ordering::Less) => Some(Ordering::Less),
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
            (Ordering::Greater, Ordering::Greater) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ParetoFront {
    hash_map: HashMap<ParetoKey, Vec<ParetoValue>>,
}

impl ParetoFront {
    pub fn new() -> ParetoFront {
        ParetoFront {
            hash_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, state: &InProgress) -> bool {
        let key = ParetoKey::new(&state);
        if !self.hash_map.contains_key(&key) {
            self.hash_map.insert(key.clone(), Vec::new());
        }
        let front: &mut Vec<ParetoValue> = self.hash_map.get_mut(&key).unwrap();
        let new_value = ParetoValue::new(&state);
        for value in front.iter_mut() {
            match (*value).partial_cmp(&new_value) {
                Some(Ordering::Less) => *value = new_value.clone(),
                Some(Ordering::Equal) | Some(Ordering::Greater) => return false,
                None => (),
            };
        }
        front.dedup();
        front.push(new_value);
        true
    }

    pub fn has(&self, state: &InProgress) -> bool {
        let key = ParetoKey::new(&state);
        if let Some(front) = self.hash_map.get(&key) {
            let new_value = ParetoValue::new(&state);
            for value in front.iter() {
                if *value == new_value {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }
}
