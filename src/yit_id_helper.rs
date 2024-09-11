use crate::default_id_generator::DefaultIdGenerator;
use crate::id_generator_options::IdGeneratorOptions;
use std::sync::Mutex;
use std::time::SystemTime;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref ID_GENERATOR: Mutex<DefaultIdGenerator> = Mutex::new(DefaultIdGenerator::default());
}
/// set the options of id_generator.
///
/// # Examples
///
/// ```
/// use idgen_rs::{id_generator_options::IdGeneratorOptions, yit_id_helper};
/// 
/// let mut options = IdGeneratorOptions::new(1);
/// options.worker_id_bit_length = 10;
/// options.worker_id = 1024;
/// options.seq_bit_length = 14;
/// options.max_seq_number = 16383;
/// 
/// yit_id_helper::set_id_generator(options);
///
/// ```
pub fn set_id_generator(options: IdGeneratorOptions) {
    let mut generator = ID_GENERATOR.lock().unwrap();
    *generator = DefaultIdGenerator::new(options);
}

/// generate next id.
///
/// # Examples
///
/// ```
///let new_id = yit_id_helper::next_id();
/// println!("new_id: {}", new_id);
/// 
/// ```
pub fn next_id() -> i64 {
    let mut generator = ID_GENERATOR.lock().unwrap();
    generator.new_long() // D_GENERATOR是通过lazy_static初始化的，不可能为空值。
}

/// extract the timestamp from id.
///
/// # Examples
///
/// ```
///let time = yit_id_helper::extract_time(new_id);
/// 
/// ```
pub fn extract_time(id: i64) -> SystemTime {
    let generator = ID_GENERATOR.lock().unwrap();
    generator.extract_time(id)
}
 