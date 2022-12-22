//! [`Entity`] implementation, storage, and interation.

use crate::prelude::*;

/// An entity index.
///
/// They are created using the `Entities` struct. They are used as indices with `Components`
/// structs.
///
/// Entities are conceptual "things" which possess attributes (Components). As an exemple, a Car
/// (Entity) has a Color (Component), a Position (Component) and a Speed (Component).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(u32, u32);
impl Entity {
    /// Creates a new `Entity` from the provided index and generation.
    pub(crate) fn new(index: u32, generation: u32) -> Entity {
        Entity(index, generation)
    }

    /// Returns the index of this `Entity`.
    ///
    /// In most cases, you do not want to use this directly.
    /// However, it can be useful to create caches to improve performances.
    pub fn index(&self) -> u32 {
        self.0
    }

    /// Returns the generation of this `Entity`.
    ///
    ///
    /// In most cases, you do not want to use this directly.
    /// However, it can be useful to create caches to improve performances.
    pub fn generation(&self) -> u32 {
        self.1
    }
}

/// Holds a list of alive entities.
///
/// It also holds a list of entities that were recently killed, which allows to remove components of
/// deleted entities at the end of a game frame.
#[derive(TypeUuid, Clone)]
#[uuid = "15f8b67d-a093-4a99-b87e-5eca4bb32e25"]
pub struct Entities {
    /// Bitset containing all living entities
    alive: BitSetVec,
    generation: Vec<u32>,
    killed: Vec<Entity>,
    next_id: usize,
    /// helps to know if we should directly append after next_id or if we should look through the
    /// bitset.
    has_deleted: bool,
}

impl Default for Entities {
    fn default() -> Self {
        Self {
            alive: create_bitset(),
            generation: vec![0u32; BITSET_SIZE],
            killed: vec![],
            next_id: 0,
            has_deleted: false,
        }
    }
}

impl Entities {
    /// Creates a new `Entity` and returns it.
    ///
    /// This function will not reuse the index of an entity that is still in the killed entities.
    pub fn create(&mut self) -> Entity {
        if !self.has_deleted {
            let i = self.next_id;
            if i >= BITSET_SIZE {
                panic!("Exceeded maximum amount of concurrent entities.");
            }
            self.next_id += 1;
            self.alive.bit_set(i);
            Entity::new(i as u32, self.generation[i])
        } else {
            let mut section = 0;
            // Find section where at least one bit isn't set
            while self.alive[section].bit_all() {
                section += 1;
            }
            let mut i = section * (32 * 8);
            while self.alive.bit_test(i) || self.killed.iter().any(|e| e.index() == i as u32) {
                i += 1;
            }
            self.alive.bit_set(i);
            if i >= self.next_id {
                self.next_id = i + 1;
                self.has_deleted = false;
            }
            if i >= BITSET_SIZE {
                panic!("Exceeded maximum amount of concurrent entities.");
            }
            Entity::new(i as u32, self.generation[i])
        }
    }

    /// Checks if the `Entity` is still alive.
    ///
    /// Returns true if it is alive. Returns false if it has been killed.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive.bit_test(entity.index() as usize)
            && self.generation[entity.index() as usize] == entity.generation()
    }

    /// Kill an entity.
    pub fn kill(&mut self, entity: Entity) {
        if self.alive.bit_test(entity.index() as usize) {
            self.alive.bit_reset(entity.index() as usize);
            self.generation[entity.index() as usize] += 1;
            self.killed.push(entity);
            self.has_deleted = true;
        }
    }

    /// Returns entities in the killed list.
    pub fn killed(&self) -> &Vec<Entity> {
        &self.killed
    }

    /// Clears the killed entity list.
    pub fn clear_killed(&mut self) {
        self.killed.clear();
    }

    /// Returns a bitset where each index where the bit is set to 1 indicates the index of an alive
    /// entity.
    ///
    /// Useful for joining over [`Entity`] and [`ComponentStore<T>`] at the same time.
    pub fn bitset(&self) -> &BitSetVec {
        &self.alive
    }

    /// Iterates over entities using the provided bitset.
    pub fn iter_with_bitset(&self, bitset: std::rc::Rc<BitSetVec>) -> EntityIterator {
        EntityIterator {
            current_id: 0,
            next_id: self.next_id,
            entities: &self.alive,
            generations: &self.generation,
            bitset,
        }
    }
}

