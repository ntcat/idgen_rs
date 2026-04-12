/// 默认基准时间：2020-02-20 02:00:02 UTC (2020年2月20日)
/// 选择此时间确保有足够的时间戳空间，同时接近库发布的日期
pub const DEFAULT_BASE_TIME: i64 = 1582136402000;

#[derive(Debug, Clone)]
pub struct IGOptions {
    pub method: u16,                // 雪花计算方法,（1-漂移算法|2-传统算法），默认1
    pub base_time: i64,             // 基础时间（ms单位），不能超过当前系统时间
    pub worker_id: u16,             // 机器码，必须由外部设定，最大值 2^WorkerIdBitLength-1
    pub worker_id_bit_length: u8,   // 机器码位长，默认值6，取值范围 [1, 15]（要求：序列数位长+机器码位长不超过22）
    pub seq_bit_length: u8,         // 序列数位长，默认值6，取值范围 [3, 21]（要求：序列数位长+机器码位长不超过22）
    pub max_seq_number: u32,        // 最大序列数（含），设置范围 [MinSeqNumber, 2^SeqBitLength-1]，默认值0，表示最大序列数取最大值（2^SeqBitLength-1]）
    pub min_seq_number: u32,        // 最小序列数（含），默认值5，取值范围 [5, MaxSeqNumber]，每毫秒的前5个序列数对应编号0-4是保留位，其中1-4是时间回拨相应预留位，0是手工新值预留位
    pub top_over_cost_count: u32,   // 最大漂移次数（含），默认2000，推荐范围500-10000（与计算能力有关）
}

impl IGOptions {
    pub fn new(worker_id: u16) -> Self {
        IGOptions {
            method: 1,
            base_time: DEFAULT_BASE_TIME,
            worker_id,
            worker_id_bit_length: 6,
            seq_bit_length: 6,
            max_seq_number: 0,
            min_seq_number: 5,
            top_over_cost_count: 2000,
        }
    }

    /// Builder 模式构建 IGOptions
    pub fn builder(worker_id: u16) -> IGOptionsBuilder {
        IGOptionsBuilder::new(worker_id)
    }

    /// 便捷快速初始化（使用合理默认值）
    pub fn quick_init(worker_id: u16) -> Self {
        Self::new(worker_id)
    }

    /// 根据业务容量自动计算位长
    ///
    /// # Arguments
    /// * `worker_id` - 当前节点 ID (0~65535)
    /// * `max_nodes` - 集群最大节点数
    /// * `max_qps`   - 目标最大 QPS
    ///
    /// # Examples
    ///
    /// ```
    /// use idgen_rs::options::IGOptions;
    ///
    /// // 1024 节点集群，5万 QPS
    /// let opts = IGOptions::with_capacity(1, 1024, 50_000);
    /// ```
    pub fn with_capacity(worker_id: u16, max_nodes: u32, max_qps: u32) -> Self {
        // 1. 计算 Worker Bits (精确向上取整)
        let worker_id_bit_length = if max_nodes <= 1 {
            1
        } else {
            (32 - max_nodes.leading_zeros()) as u8
        };

        // 2. 计算 Seq Bits (QPS 转 每毫秒需求 + 20% 缓冲)
        let ids_per_ms = ((max_qps as f64 / 1000.0).ceil() as u32).max(1);
        let seq_bits_raw = (32 - (ids_per_ms + ids_per_ms / 5).leading_zeros()) as u8;

        // 3. 严格约束总位长 ≤ 22，且 seq ≥ 3
        let total = worker_id_bit_length + seq_bits_raw;
        let seq_bit_length = if total > 22 {
            (22 - worker_id_bit_length).max(3)
        } else {
            seq_bits_raw.max(3)
        };

        Self {
            method: 1,
            base_time: DEFAULT_BASE_TIME,
            worker_id,
            worker_id_bit_length,
            seq_bit_length,
            max_seq_number: 0,
            min_seq_number: 5,
            top_over_cost_count: 2000,
        }
    }
}

/// IGOptions 构建器
#[derive(Debug, Clone)]
pub struct IGOptionsBuilder {
    inner: IGOptions,
}

impl IGOptionsBuilder {
    pub fn new(worker_id: u16) -> Self {
        Self {
            inner: IGOptions::new(worker_id),
        }
    }

    /// 设置雪花计算方法 (1-漂移算法|2-传统算法)
    pub fn method(mut self, method: u16) -> Self {
        self.inner.method = method;
        self
    }

    /// 设置基础时间（毫秒时间戳）
    pub fn base_time_ms(mut self, base_time: i64) -> Self {
        self.inner.base_time = base_time;
        self
    }

    /// 设置基础时间（使用 DateTime）
    pub fn base_time(mut self, base_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.inner.base_time = base_time.timestamp_millis();
        self
    }

    /// 设置机器码位长 (范围: [1, 15])
    pub fn worker_id_bit_length(mut self, length: u8) -> Self {
        self.inner.worker_id_bit_length = length;
        self
    }

    /// 设置序列数位长 (范围: [3, 21])
    pub fn seq_bit_length(mut self, length: u8) -> Self {
        self.inner.seq_bit_length = length;
        self
    }

    /// 设置最大序列数
    pub fn max_seq_number(mut self, max: u32) -> Self {
        self.inner.max_seq_number = max;
        self
    }

    /// 设置最小序列数
    pub fn min_seq_number(mut self, min: u32) -> Self {
        self.inner.min_seq_number = min;
        self
    }

    /// 设置最大漂移次数
    pub fn top_over_cost_count(mut self, count: u32) -> Self {
        self.inner.top_over_cost_count = count;
        self
    }

    /// 构建 IGOptions
    pub fn build(self) -> IGOptions {
        self.inner
    }
}
 