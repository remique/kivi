use std::path::PathBuf;

pub struct Config {
    /// Main Database directory that contains data and hints files
    db_path: PathBuf,

    /// Data directory
    data_dir: String,

    /// Extension of the data files
    data_extension: String,

    /// Temporary data directory that is used for data compaction
    temp_data_dir: String,
}

pub struct ConfigBuilder {
    db_path: PathBuf,
    data_dir: String,
    data_extension: String,
    temp_data_dir: String,
}

impl ConfigBuilder {
    pub fn set_db_path(&mut self, p: PathBuf) -> &mut Self {
        self.db_path = p;
        self
    }

    pub fn set_data_dir(&mut self, dd: String) -> &mut Self {
        self.data_dir = dd;
        self
    }

    pub fn set_data_extension(&mut self, de: String) -> &mut Self {
        self.data_extension = de;
        self
    }

    pub fn set_temp_data_dir(&mut self, tdd: String) -> &mut Self {
        self.temp_data_dir = tdd;
        self
    }

    pub fn build(&mut self) -> Config {
        Config {
            db_path: self.db_path.clone(),
            data_dir: self.data_dir.clone(),
            data_extension: self.data_extension.clone(),
            temp_data_dir: self.temp_data_dir.clone(),
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        let db_path = PathBuf::from("./db");
        let data_dir = "data".to_string();
        let data_extension = "log".to_string(); // file.log
        let temp_data_dir = "temp".to_string();

        Self {
            db_path,
            data_dir,
            data_extension,
            temp_data_dir,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        ConfigBuilder::default().build()
    }
}

impl Config {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn get_glob_pattern(&self) -> String {
        format!(
            "{}/{}/[0-9]*.{}",
            &self.db_path.to_str().unwrap(),
            self.data_dir,
            self.data_extension
        )
    }

    pub fn new_active_file_path(&self, index: usize) -> String {
        format!(
            "{}/{}/{}.{}",
            &self.db_path.to_str().unwrap(),
            self.data_dir,
            index,
            self.data_extension
        )
    }

    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    pub fn get_data_dir(&self) -> &String {
        &self.data_dir
    }

    pub fn get_data_extension(&self) -> &String {
        &self.data_extension
    }

    pub fn get_temp_data_dir(&self) -> &String {
        &self.temp_data_dir
    }

    pub fn get_full_path(&self) -> String {
        format!("{}/{}", &self.db_path.to_str().unwrap(), self.data_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = Config::default();
        let p = PathBuf::from("./db");

        assert_eq!(c.db_path, p);
        assert_eq!(c.get_glob_pattern(), String::from("./db/data/[0-9]*.log"));
        assert_eq!(c.new_active_file_path(1), String::from("./db/data/1.log"));
        assert_eq!(c.get_full_path(), String::from("./db/data"))
    }

    #[test]
    fn test_builder_default() {
        // Equivalent to Config::default()
        let c = Config::new().build();
        let p = PathBuf::from("./db");

        assert_eq!(c.db_path, p);
        assert_eq!(c.get_glob_pattern(), String::from("./db/data/[0-9]*.log"));
        assert_eq!(c.new_active_file_path(1), String::from("./db/data/1.log"));
        assert_eq!(c.get_full_path(), String::from("./db/data"))
    }

    #[test]
    fn test_builder_custom() {
        let c = Config::new()
            .set_db_path(PathBuf::from("/var/folders/h_/abc"))
            .set_data_dir(String::from("ddd"))
            .set_data_extension(String::from("filez"))
            .build();

        assert_eq!(c.temp_data_dir, String::from("temp"));
        assert_eq!(
            c.get_glob_pattern(),
            String::from("/var/folders/h_/abc/ddd/[0-9]*.filez")
        );
        assert_eq!(
            c.new_active_file_path(1),
            String::from("/var/folders/h_/abc/ddd/1.filez")
        );
        assert_eq!(c.get_full_path(), String::from("/var/folders/h_/abc/ddd"))
    }
}
