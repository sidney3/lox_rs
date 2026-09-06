use std::{collections::HashMap, hash::Hash, ops::Index};

pub struct InternPool<Id, T> {
  elt_to_id: HashMap<T, Id>,
  id_to_elt: Vec<T>, // indexed by Id
}

impl<Id, T> InternPool<Id, T>
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

  pub fn get_or_intern(&mut self, t: T) -> Id
  where
    T: Clone,
  {
    match self.elt_to_id.get(&t) {
      Some(id) => *id,
      None => {
        let id: Id = self.id_to_elt.len().into();
        self.elt_to_id.insert(t.clone(), id);
        self.id_to_elt.push(t);

        id
      }
    }
  }

  fn get_left(&self, i: Id) -> &T {
    &self.id_to_elt[i.into()]
  }

  pub fn keys(&self) -> impl Iterator<Item = &Id> {
    self.elt_to_id.values()
  }

  // total number of interned values
  pub fn len(&self) -> usize {
    self.id_to_elt.len()
  }
}

impl<Id, T> Index<Id> for InternPool<Id, T>
where
  T: Hash + Eq,
  Id: Into<usize> + From<usize> + Copy + Clone,
{
  type Output = T;
  fn index(&self, index: Id) -> &Self::Output {
    self.get_left(index)
  }
}
