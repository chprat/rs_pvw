extern crate ini;
use ini::Ini;
use std::path::Path;

const SECTION: &str = "General";
const CONFIG_FILE: &str = "pvw.ini";
const PICTURE_PATH_ENTRY: &str = "picPath";
const DATABASE_FILE_ENTRY: &str = "dbPath";

#[derive(Debug)]
pub struct Configuration {
    config_file: String,
    pub database_file: Option<String>,
    pub picture_folder: Option<String>,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration {
            config_file: String::from(CONFIG_FILE),
            database_file: None,
            picture_folder: None,
        }
    }
    pub fn read(&mut self) {
        if !Path::new(&self.config_file).exists() {
            return;
        }
        let conf = Ini::load_from_file(self.config_file.clone()).unwrap();
        if let Some(section) = conf.section(Some(SECTION)) {
            if let Some(value) = section.get(DATABASE_FILE_ENTRY) {
                self.database_file = Some(String::from(value));
            }
            if let Some(value) = section.get(PICTURE_PATH_ENTRY) {
                self.picture_folder = Some(String::from(value));
            }
        }
    }
    pub fn save(&self) {
        let mut conf = Ini::new();
        let mut db_path = String::from("");
        let mut pic_path = String::from("");
        if let Some(value) = self.database_file.clone() {
            db_path = value;
        }
        if let Some(value) = self.picture_folder.clone() {
            pic_path = value;
        }
        conf.with_section(Some("General"))
            .set(DATABASE_FILE_ENTRY, db_path)
            .set(PICTURE_PATH_ENTRY, pic_path);
        conf.write_to_file(self.config_file.clone()).unwrap();
    }
}
