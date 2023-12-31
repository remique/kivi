pub struct Config {
    /// Main Database directory that contains data and hints files
    main_db_dir: String,

    /// Data directory
    data_dir: String,

    /// Extension of the data files
    data_extension: String,

    /// Temporary data directory that is used for data compaction
    temp_data_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let main_db_dir = "db".to_string();
        let data_dir = "data".to_string();
        let data_extension = "log".to_string(); // file.log
        let temp_data_dir = "temp".to_string();

        Self {
            main_db_dir,
            data_dir,
            data_extension,
            temp_data_dir,
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
