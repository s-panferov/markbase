use mozjs::jsapi as js;
use mozjs::rooted;
use mozjs::rust::{Runtime, Trace, SIMPLE_GLOBAL_CLASS};

use std::ffi::CString;
use std::ptr;
use std::slice::from_raw_parts;
use std::str;

use std::rc::Rc;

use libc::c_uint;
use mozjs::jsapi::{Heap, JSContext, JSObject, JSTracer, Value};
use std::collections::BTreeMap;
use std::os;

use wiki_log::prelude::*;

use mozjs::conversions::FromJSValConvertible;
use mozjs::conversions::ToJSValConvertible;

pub struct Vm {
  pub funcs: BTreeMap<String, Rc<Func>>,
  pub global: Box<Heap<*mut JSObject>>,
  pub rt: Runtime,
}

pub struct Func {
  value: Heap<Value>,
}

impl Func {
  pub fn call_string(&self, vm: &mut Vm, value: String) -> Result<String, ErrorInfo> {
    unsafe {
      let cx = vm.rt.cx();
      let _ac = js::JSAutoCompartment::new(cx, vm.global.handle().get());

      let result = Heap::default();

      rooted!(in(cx) let mut source_string = mozjs::jsval::UndefinedValue());
      rooted!(in(cx) let this = mozjs::jsapi::JS_NewPlainObject(cx));

      value.to_jsval(cx, source_string.handle_mut());

      let mut vec = Vec::new();
      vec.push(source_string.get());

      let argv = mozjs::jsapi::HandleValueArray::from_rooted_slice(vec.as_slice());
      let res = mozjs::rust::wrappers::JS_CallFunctionValue(
        cx,
        this.handle(),
        mozjs::rust::Handle::from_raw(self.value.handle()),
        &argv,
        mozjs::rust::MutableHandle::from_raw(result.handle_mut()),
      );

      if !res {
        let exception = get_exception(cx);
        return Err(exception);
      }

      Ok(
        String::from_jsval(cx, mozjs::rust::Handle::from_raw(result.handle()), ())
          .unwrap()
          .get_success_value()
          .unwrap()
          .to_string(),
      )
    }
  }
}

/// A struct encapsulating information about a runtime script error.
#[derive(Debug)]
pub struct ErrorInfo {
  /// The error message.
  pub message: String,
  /// The file name.
  pub filename: String,
  /// The line number.
  pub lineno: c_uint,
  /// The column number.
  pub column: c_uint,
}

impl ErrorInfo {
  unsafe fn from_native_error(
    cx: *mut mozjs::jsapi::JSContext,
    object: mozjs::rust::HandleObject,
  ) -> Option<ErrorInfo> {
    let report = mozjs::rust::wrappers::JS_ErrorFromException(cx, object);
    if report.is_null() {
      return None;
    }

    let filename = {
      let filename = (*report)._base.filename as *const u8;
      if !filename.is_null() {
        let length = (0..).find(|idx| *filename.offset(*idx) == 0).unwrap();
        let filename = from_raw_parts(filename, length as usize);
        String::from_utf8_lossy(filename).into_owned()
      } else {
        "none".to_string()
      }
    };

    let lineno = (*report)._base.lineno;
    let column = (*report)._base.column;

    let message = {
      let message = (*report)._base.message_.data_ as *const u8;
      let length = (0..).find(|idx| *message.offset(*idx) == 0).unwrap();
      let message = from_raw_parts(message, length as usize);
      String::from_utf8_lossy(message).into_owned()
    };

    Some(ErrorInfo {
      filename: filename,
      message: message,
      lineno: lineno,
      column: column,
    })
  }
}

unsafe fn get_exception(cx: *mut JSContext) -> ErrorInfo {
  rooted!(in(cx) let mut exception = mozjs::jsval::UndefinedValue());
  assert!(mozjs::rust::wrappers::JS_GetPendingException(
    cx,
    exception.handle_mut()
  ));

  mozjs::jsapi::JS_ClearPendingException(cx);

  rooted!(in(cx) let mut exception = exception.to_object());
  ErrorInfo::from_native_error(cx, exception.handle()).unwrap()
}

#[allow(unsafe_code)]
unsafe extern "C" fn visit(tr: *mut JSTracer, data: *mut os::raw::c_void) {
  println!("VISIT TRACE");
  let vm = data as *mut Vm;
  for func in (*vm).funcs.values() {
    func.value.trace(tr);
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
        funcs: BTreeMap::new(),
      });

      mozjs::jsapi::JS_AddExtraGCRootsTracer(
        cx,
        Some(visit),
        &mut *vm_box as *mut _ as *mut os::raw::c_void,
      );

      return vm_box;
    }
  }

  pub fn compile(&mut self, filename: &str, code: &str) -> Result<Rc<Func>, ErrorInfo> {
    unsafe {
      let cx = self.rt.cx();
      let _ac = js::JSAutoCompartment::new(cx, self.global.get());

      let code: Vec<u16> = code.encode_utf16().collect();
      let script = Heap::default();

      let cfilename = CString::new(filename).unwrap();
      let options = mozjs::rust::CompileOptionsWrapper::new(cx, cfilename.as_ptr(), 1);
      let mut source = mozjs::jsapi::SourceBufferHolder {
        data_: code.as_ptr(),
        length_: code.len() as libc::size_t,
        ownsChars_: false,
      };

      println!("Compiling");

      let res = mozjs::jsapi::Compile(cx, options.ptr, &mut source, script.handle_mut());
      if !res {
        println!("Error {}", res);
      }

      let func = Heap::default();
      let res = mozjs::jsapi::JS_ExecuteScript(cx, script.handle(), func.handle_mut());
      if !res {
        let exception = get_exception(cx);
        return Err(exception);
      }

      // assert!(func.get().is_object());

      let func = Rc::new(Func { value: func });
      self.funcs.insert(filename.to_string(), func.clone());

      Ok(func)
    }
  }
}

impl Drop for Vm {
  fn drop(&mut self) {}
}

unsafe extern "C" fn require(context: *mut js::JSContext, argc: u32, vp: *mut js::Value) -> bool {
  let args = js::CallArgs::from_vp(vp, argc);
  info!("Require {:?}", args);
  // args.rval().set(val::Int32Value(2));
  return true;
}
