pub mod config;
pub mod error;
pub mod kv;

// trait Engine {
//     fn get(&self, key: String) -> Option<KeyValue>;

//     fn set(&mut self, key: String, value: String) -> Result<()>;

//     fn delete(&mut self, key: String) -> Result<()>;

//     fn compact(&mut self) -> Result<Self>;
// }
