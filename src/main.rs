#[macro_use]
extern crate mozjs;

#[macro_use]
extern crate slog;
extern crate slog_json;

extern crate libc;

use mozjs::glue;
use mozjs::jsapi as js;
use mozjs::jsval as val;
use mozjs::rust::{Runtime, SIMPLE_GLOBAL_CLASS};
use rocksdb::DB;
use std::ffi::CStr;
use std::ptr;
use std::str;

use slog::Drain;
use std::ffi::CString;

fn init_logger() -> slog::Logger {
   let decorator = slog_term::TermDecorator::new().build();
   let drain = slog_term::CompactFormat::new(decorator).build().fuse();
   let drain = slog_async::Async::new(drain).build().fuse();

   let log = slog::Logger::root(drain, o!());
   info!(log, "Logging ready!"; "10" => 2);

   log
}

fn main() {
   let log = init_logger();

   // NB: db is automatically closed at end of lifetime
   let db = DB::open_default("./storage").unwrap();
   db.put(b"my key", b"my value");

   match db.get(b"my key") {
      Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
      Ok(None) => println!("value not found"),
      Err(e) => println!("operational problem encountered: {}", e),
   }

   db.delete(b"my key").unwrap();

   unsafe {
      let rt = Runtime::new().unwrap();
      let cx = rt.cx();

      let h_option = js::OnNewGlobalHookOption::FireOnNewGlobalHook;
      let c_option = js::CompartmentOptions::default();

      let global = js::JS_NewGlobalObject(
         cx,
         &SIMPLE_GLOBAL_CLASS,
         ptr::null_mut(),
         h_option,
         &c_option,
      );

      rooted!(in(cx) let global_root = global);
      let global = global_root.handle();

      let _ac = js::JSAutoCompartment::new(cx, global.get());
      assert!(js::JS_InitStandardClasses(cx, global.into()));

      let resolve = js::JS_DefineFunction(
         cx,
         global.into(),
         b"resolve\0".as_ptr() as *const libc::c_char,
         Some(resolve),
         1,
         0,
      );

      rooted!(in(cx) let mut resolve = resolve);
      mozjs::rust::wrappers::SetModuleResolveHook(cx, resolve.handle());

      let code: Vec<u16> = "export default () => 2".encode_utf16().collect();

      rooted!(in(cx) let mut script = ptr::null_mut::<js::JSObject>());

      let filename = CString::new("script.js").unwrap();
      let options = mozjs::rust::CompileOptionsWrapper::new(cx, filename.as_ptr(), 1);
      let mut source = mozjs::jsapi::SourceBufferHolder {
         data_: code.as_ptr(),
         length_: code.len() as libc::size_t,
         ownsChars_: false,
      };

      let res =
         mozjs::rust::wrappers::CompileModule(cx, options.ptr, &mut source, script.handle_mut());

      mozjs::rust::wrappers::ModuleInstantiate(cx, script.handle());
      mozjs::rust::wrappers::ModuleEvaluate(cx, script.handle());
      rooted!(in(cx) let mut scr = mozjs::rust::wrappers::GetModuleScript(script.handle()));

      info!(log, "CompileModule"; "result" => res);

      let ids = mozjs::rust::IdVector::new(cx);
      mozjs::rust::wrappers::GetPropertyKeys(
         cx,
         scr.handle(),
         mozjs::jsapi::JSITER_OWNONLY | mozjs::jsapi::JSITER_HIDDEN | mozjs::jsapi::JSITER_SYMBOLS,
         ids.get(),
      );

      println!("{:?}", ids.len());
      for id in &*ids {
         println!("{:?}", id);
      }

      rooted!(in(cx) let mut default = val::UndefinedValue());

      {
         let ok = mozjs::rust::wrappers::JS_GetProperty(
            cx,
            script.handle(),
            b"default\0".as_ptr() as *const libc::c_char,
            default.handle_mut(),
         );

         if !ok {
            error!(log, "Cannot get a default function")
         }
      }

      assert!(default.is_object());

      rooted!(in(cx) let mut this = js::JS_NewPlainObject(cx));
      rooted!(in(cx) let mut res = val::UndefinedValue());

      let args = mozjs::jsapi::HandleValueArray::new();

      {
         let ret = mozjs::rust::wrappers::JS_CallFunctionValue(
            cx,
            this.handle(),
            default.handle(),
            &args,
            res.handle_mut(),
         );

         if !ret {
            error!(log, "ERROR CALL")
         }
      }

      info!(log, "RESULT"; "res" => res.to_int32());
   }
}

unsafe extern "C" fn resolve(context: *mut js::JSContext, argc: u32, vp: *mut js::Value) -> bool {
   let args = js::CallArgs::from_vp(vp, argc);
   println!("RESOLVE {:?}", args);
   // args.rval().set(val::Int32Value(2));
   return true;
}
