use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::id_helper::IdInfo;
use crate::options::IGOptions;

/// ID 生成器指标
#[cfg(feature = "metrics")]
#[derive(Debug, Clone)]
pub struct IdGeneratorMetrics {
    /// 总生成 ID 数量
    pub total_generated: u64,
    /// 时钟回拨次数
    pub clock_backwards_count: u64,
    /// CAS 冲突次数
    pub cas_conflict_count: u64,
}

/// 高性能雪花 ID 生成器
/// 使用原子操作替代 Mutex，保留完整的 ID 可解析性
#[derive(Debug)]
pub struct FastIdGenerator {
    base_time: i64,
    worker_id: u64,
    timestamp_shift: u8,
    seq_bits: u8,
    seq_mask: u64,
    max_seq: u64,

    // 原子状态：存储 (last_timestamp << seq_bits) | sequence
    state: AtomicU64,

    // 指标统计（可选）
    #[cfg(feature = "metrics")]
    total_generated: AtomicU64,
    #[cfg(feature = "metrics")]
    clock_backwards_count: AtomicU64,
    #[cfg(feature = "metrics")]
    cas_conflict_count: AtomicU64,
}

impl FastIdGenerator {
    pub fn new(options: &IGOptions) -> Self {
        let seq_bits = options.seq_bit_length;
        let wid_bits = options.worker_id_bit_length;

        Self {
            base_time: options.base_time,
            worker_id: options.worker_id as u64,
            timestamp_shift: wid_bits + seq_bits,
            seq_bits,
            seq_mask: (1u64 << seq_bits) - 1,
            max_seq: (1u64 << seq_bits) - 1,
            state: AtomicU64::new(0),
            #[cfg(feature = "metrics")]
            total_generated: AtomicU64::new(0),
            #[cfg(feature = "metrics")]
            clock_backwards_count: AtomicU64::new(0),
            #[cfg(feature = "metrics")]
            cas_conflict_count: AtomicU64::new(0),
        }
    }

    /// 无锁生成下一个 ID
    ///
    /// 使用原子 CAS 操作实现无锁并发：
    /// 1. 读取当前状态 (last_timestamp, sequence)
    /// 2. 根据时间戳决定行为：
    ///    - 同一毫秒：尝试递增序列号
    ///    - 新毫秒：重置序列号为 0
    ///    - 时钟回拨：小抖动自旋，大回拨休眠
    /// 3. CAS 更新状态，失败则重试
    #[inline]
    pub fn next_id(&self) -> u64 {
        // 初始时间戳（相对 base_time 的毫秒差）
        let mut now = (current_time_millis() - self.base_time) as u64;

        loop {
            let state = self.state.load(Ordering::Acquire);
            let last_timestamp = state >> self.seq_bits;
            let last_seq = state & self.seq_mask;

            if now == last_timestamp {
                let new_seq = last_seq + 1;
                if new_seq > self.max_seq {
                    // 序列号耗尽：让出 CPU 核心管线，等待下一毫秒
                    std::hint::spin_loop();
                    // ✅ 必须刷新时间，否则会用过期时间无限循环
                    now = (current_time_millis() - self.base_time) as u64;
                    continue;
                }

                let new_state = (now << self.seq_bits) | new_seq;
                if self
                    .state
                    .compare_exchange_weak(state, new_state, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    return (now << self.timestamp_shift)
                        | (self.worker_id << self.seq_bits)
                        | new_seq;
                }
            } else if now > last_timestamp {
                let new_state = now << self.seq_bits; // 序列号重置为 0
                if self
                    .state
                    .compare_exchange_weak(state, new_state, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    return (now << self.timestamp_shift) | (self.worker_id << self.seq_bits);
                }
            } else {
                // 时钟回拨保护
                let drift = last_timestamp - now;
                if drift > 10 {
                    // 超过 10ms 回拨直接 panic，避免 NTP 校准导致 ID 重复或业务阻塞
                    panic!(
                        "⏰ Clock rollback detected ({} ms). ID generation halted.",
                        drift
                    );
                }
                std::hint::spin_loop();
                now = (current_time_millis() - self.base_time) as u64;
            }
        }
    }

