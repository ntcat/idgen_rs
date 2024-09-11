pub struct OverCostActionArg {
    pub action_type: i32,
    pub time_tick: i64,
    pub worker_id: u16,
    pub over_cost_count_in_one_term: i32,
    pub gen_count_in_one_term: i32,
    pub term_index: i32,
}

impl OverCostActionArg {
    pub fn new(
        worker_id: u16,
        time_tick: i64,
        action_type: i32,
        over_cost_count_in_one_term: i32,
        gen_count_when_over_cost: i32,
        index: i32,
    ) -> Self {
        OverCostActionArg {
            action_type,
            time_tick,
            worker_id,
            over_cost_count_in_one_term,
            gen_count_in_one_term: gen_count_when_over_cost,
            term_index: index,
        }
    }
}