/// Iterator over entities using the provided bitset.
pub struct EntityIterator<'a> {
    pub(crate) current_id: usize,
    pub(crate) next_id: usize,
    pub(crate) entities: &'a BitSetVec,
    pub(crate) generations: &'a Vec<u32>,
    //pub(crate) bitset: &'a BitSetVec,
    pub(crate) bitset: std::rc::Rc<BitSetVec>,
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = Option<Entity>;
    fn next(&mut self) -> Option<Self::Item> {
        while !self.bitset.bit_test(self.current_id) && self.current_id < self.next_id {
            self.current_id += 1;
        }
        let ret = if self.current_id < self.next_id {
            if self.entities.bit_test(self.current_id) {
                Some(Some(Entity::new(
                    self.current_id as u32,
                    self.generations[self.current_id],
                )))
            } else {
                Some(None)
            }
        } else {
            None
        };
        self.current_id += 1;
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::prelude::*;

    #[test]
    fn create_kill_entities() {
        let mut entities = Entities::default();
        let e1 = entities.create();
        let e2 = entities.create();
        let e3 = entities.create();
        assert_eq!(e1.index(), 0);
        assert_eq!(e2.index(), 1);
        assert_eq!(e3.index(), 2);
        assert_eq!(e1.generation(), 0);
        assert!(entities.is_alive(e1));
        assert!(entities.is_alive(e2));
        assert!(entities.is_alive(e3));
        entities.kill(e1);
        assert!(!entities.is_alive(e1));
        assert!(entities.is_alive(e2));
        assert!(entities.is_alive(e3));
        let e4 = entities.create();
        assert!(!entities.is_alive(e1));
        assert!(entities.is_alive(e2));
        assert!(entities.is_alive(e3));
        assert!(entities.is_alive(e4));

        assert_eq!(*entities.killed(), vec![e1]);
        entities.clear_killed();
        assert_eq!(*entities.killed(), vec![]);
    }

    #[test]
    fn test_interleaved_create_kill() {
        let mut entities = Entities::default();

        let e1 = entities.create();
        assert_eq!(e1.index(), 0);
        let e2 = entities.create();
        assert_eq!(e2.index(), 1);
        entities.kill(e1);
        entities.kill(e2);
        assert!(!entities.is_alive(e1));
        assert!(!entities.is_alive(e2));

        let e3 = entities.create();
        assert_eq!(e3.index(), 2);
        let e4 = entities.create();
        assert_eq!(e4.index(), 3);
        entities.kill(e3);
        entities.kill(e4);
        assert!(!entities.is_alive(e3));
        assert!(!entities.is_alive(e4));
    }

    #[test]
    fn clone_debug_hash() {
        let mut entities = Entities::default();

        let e1 = entities.create();
        let _ = e1.clone();
        println!("{:?}", e1);
        let mut h = HashSet::new();
        h.insert(e1);
    }

    /// Test to cover the code where an entity is allocated in the next free section.
    ///
    /// Exercises a code path not tested according to code coverage.
    #[test]
    fn force_generate_next_section() {
        let mut entities = Entities::default();
        // Create enough entities to fil up the first section of the bitset
        for _ in 0..256 {
            entities.create();
        }
        // Create another entity ( this will be the second section)
        let e1 = entities.create();
        // Kill the entity ( now we will have a deleted entity, but not in the first section )
        entities.kill(e1);
        // Create a new entity
        entities.create();
    }

    #[test]
    #[should_panic(expected = "Exceeded maximum amount")]
    fn force_max_entity_panic() {
        let mut entities = Entities::default();
        for _ in 0..(BITSET_SIZE + 1) {
            entities.create();
        }
    }

    #[test]
    #[should_panic(expected = "Exceeded maximum amount")]
    fn force_max_entity_panic2() {
        let mut entities = Entities::default();
        let mut e = None;
        for _ in 0..BITSET_SIZE {
            e = Some(entities.create());
        }
        let e = e.unwrap();
        entities.kill(e);
        entities.create();
        entities.create();
    }
}
