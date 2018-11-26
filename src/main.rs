#[macro_use]
extern crate mozjs;

#[macro_use(
   slog_o,
   slog_info,
   slog_log,
   slog_record,
   slog_record_static,
   slog_b,
   slog_kv
)]
extern crate slog;
extern crate slog_json;

#[macro_use]
extern crate slog_scope;

extern crate libc;

extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate oxide_auth;
extern crate url;

use rocksdb::DB;
use slog::Drain;

mod js;

fn init_logger() -> (slog::Logger) {
   let decorator = slog_term::TermDecorator::new().build();
   let drain = slog_term::CompactFormat::new(decorator).build().fuse();
   let drain = slog_async::Async::new(drain).build().fuse();
   let log = slog::Logger::root(drain, slog_o!());
   log
}

fn main() {
   let log = init_logger();
   let _guard = slog_scope::set_global_logger(log);

   // NB: db is automatically closed at end of lifetime
   let db = DB::open_default("./storage").unwrap();
   db.put(b"my key", b"my value");

   let mut vm = js::Vm::new();
   let mut script = vm.compile("test-script.js", "get('my_key')");

   println!(">>>> AFTER COMPILE");

   let value = script.run(&mut vm);
   assert!(value.is_ok());

   println!(">>>> END");
}
