use super::module_execution::ModuleExecution;
use super::runtime_error::RuntimeError;
use super::value::Value;
use crate::codegen::Chunk;

pub struct Vm {
    pub stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn run<'a>(&mut self, module: &'a Chunk) -> Result<(), RuntimeError> {
        ModuleExecution::new(self, module).execute()
    }
}
