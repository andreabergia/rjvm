use crate::class_path_entry::ClassPathEntry;

#[allow(dead_code)]
pub struct ClassPath {
    entries: Vec<Box<dyn ClassPathEntry>>,
}

impl ClassPath {
    #[allow(dead_code)]
    pub fn push(&mut self, entry: Box<dyn ClassPathEntry>) {
        self.entries.push(entry)
    }
}
