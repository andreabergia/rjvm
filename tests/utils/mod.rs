use rjvm::class_file::ClassFile;
use rjvm::class_reader;
use std::path::PathBuf;
use tracing::{info, Level};

pub fn read_class_from_file(file: &str) -> ClassFile {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/rjvm");
    path.push(String::from(file) + ".class");
    info!("reading class from file: {}", path.display());

    class_reader::read(path.as_path()).unwrap()
}

pub fn setup_tracing() {
    let format = tracing_subscriber::fmt::format().pretty();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .event_format(format)
        .init();
}
