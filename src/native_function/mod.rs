use crate::obj::NativeFunction;
use crate::runtime::Runtime;
mod assert;
mod print;

pub fn load_native_functions(rt: &mut Runtime) {
  rt.add_native(NativeFunction::new("print", Box::new(print::print)));
  rt.add_native(NativeFunction::new("assert", Box::new(assert::assert)));
}
