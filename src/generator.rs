use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::options::IGOptions;
use crate::iworker::IWorker;
use crate::worker_m1::WorkerM1;
use crate::worker_m2::WorkerM2;

pub trait IIdGenerator {
    fn new_long(&mut self) -> i64;
}

pub struct IdGenerator {
    pub options: IGOptions,
    pub worker: Box<dyn IWorker>,
}
// 实现 Send 和 Sync
unsafe impl Send for IdGenerator {}
unsafe impl Sync for IdGenerator {}

impl IdGenerator {
    pub fn new(options: IGOptions) -> Self {
        if options.base_time < 631123200000 || options.base_time > current_time_millis() {
            panic!("base_time error.");
        }

        if options.worker_id_bit_length == 0 {
            panic!("worker_id_bit_length error.(range:[1, 21])");
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

        let worker: Box<dyn IWorker> = match options.method {
            1 => Box::new(WorkerM1::new(&options)),
            2 => Box::new(WorkerM2::new(&options)),
            _ => Box::new(WorkerM1::new(&options)),
        };

        if options.method == 1 {
            sleep(Duration::from_micros(500));
        }

        IdGenerator {
            options,
            worker,
        }
    }

    pub fn new_long(&mut self) -> i64 {
        self.worker.next_id()
    }

    pub fn extract_time(&self, id: i64) -> SystemTime {
        UNIX_EPOCH
            + Duration::from_millis(
                (id >> (self.options.worker_id_bit_length + self.options.seq_bit_length)) as u64
                    + self.options.base_time as u64,
            )
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        IdGenerator::new(IGOptions {
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

