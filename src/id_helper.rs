use crate::options::IGOptions;
use crate::fast_generator::FastIdGenerator;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::OnceLock;

/// 全局 ID 生成器，使用 OnceLock 保证仅初始化一次
/// FastIdGenerator 内部已实现 Sync，无需外部加锁
static ID_GENERATOR: OnceLock<FastIdGenerator> = OnceLock::new();

fn get_generator() -> &'static FastIdGenerator {
    ID_GENERATOR.get().expect("ID generator not initialized.")
}

/// ID 信息结构体
#[derive(Debug, Clone)]
pub struct IdInfo {
    /// 时间戳（ms）
    pub timestamp: i64,
    /// 机器码
    pub worker_id: u16,
    /// 序列号
    pub sequence: u32,
    /// SystemTime 格式的时间
    pub system_time: SystemTime,
}

impl Default for IdInfo {
    fn default() -> Self {
        Self {
            timestamp: 0,
            worker_id: 0,
            sequence: 0,
            system_time: UNIX_EPOCH,
        }
    }
}

/// 快速初始化（使用默认位长配置）
///
/// # Examples
///
/// ```
/// use idgen_rs::snowflake_init;
/// 
/// snowflake_init(1); // worker_id = 1
/// ```
pub fn snowflake_init(worker_id: u16) {
    set_options(IGOptions::quick_init(worker_id));
}

/// 使用业务语义初始化（推荐方式）
///
/// 自动根据集群规模和目标 QPS 计算位长配置。
///
/// # Arguments
/// * `worker_id` - 当前节点 ID (0~65535)
/// * `max_nodes` - 集群最大节点数
/// * `max_qps`   - 目标最大 QPS
///
/// # Examples
///
/// ```
/// use idgen_rs::id_helper;
/// 
/// // 1024 节点集群，5万 QPS
/// id_helper::init_with_capacity(1, 1024, 50_000);
/// let id = id_helper::next_id();
/// ```
pub fn init_with_capacity(worker_id: u16, max_nodes: u32, max_qps: u32) {
    let options = IGOptions::with_capacity(worker_id, max_nodes, max_qps);
    set_options(options);
}

/// 设置选项
pub fn set_options(options: IGOptions) {
    // 幂等保护：已初始化则直接返回，避免 Criterion 多次触发 panic
    if ID_GENERATOR.get().is_some() {
        return;
    }
    ID_GENERATOR
        .set(FastIdGenerator::new(&options))
        .expect("ID generator has already been initialized with different options.");
}

/// 使用 Builder 模式初始化
///
/// # Examples
///
/// ```
/// use idgen_rs::{id_helper, options::IGOptions};
/// 
/// id_helper::set_options(
///     IGOptions::builder(1)
///         .worker_id_bit_length(10)
///         .seq_bit_length(10)
///         .top_over_cost_count(5000)
///         .build()
/// );
/// ```
pub fn init_with_builder(options: IGOptions) {
    set_options(options);
}

/// 检查是否已初始化
pub fn is_initialized() -> bool {
    ID_GENERATOR.get().is_some()
}

/// get the options of id_generator.
pub fn get_options() -> IGOptions {
    ID_GENERATOR.get()
        .map(|g| IGOptions::builder(g.worker_id()).build())
        .unwrap_or_else(|| IGOptions::quick_init(1))
}

/// generate next id.
pub fn next_id() -> u64 {
    get_generator().next_id()
}

/// 尝试生成 ID（可选 API，一般不推荐使用，返回 Result 版本，可用于需要错误处理的场景）
pub fn try_next_id() -> Result<u64, &'static str> {
    ID_GENERATOR
        .get()
        .map(|g| g.next_id())
        .ok_or("ID generator not initialized. Call snowflake_init() or set_options() first.")
}

/// 批量生成 ID
///
/// # Examples
///
/// ```
/// use idgen_rs::{snowflake_init, id_helper};
/// 
/// snowflake_init(1); // 必须先初始化
/// let ids = id_helper::next_ids(100);
/// assert_eq!(ids.len(), 100);
/// ```
pub fn next_ids(count: usize) -> Vec<u64> {
    let gen = get_generator();
    (0..count).map(|_| gen.next_id()).collect()
}

/// extract the timestamp from id.
pub fn extract_time(id: u64) -> SystemTime {
    UNIX_EPOCH + std::time::Duration::from_millis(get_generator().extract_timestamp(id) as u64)
}

/// 解析 ID 的完整信息
///
/// # Examples
///
/// ```
/// use idgen_rs::{snowflake_init, id_helper};
/// 
/// snowflake_init(1);
/// let id = id_helper::next_id();
/// let info = id_helper::extract_id_info(id);
/// println!("worker_id: {}, sequence: {}", info.worker_id, info.sequence);
/// ```
pub fn extract_id_info(id: u64) -> IdInfo {
    get_generator().extract_id_info(id)
}

/// 批量解析 ID 信息
pub fn extract_id_infos(ids: &[u64]) -> Vec<IdInfo> {
    let gen = get_generator();
    ids.iter().map(|&id| gen.extract_id_info(id)).collect()
}

/// 使用 chrono 解析 ID 时间（返回 Utc）
///
/// # Examples
///
/// ```
/// use idgen_rs::{snowflake_init, id_helper};
/// 
/// snowflake_init(1);
/// let id = id_helper::next_id();
/// let datetime = id_helper::extract_time_utc(id).unwrap();
/// println!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
/// ```
pub fn extract_time_utc(id: u64) -> Option<chrono::DateTime<chrono::Utc>> {
    let ms = get_generator().extract_timestamp(id);
    chrono::DateTime::from_timestamp_millis(ms)
}