    #[inline]
    pub fn next_ids_batch(&self, count: usize) -> Vec<u64> {
        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            ids.push(self.next_id());
        }
        ids
    }

    /// 获取指标（需要启用 metrics feature）
    #[cfg(feature = "metrics")]
    pub fn metrics(&self) -> IdGeneratorMetrics {
        IdGeneratorMetrics {
            total_generated: self.total_generated.load(Ordering::Relaxed),
            clock_backwards_count: self.clock_backwards_count.load(Ordering::Relaxed),
            cas_conflict_count: self.cas_conflict_count.load(Ordering::Relaxed),
        }
    }

    /// 从 ID 提取时间戳（毫秒）
    #[inline]
    pub fn extract_timestamp(&self, id: u64) -> i64 {
        let ts = id >> self.timestamp_shift;
        (ts as i64) + self.base_time
    }

    /// 从 ID 提取 worker_id
    #[inline]
    pub fn extract_worker_id(&self, id: u64) -> u16 {
        let wid_bits = self.timestamp_shift - self.seq_bits;
        ((id >> self.seq_bits) & ((1u64 << wid_bits) - 1)) as u16
    }

    /// 获取当前生成器的 worker_id
    #[inline]
    pub fn worker_id(&self) -> u16 {
        self.worker_id as u16
    }

    /// 从 ID 提取序列号
    #[inline]
    pub fn extract_sequence(&self, id: u64) -> u32 {
        (id & self.seq_mask) as u32
    }

    /// 解析 ID 完整信息
    #[inline]
    pub fn extract_id_info(&self, id: u64) -> IdInfo {
        let timestamp = self.extract_timestamp(id);
        IdInfo {
            timestamp,
            worker_id: self.extract_worker_id(id),
            sequence: self.extract_sequence(id),
            system_time: UNIX_EPOCH + Duration::from_millis(timestamp as u64),
        }
    }
}

/// 获取当前时间戳（毫秒）
#[inline]
fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::DEFAULT_BASE_TIME;
    use std::sync::Arc;
    use std::thread;

    fn create_test_options() -> IGOptions {
        IGOptions {
            method: 1,
            base_time: DEFAULT_BASE_TIME,
            worker_id: 1,
            worker_id_bit_length: 10,
            seq_bit_length: 12,
            max_seq_number: 0,
            min_seq_number: 5,
            top_over_cost_count: 2000,
        }
    }

    #[test]
    fn test_basic_generation() {
        let gen = FastIdGenerator::new(&create_test_options());
        let id = gen.next_id();

        assert!(id > 0);
        assert_eq!(gen.extract_worker_id(id), 1);
    }

    #[test]
    fn test_id_info_extraction() {
        let gen = FastIdGenerator::new(&create_test_options());
        let id = gen.next_id();

        let info = gen.extract_id_info(id);
        assert_eq!(info.worker_id, 1);
        assert!(info.timestamp > DEFAULT_BASE_TIME);
    }

    #[test]
    fn test_sequence_increments() {
        let gen = FastIdGenerator::new(&create_test_options());

        // 同一毫秒内，序列号应该递增
        let id1 = gen.next_id();
        let id2 = gen.next_id();

        let seq1 = gen.extract_sequence(id1);
        let seq2 = gen.extract_sequence(id2);

        assert_eq!(seq2, seq1 + 1);
    }

    #[test]
    fn test_concurrent_generation() {
        let gen = Arc::new(FastIdGenerator::new(&create_test_options()));
        let mut handles = vec![];

        for _ in 0..4 {
            let gen = Arc::clone(&gen);
            handles.push(thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..1000 {
                    ids.push(gen.next_id());
                }
                ids
            }));
        }

        let mut all_ids = vec![];
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // 检查是否有重复
        let mut sorted_ids = all_ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();

        assert_eq!(all_ids.len(), sorted_ids.len(), "Found duplicate IDs");
    }

    #[test]
    fn test_id_uniqueness() {
        let gen = FastIdGenerator::new(&create_test_options());
        let mut ids = std::collections::HashSet::new();

        for _ in 0..10000 {
            let id = gen.next_id();
            assert!(ids.insert(id), "Duplicate ID found: {}", id);
        }
    }
}
