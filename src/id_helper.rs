use crate::generator::IdGenerator;
use crate::options::IGOptions;
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref ID_GENERATOR: Mutex<IdGenerator> = Mutex::new(IdGenerator::default());
}
/// set the options of id_generator.
///
/// # Examples
///
/// ```
/// use idgen_rs::{id_generator_options::IGOptions, yit_id_helper};
/// 
/// let mut options = IGOptions::new(1);
/// options.worker_id_bit_length = 10;
/// options.worker_id = 1024;
/// options.seq_bit_length = 14;
/// options.max_seq_number = 16383;
/// 
/// yit_id_helper::set_options(options);
///
/// ```
pub fn set_options(options: IGOptions) {
    let mut generator = ID_GENERATOR.lock().unwrap();
    *generator = IdGenerator::new(options);
}
/// get the options of id_generator.
///
/// # Examples
///
/// ```
/// use idgen_rs::{id_generator_options::IGOptions, yit_id_helper};
/// 
/// let mut options = yit_id_helper::get_options();
/// 
///
/// ```
pub fn get_options() -> IGOptions {
    let generator: MutexGuard<IdGenerator> = ID_GENERATOR.lock().unwrap();
    generator.options.clone()
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
 