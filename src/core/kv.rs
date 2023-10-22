impl KiviStore {
    fn new() -> Self {
        let mut mem_index = BTreeMap::new();

        // get read-only stale files
        let stale_file_list = get_files();
        let new_active_file_index = stale_file_list
            .last()
            .and_then(|x| x.file_stem())
            .and_then(|x| x.to_str())
            .and_then(|x| x.parse::<usize>().ok())
            .unwrap()
            + 1;

        println!("new active file idx: {:?}", new_active_file_index);

        let mut stale_files = Vec::new();

        for item in stale_file_list {
            let file_d = OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(item)
                .unwrap();

            stale_files.push(file_d);
        }

        let active_file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            // TODO: Same shit, move to config
            .open(format!("{}.log", new_active_file_index.to_string()))
            .unwrap();

        build_keydir(&active_file, &mut mem_index);

        println!("Stale files: {:?}", stale_files);

        Self {
            mem_index,
            active_file,
            stale_files,
        }
    }

    fn get(&self, key: String) {
        // println!("Doing KiviStore GET. Key: {}", key);
        let get_by_key = self.mem_index.get(&key);
        if let Some(val) = get_by_key {
            println!("KiviStore GET: Got {:?}", val);
        } else {
            println!("KiviStore GET: not found");
        }
    }

    fn set(&mut self, key: String, value: String) {
        println!("Doing KiviStore SET. Key: {}, Value: {}", key, value);

        let set_com = KiviCommand::Set { key, value };

        let j = serde_json::to_string(&set_com).unwrap();

        self.active_file.write_all(j.as_bytes()).unwrap();
        // self.mem_index.insert(key, value);
    }

    fn compact(&mut self) {
        unimplemented!();
    }
}
