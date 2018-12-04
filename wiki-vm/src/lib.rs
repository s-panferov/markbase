extern crate libc;
extern crate mozjs;

pub use crate::js::{ErrorInfo, Func, Vm};
use std::rc::Rc;

mod js;

pub struct Compiler {
  vm: Box<Vm>,
  func: Rc<Func>,
}

impl Compiler {
  pub fn new(src: &str) -> Compiler {
    let mut vm = js::Vm::new();
    let func = vm.compile("script", &src).unwrap();

    Compiler { vm, func }
  }

  pub fn compile(&mut self, buf: String) -> Result<String, ErrorInfo> {
    self.func.call_string(&mut self.vm, buf)
  }
}
