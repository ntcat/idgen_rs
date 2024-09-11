use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::id_generator_options::IdGeneratorOptions;
use crate::isnow_worker::ISnowWorker;
use crate::snow_worker_m1::SnowWorkerM1;
use crate::snow_worker_m2::SnowWorkerM2;

pub trait IIdGenerator {
    fn new_long(&mut self) -> i64;
}

pub struct DefaultIdGenerator {
    pub options: IdGeneratorOptions,
    pub snow_worker: Box<dyn ISnowWorker>,
}
// 实现 Send 和 Sync
unsafe impl Send for DefaultIdGenerator {}
unsafe impl Sync for DefaultIdGenerator {}

impl DefaultIdGenerator {
    pub fn new(options: IdGeneratorOptions) -> Self {
        if options.base_time < 631123200000 || options.base_time > current_time_millis() {
            panic!("BaseTime error.");
        }

        if options.worker_id_bit_length == 0 {
            panic!("WorkerIdBitLength error.(range:[1, 21])");
        }
        if options.worker_id_bit_length + options.seq_bit_length > 22 {
            panic!("worker_id_bit_length + seq_bit_length <= 22");
        }

        let max_worker_id_number = (1 << options.worker_id_bit_length) - 1;
        if options.worker_id > max_worker_id_number {
            panic!("worker_id error. (range:[0, {}])", max_worker_id_number);
        }

        if options.seq_bit_length < 2 || options.seq_bit_length > 21 {
            panic!("seq_bit_length error. (range:[2, 21])");
        }

        let max_seq_number = (1 << options.seq_bit_length) - 1;
        if options.max_seq_number > max_seq_number {
            panic!("max_seq_number error. (range:[1, {}])", max_seq_number);
        }

        if options.min_seq_number < 5 || options.min_seq_number > max_seq_number {
            panic!("min_seq_number error. (range:[5, {}])", max_seq_number);
        }

        if options.top_over_cost_count > 10000 {
            panic!("top_over_cost_count error. (range:[0, 10000])");
        }

        let snow_worker: Box<dyn ISnowWorker> = match options.method {
            1 => Box::new(SnowWorkerM1::new(&options)),
            2 => Box::new(SnowWorkerM2::new(&options)),
            _ => Box::new(SnowWorkerM1::new(&options)),
        };

        if options.method == 1 {
            sleep(Duration::from_micros(500));
        }

        DefaultIdGenerator {
            options,
            snow_worker,
        }
    }

    pub fn new_long(&mut self) -> i64 {
        self.snow_worker.next_id()
    }

    pub fn extract_time(&self, id: i64) -> SystemTime {
        UNIX_EPOCH
            + Duration::from_millis(
                (id >> (self.options.worker_id_bit_length + self.options.seq_bit_length)) as u64
                    + self.options.base_time as u64,
            )
    }
}

impl Default for DefaultIdGenerator {
    fn default() -> Self {
        DefaultIdGenerator::new(IdGeneratorOptions {
            base_time: 631123200000,
            worker_id_bit_length: 6,
            worker_id: 1,
            seq_bit_length: 6,
            max_seq_number: 63,
            min_seq_number: 5,
            top_over_cost_count: 2000,
            method: 1,
        })
    }
}

fn current_time_millis() -> i64 {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_epoch.as_millis() as i64
}

