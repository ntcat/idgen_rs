use std::sync::MutexGuard;
use crate::id_generator_options::IdGeneratorOptions;
use crate::isnow_worker::ISnowWorker;
use crate::snow_worker_m1::SnowWorkerM1;

pub struct SnowWorkerM2 {
    base: SnowWorkerM1,
}

impl SnowWorkerM2 {
    pub fn new(options: &IdGeneratorOptions) -> Self {
        SnowWorkerM2 {
            base: SnowWorkerM1::new(options),
        }
    }

    pub fn next_id(&mut self) -> i64 {
        let _lock: MutexGuard<()> = self.base.lock.lock().unwrap();
        let mut current_time_tick = self.base.get_current_time_tick();
        if self.base.last_time_tick == current_time_tick {
            self.base.current_seq_number += 1;
            if self.base.current_seq_number > self.base.max_seq_number {
                self.base.current_seq_number = self.base.min_seq_number;
                current_time_tick = self.base.get_next_time_tick();
            }
        } else {
            self.base.current_seq_number = self.base.min_seq_number;
        }
        if current_time_tick < self.base.last_time_tick {
            println!(
                "Time error for {} milliseconds",
                self.base.last_time_tick - current_time_tick
            );
        }
        self.base.last_time_tick = current_time_tick;
        
        (current_time_tick << self.base.timestamp_shift)
            + (self.base.worker_id << self.base.seq_bit_length) as i64
            + self.base.current_seq_number as i64
    }
}


impl ISnowWorker for SnowWorkerM2 {
    fn next_id(&mut self) -> i64 {
        SnowWorkerM2::next_id(self)
    }
}