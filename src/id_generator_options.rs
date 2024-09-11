 pub struct IdGeneratorOptions {
    pub method: u16,                // 雪花计算方法,（1-漂移算法|2-传统算法），默认1
    pub base_time: i64,             // 基础时间（ms单位），不能超过当前系统时间
    pub worker_id: u16,             // 机器码，必须由外部设定，最大值 2^WorkerIdBitLength-1
    pub worker_id_bit_length: u8,   // 机器码位长，默认值6，取值范围 [1, 15]（要求：序列数位长+机器码位长不超过22）
    pub seq_bit_length: u8,         // 序列数位长，默认值6，取值范围 [3, 21]（要求：序列数位长+机器码位长不超过22）
    pub max_seq_number: u32,        // 最大序列数（含），设置范围 [MinSeqNumber, 2^SeqBitLength-1]，默认值0，表示最大序列数取最大值（2^SeqBitLength-1]）
    pub min_seq_number: u32,        // 最小序列数（含），默认值5，取值范围 [5, MaxSeqNumber]，每毫秒的前5个序列数对应编号0-4是保留位，其中1-4是时间回拨相应预留位，0是手工新值预留位
    pub top_over_cost_count: u32,   // 最大漂移次数（含），默认2000，推荐范围500-10000（与计算能力有关）
}

impl IdGeneratorOptions {
    pub fn new(worker_id: u16) -> Self {
        IdGeneratorOptions {
            method: 1,
            base_time: 1582136402000,
            worker_id,
            worker_id_bit_length: 6,
            seq_bit_length: 6,
            max_seq_number: 0,
            min_seq_number: 5,
            top_over_cost_count: 2000,
        }
    }
}
 