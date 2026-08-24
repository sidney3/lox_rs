use super::Function;
use super::ObjData;
use super::ObjKind;
use crate::gc::{Trace, Tracer};
use crate::runtime::Symbol;
use crate::runtime::{Runtime, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ClassDef {
  pub ident: Symbol,
}

#[derive(Debug)]
pub struct ClassInstance {
  ident: Symbol,
  properties: HashMap<Symbol, Value>,
}

impl ClassDef {
  pub fn new(ident: Symbol) -> Self {
    Self { ident }
  }
}

impl ObjKind for ClassDef {
  fn embed(self) -> ObjData {
    ObjData::ClassDef(self)
  }
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::ClassDef(class) => Some(class),
      _ => None,
    }
  }
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::ClassDef(class) => Some(class),
      _ => None,
    }
  }
}

impl ClassInstance {
  pub fn new(def: &ClassDef) -> Self {
    Self {
      ident: def.ident,
      properties: HashMap::new(),
    }
  }

  pub fn equals(&self, rt: &Runtime, rhs: &ClassInstance) -> bool {
    rhs.ident == self.ident
  }
  pub fn load_attr(&self, sym: Symbol) -> Option<Value> {
    self.properties.get(&sym).cloned()
  }
  pub fn set_attr(&mut self, sym: Symbol, to: Value) {
    self.properties.insert(sym, to);
  }
  pub fn name<'a>(&self, rt: &'a Runtime) -> &'a str {
    rt.symbols.resolve(&self.ident)
  }
}

impl Trace<ObjData> for ClassDef {
  fn trace(&self, _: &mut Tracer<ObjData>) {}
}

impl ObjKind for ClassInstance {
  fn embed(self) -> ObjData {
    ObjData::ClassInstance(self)
  }
  fn project(obj: &ObjData) -> Option<&Self> {
    match obj {
      ObjData::ClassInstance(class) => Some(class),
      _ => None,
    }
  }
  fn project_mut(obj: &mut ObjData) -> Option<&mut Self> {
    match obj {
      ObjData::ClassInstance(class) => Some(class),
      _ => None,
    }
  }
}

impl Trace<ObjData> for ClassInstance {
  fn trace(&self, tracer: &mut Tracer<ObjData>) {
    for member in self.properties.values() {
      member.trace(tracer);
    }
  }
}
