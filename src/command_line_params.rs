use std::{
    fs::{self, File},
    path::PathBuf,
};

use crate::config::Config;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CommandLineParams {
    #[arg(short, long, value_name = "CONFIG_FILE")]
    pub config_file: String,

    #[arg(long, value_name = "SOURCE_FILE")]
    pub source_file: Option<String>,

    #[arg(long, value_name = "SOURCE_FOLDER")]
    pub source_folder: Option<String>,
}

impl CommandLineParams {
    pub fn config(&self) -> Config {
        let file = File::open(&self.config_file).expect("Cannot load config file");
        serde_json::from_reader(file).unwrap()
    }

    pub fn source_folder(&self) -> PathBuf {
        if let Some(ref source_folder) = self.source_folder {
            PathBuf::from(source_folder)
        } else if let Some(ref source_file) = self.source_file {
            let mut path = fs::canonicalize(PathBuf::from(source_file))
                .expect("Failed to get path of source file");
            path.pop();

            path
        } else {
            fs::canonicalize(PathBuf::from("./"))
                .expect("Failed to get absolute path for current dir")
        }
    }
}
