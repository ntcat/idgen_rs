use crate::id_generator_options::IdGeneratorOptions;
use crate::isnow_worker::ISnowWorker;
use crate::over_cost_action_arg::OverCostActionArg;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct SnowWorkerM1 {
    pub  base_time: i64,
    pub  worker_id: u16,
    pub  worker_id_bit_length: u8,
    pub  seq_bit_length: u8,
    pub  max_seq_number: u32,
    pub  min_seq_number: u32,
    pub  top_over_cost_count: u32,
    pub  timestamp_shift: u8,
    pub  current_seq_number: u32,
    pub  last_time_tick: i64,
    pub  turn_back_time_tick: i64,
    pub  turn_back_index: u8,
    pub  is_over_cost: bool,
    pub  over_cost_count_in_one_term: u32,
    // gen_count_in_one_term: u32,
    // term_index: u32,
    pub lock: Mutex<()>,
}

impl SnowWorkerM1 {
    pub fn new(options: &IdGeneratorOptions) -> Self {
        let base_time = if options.base_time != 0 {
            options.base_time
        } else {
            1582136402000
        };

        let worker_id_bit_length = if options.worker_id_bit_length == 0 {
            6
        } else {
            options.worker_id_bit_length
        };

        let worker_id = options.worker_id;

        let seq_bit_length = if options.seq_bit_length == 0 {
            6
        } else {
            options.seq_bit_length
        };

        let max_seq_number = if options.max_seq_number == 0 {
            (1 << seq_bit_length) - 1
        } else {
            options.max_seq_number
        };

        let min_seq_number = options.min_seq_number;
        let top_over_cost_count = options.top_over_cost_count;
        let timestamp_shift = worker_id_bit_length + seq_bit_length;
        let current_seq_number = min_seq_number;

        SnowWorkerM1 {
            base_time,
            worker_id,
            worker_id_bit_length,
            seq_bit_length,
            max_seq_number,
            min_seq_number,
            top_over_cost_count,
            timestamp_shift,
            current_seq_number,
            last_time_tick: 0,
            turn_back_time_tick: 0,
            turn_back_index: 0,
            is_over_cost: false,
            over_cost_count_in_one_term: 0,
            // gen_count_in_one_term: 0,
            // term_index: 0,
            lock: Mutex::new(()),
        }
    }

    pub fn do_gen_id_action(&self, _arg: &OverCostActionArg) {
        // 实现生成 ID 的逻辑
    }

    pub fn begin_over_cost_action(&self, _use_time_tick: i64) {
        // 实现开始漂移动作的逻辑
    }

    pub fn end_over_cost_action(&self, _use_time_tick: i64) {
        // 实现结束漂移动作的逻辑
    }

    pub fn begin_turn_back_action(&self, _use_time_tick: i64) {
        // 实现开始回拨动作的逻辑
    }

    pub fn end_turn_back_action(&self, _use_time_tick: i64) {
        // 实现结束回拨动作的逻辑
    }

    pub fn next_over_cost_id(&mut self) -> i64 {
        let current_time_tick = self.get_current_time_tick();
        if current_time_tick > self.last_time_tick {
            self.last_time_tick = current_time_tick;
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = false;
            self.over_cost_count_in_one_term = 0;
            return self.calc_id(self.last_time_tick);
        }
        if self.over_cost_count_in_one_term >= self.top_over_cost_count {
            self.last_time_tick = self.get_next_time_tick();
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = false;
            self.over_cost_count_in_one_term = 0;
            return self.calc_id(self.last_time_tick);
        }
        if self.current_seq_number > self.max_seq_number {
            self.last_time_tick += 1;
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = true;
            self.over_cost_count_in_one_term += 1;
            return self.calc_id(self.last_time_tick);
        }
        self.calc_id(self.last_time_tick)
    }

    pub fn next_normal_id(&mut self) -> i64 {
        let current_time_tick = self.get_current_time_tick();
        if current_time_tick < self.last_time_tick {
            if self.turn_back_time_tick < 1 {
                self.turn_back_time_tick = self.last_time_tick - 1;
                self.turn_back_index += 1;
                if self.turn_back_index > 4 {
                    self.turn_back_index = 1;
                }
                self.begin_turn_back_action(self.turn_back_time_tick);
            }
            return self.calc_turn_back_id(self.turn_back_time_tick);
        }
        if self.turn_back_time_tick > 0 {
            self.end_turn_back_action(self.turn_back_time_tick);
            self.turn_back_time_tick = 0;
        }
        if current_time_tick > self.last_time_tick {
            self.last_time_tick = current_time_tick;
            self.current_seq_number = self.min_seq_number;
            return self.calc_id(self.last_time_tick);
        }
        if self.current_seq_number > self.max_seq_number {
            self.begin_over_cost_action(current_time_tick);
            self.last_time_tick += 1;
            self.current_seq_number = self.min_seq_number;
            self.is_over_cost = true;
            self.over_cost_count_in_one_term = 1;
            return self.calc_id(self.last_time_tick);
        }
        self.calc_id(self.last_time_tick)
    }

    pub fn calc_id(&mut self, use_time_tick: i64) -> i64 {
        let result = (use_time_tick << self.timestamp_shift)
            + (self.worker_id << self.seq_bit_length) as i64
            + self.current_seq_number as i64;
        self.current_seq_number += 1;
        result
    }

    pub fn calc_turn_back_id(&mut self, use_time_tick: i64) -> i64 {
        let result = (use_time_tick << self.timestamp_shift)
            + (self.worker_id << self.seq_bit_length) as i64
            + self.turn_back_index as i64;
        self.turn_back_time_tick -= 1;
        result
    }

    pub fn get_current_time_tick(&self) -> i64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;
        millis - self.base_time
    }
    fn next_id(&mut self) -> i64 { 
        if self.is_over_cost {
            self.next_over_cost_id()
        } else {
            self.next_normal_id()
        }
    }
    pub fn get_next_time_tick(&self) -> i64 {
        let mut temp_time_tick = self.get_current_time_tick();
        while temp_time_tick <= self.last_time_tick {
            sleep(Duration::from_millis(1));
            temp_time_tick = self.get_current_time_tick();
        }
        temp_time_tick
    }


}

impl ISnowWorker for SnowWorkerM1 {
    fn next_id(&mut self) -> i64 {
        SnowWorkerM1::next_id(self)
    }
}