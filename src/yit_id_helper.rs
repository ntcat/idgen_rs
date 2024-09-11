use crate::default_id_generator::DefaultIdGenerator;
use crate::id_generator_options::IdGeneratorOptions;
use std::sync::Mutex;
use std::time::SystemTime;
use lazy_static::lazy_static;

// 创建全局实例
lazy_static! {
    pub static ref ID_GENERATOR: Mutex<DefaultIdGenerator> = Mutex::new(DefaultIdGenerator::new(IdGeneratorOptions {
        base_time: 631123200000,
        worker_id_bit_length: 5,
        worker_id: 1,
        seq_bit_length: 12,
        max_seq_number: 4095,
        min_seq_number: 5,
        top_over_cost_count: 2000,
        method: 1,
    }));
}

pub fn set_id_generator(options: IdGeneratorOptions) {
    let mut generator = ID_GENERATOR.lock().unwrap();
    *generator = DefaultIdGenerator::new(options);
}


pub fn next_id() -> i64 {
    let mut generator = ID_GENERATOR.lock().unwrap();
    generator.new_long() // D_GENERATOR是通过lazy_static初始化的，不可能为空值。
}

pub fn extract_time(id: i64) -> SystemTime {
    let generator = ID_GENERATOR.lock().unwrap();
    generator.extract_time(id)
}
 