use std::path::PathBuf;

pub struct FileSelectorPrinter;

impl FileSelectorPrinter {
    pub fn new() -> FileSelectorPrinter {
        FileSelectorPrinter
    }

    pub fn print(&self, files: Vec<PathBuf>) {
        for file in files {}
    }
}
