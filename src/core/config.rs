pub struct Config {
    /// Main Database directory that contains data and hints files
    main_db_dir: String,

    /// Data directory
    data_dir: String,

    /// Extension of the data files
    data_extension: String,
}

impl Default for Config {
    fn default() -> Self {
        let main_db_dir = "db".to_string();
        let data_dir = "data".to_string();
        let data_extension = "log".to_string(); // file.log

        Self {
            main_db_dir,
            data_dir,
            data_extension,
        }
    }
}

impl Config {
    pub fn get_glob_path(&self) -> String {
        format!(
            "./{}/{}/*.{}",
            self.main_db_dir, self.data_dir, self.data_extension
        )
    }

    pub fn new_active_file_path(&self, index: usize) -> String {
        format!(
            "./{}/{}/{}.{}",
            self.main_db_dir, self.data_dir, index, self.data_extension
        )
    }
}
