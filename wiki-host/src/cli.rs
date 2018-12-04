use clap::{App, Arg, ArgMatches};

pub fn init() -> Box<ArgMatches<'static>> {
  let app = App::new("Wiki")
    .version("1.0")
    .author("Stanislav Panferov <fnight.m@gmail.com>")
    .about("Fast and simple wiki generator")
    .arg(
      Arg::with_name("input")
        .short("i")
        .long("input")
        .default_value(".")
        .help("Root wiki folder")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("storage")
        .short("s")
        .long("storage")
        .default_value("storage")
        .help("Compiled wiki storage")
        .takes_value(true),
    );

  return Box::new(app.get_matches());
}
