use config::{Config, File, FileFormat};

pub fn init() -> Config {
    let mut conf = Config::new();
    conf.merge(File::from_str("Wiki.yml", FileFormat::Yaml)).unwrap();
    conf
}