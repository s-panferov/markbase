extern crate slog;
extern crate slog_json;

use slog::slog_o;

pub mod prelude {
   pub use slog::crit;
   pub use slog::debug;
   pub use slog::info;
   pub use slog::error;
   pub use slog::slog_b;
   pub use slog::slog_info;
   pub use slog::slog_kv;
   pub use slog::slog_log;
   pub use slog::slog_o;
   pub use slog::slog_record;
   pub use slog::slog_record_static;
   pub use slog::trace;
   pub use slog::warn;
}

use slog::Drain;
pub use slog::Logger;

pub fn init() -> (slog::Logger) {
   let decorator = slog_term::TermDecorator::new().build();
   let drain = slog_term::CompactFormat::new(decorator).build().fuse();
   let drain = slog_async::Async::new(drain).build().fuse();
   let log = slog::Logger::root(drain, slog_o!());
   log
}
