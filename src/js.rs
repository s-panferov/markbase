use mozjs::glue;
use mozjs::jsapi as js;
use mozjs::jsval as val;
use mozjs::rust::{Runtime, Trace, SIMPLE_GLOBAL_CLASS};

use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;
use std::str;

use std::rc::{Rc, Weak};

use mozjs::jsapi::{Heap, JSContext, JSObject, JSScript, JSTracer, Value};
use std::collections::BTreeMap;
use std::os;


pub struct Vm {
  pub rt: Runtime,
  pub global: Box<Heap<*mut JSObject>>,
  pub scripts: BTreeMap<String, Rc<Script>>,
}

pub struct Script {
  script: Heap<*mut JSScript>,
}

impl Script {
  pub fn run(&self, vm: &mut Vm) -> Result<Heap<Value>, Heap<Value>> {
    unsafe {
      let cx = vm.rt.cx();
      let _ac = js::JSAutoCompartment::new(cx, vm.global.handle().get());

      let result = Heap::default();
      println!("BEFORE EXEC");
      let res = mozjs::jsapi::JS_ExecuteScript(cx, self.script.handle(), result.handle_mut());
      println!("AFTER EXEC");
      if (!res) {
        let exception = get_exception(cx);
        return Err(exception);
      }

      Ok(result)
    }
  }
}

unsafe fn get_exception(cx: *mut JSContext) -> Heap<Value> {
  let exception = Heap::default();
  assert!(mozjs::jsapi::JS_GetPendingException(
    cx,
    exception.handle_mut()
  ));

  mozjs::jsapi::JS_ClearPendingException(cx);
  exception
}

#[allow(unsafe_code)]
unsafe extern "C" fn visit(tr: *mut JSTracer, data: *mut os::raw::c_void) {
  println!("VISIT TRACE");
  let vm = data as *mut Vm;
  for script in (*vm).scripts.values() {
    script.script.trace(tr);
  }

  (*vm).global.trace(tr);
}

impl Vm {
  pub fn new() -> Box<Vm> {
    unsafe {
      let rt = Runtime::new().unwrap();
      let cx = rt.cx();

      mozjs::jsapi::DisableIncrementalGC(cx);

      let h_option = js::OnNewGlobalHookOption::FireOnNewGlobalHook;
      let c_option = js::CompartmentOptions::default();

      let global = Heap::boxed(js::JS_NewGlobalObject(
        cx,
        &SIMPLE_GLOBAL_CLASS,
        ptr::null_mut(),
        h_option,
        &c_option,
      ));

      let _ac = js::JSAutoCompartment::new(cx, global.handle().get());
      assert!(js::JS_InitStandardClasses(cx, global.handle()));

      js::JS_DefineFunction(
        cx,
        global.handle(),
        b"require\0".as_ptr() as *const libc::c_char,
        Some(require),
        1,
        0,
      );

      let mut vm_box = Box::new(Vm {
        rt,
        global,
        scripts: BTreeMap::new(),
      });

      mozjs::jsapi::JS_AddExtraGCRootsTracer(
        cx,
        Some(visit),
        &mut *vm_box as *mut _ as *mut os::raw::c_void,
      );

      return vm_box;
    }
  }

  pub fn compile(&mut self, filename: &str, code: &str) -> Rc<Script> {
    unsafe {
      let cx = self.rt.cx();
      let _ac = js::JSAutoCompartment::new(cx, self.global.get());

      let code: Vec<u16> = code.encode_utf16().collect();
      let mut script = Heap::default();

      let cfilename = CString::new(filename).unwrap();
      let options = mozjs::rust::CompileOptionsWrapper::new(cx, cfilename.as_ptr(), 1);
      let mut source = mozjs::jsapi::SourceBufferHolder {
        data_: code.as_ptr(),
        length_: code.len() as libc::size_t,
        ownsChars_: false,
      };

      println!("Compiling");

      let res = mozjs::jsapi::Compile(cx, options.ptr, &mut source, script.handle_mut());
      if (!res) {
        println!("Error {}", res);
      }

      let script = Rc::new(Script { script });
      self.scripts.insert(filename.to_string(), script.clone());

      script
    }
  }
}

unsafe extern "C" fn require(context: *mut js::JSContext, argc: u32, vp: *mut js::Value) -> bool {
  let args = js::CallArgs::from_vp(vp, argc);
  info!("Require {:?}", args);
  // args.rval().set(val::Int32Value(2));
  return true;
}
