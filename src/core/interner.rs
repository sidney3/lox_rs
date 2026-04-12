use std::{collections::HashMap, hash::Hash};

pub struct Interner<Id, T> {
    elt_to_id: HashMap<T, Id>,
    id_to_elt: Vec<T>, // indexed by Id
}

impl<Id, T> Interner<Id, T>
where
    T: Hash + Eq,
    Id: Into<usize> + From<usize> + Copy + Clone,
{
    pub fn new() -> Self {
        Self {
            elt_to_id: HashMap::new(),
            id_to_elt: Vec::new(),
        }
    }

    pub fn intern(&mut self, t: T) -> Id {
        match self.elt_to_id.get(&t) {
            Some(id) => *id,
            None => {
                let id = self.id_to_elt.len().into();
                self.id_to_elt.push(t);

                id
            }
        }
    }

    pub fn get_right(&self, t: &T) -> Option<Id> {
        self.elt_to_id.get(t).cloned()
    }
    pub fn get_left(&self, i: Id) -> &T {
        &self.id_to_elt[i.into()]
    }

    pub fn all(&self) -> impl Iterator<Item = &Id> {
        self.elt_to_id.values()
    }

    // total number of interned values
    pub fn len(&self) -> usize {
        return self.id_to_elt.len();
    }
}
