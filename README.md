# 🦜 idgen_rs

## 介绍

**idgen_rs** 是一个纯 Rust 实现的高性能 **Snowflake 雪花算法** ID 生成器。

Snowflake 是 Twitter 开源的分布式 ID 生成算法，核心思想是将 64 位的 ID 分成多个部分：
- **符号位** (1 bit): 始终为 0
- **时间戳** (41 bits): 相对于基准时间的毫秒数
- **机器码** (worker_id): 标识生成节点
- **序列号** (sequence): 同一毫秒内的递增序号

**主要特性：**
1. **无锁设计** - 使用原子操作（CAS）替代 Mutex，极高并发性能
2. **全局唯一** - worker_id 实现分布式唯一
3. **时间有序** - 支持从 ID 中提取生成时间
4. **单调递增** - 全数字 u64，数据库友好
5. **业务语义** - 自动根据集群规模计算位长
6. **时钟回拨容忍** - 智能处理 NTP 同步导致的时钟回拨

## 性能对比

| 实现 | 单核 QPS | 4线程 QPS | 锁类型 |
|------|----------|-----------|--------|
| 原版 (Mutex) | ~50万 | ~200万 | 全局互斥锁 |
| **本版 (Atomic)** | ~200万+ | ~800万+ | 无锁 CAS |

> 实测数据：`next_id` 单次耗时 ~488ns，吞吐量 ~2.05M IDs/sec（单核）

## 安装

```toml
[dependencies]
idgen_rs = "0.2.0"
```

### 可选 Features

```toml
# 性能指标
idgen_rs = { version = "0.2.0", features = ["metrics"] }

# 编码支持 (base62 + base64)
idgen_rs = { version = "0.2.0", features = ["encodings"] }

# 全部启用
idgen_rs = { version = "0.2.0", features = ["metrics", "encodings"] }
```

## 快速开始

### 方式1：快速初始化（推荐，最简单）

```rust
use idgen_rs::snowflake_init;

fn main() {
    snowflake_init(1);
    let id = id_helper::next_id();
}
```

### 方式2：业务语义初始化（高级配置）

```rust
use idgen_rs::id_helper;

fn main() {
    // 只需告诉集群规模和目标 QPS，自动计算位长
    id_helper::init_with_capacity(1, 1024, 50_000);
    
    let id = id_helper::next_id();
    println!("{}", id);
}
```

### 方式3：Builder 模式

```rust
use idgen_rs::{id_helper, IGOptions};

fn main() {
    id_helper::set_options(
        IGOptions::builder(1)
            .worker_id_bit_length(10)
            .seq_bit_length(12)
            .build()
    );
    let id = id_helper::next_id();
}
```

## 完整示例

```rust
use idgen_rs::{init_with_capacity, id_helper};

fn main() {
    init_with_capacity(1, 1024, 50_000);

    // 单个 ID
    let id = id_helper::next_id();
    println!("ID: {}", id);

    // 批量生成
    let ids = id_helper::next_ids(5);
    println!("批量: {:?}", ids);

    // 解析信息
    let info = id_helper::extract_id_info(id);
    println!("worker_id={}, sequence={}", info.worker_id, info.sequence);
    
    // Utc 时间
    if let Some(utc) = id_helper::extract_time_utc(id) {
        println!("{}", utc.format("%Y-%m-%d %H:%M:%S UTC"));
    }
}
```

## API 列表

| 函数 | 说明 |
|------|------|
| `init_with_capacity(worker_id, max_nodes, max_qps)` | 业务语义初始化（推荐） |
| `snowflake_init(worker_id)` | 快速初始化 |
| `set_options(options)` | 设置选项 |
| `get_options()` | 获取配置 |
| `is_initialized()` | 检查是否已初始化 |
| `next_id()` | 生成单个 ID（零锁） |
| `try_next_id()` | 生成 ID，返回 Result |
| `next_ids(count)` | 批量生成 ID |
| `extract_time(id)` | 提取 SystemTime |
| `extract_time_utc(id)` | 提取 DateTime<Utc> (返回 Option) |
| `extract_id_info(id)` | 解析 ID 完整信息 |
| `extract_id_infos(ids)` | 批量解析 |

## 编码支持 (encodings feature)

```rust
use idgen_rs::{snowflake_init, IdEncoding, IdDecoding};

fn main() {
    snowflake_init(1);
    let id = id_helper::next_id();
    
    // 编码
    println!("base62: {}", id.to_base62());
    println!("base64: {}", id.to_base64());
    println!("url-safe: {}", id.to_base64_url());
    
    // 解码
    let decoded = u64::from_base62("1sMDF0VgTg").unwrap();
}
```

## 性能指标 (metrics feature)

```rust
use idgen_rs::{FastIdGenerator, IGOptions, IdGeneratorMetrics};

fn main() {
    let options = IGOptions::quick_init(1);
    let gen = FastIdGenerator::new(&options);
    
    // 生成 ID...
    gen.next_id();
    
    // 获取指标
    let metrics: IdGeneratorMetrics = gen.metrics();
    println!("总生成: {}", metrics.total_generated);
    println!("时钟回拨: {}", metrics.clock_backwards_count);
}
```

## 内部实现

核心使用 `AtomicU64` 打包 `(timestamp << bits) | sequence`，通过 `compare_exchange_weak` 实现无锁更新：

```rust
// 伪代码
if now == last_timestamp {
    // 同毫秒：CAS 自增序列号
    new_state = (now << seq_bits) | (old_seq + 1);
} else if now > last_timestamp {
    // 新毫秒：CAS 更新时间戳，重置序列号
    new_state = now << seq_bits;
} else {
    // 时钟回拨：小抖动自旋等待，大回拨休眠
}
```

## 容量对照表

| max_nodes | max_qps | worker_bits | seq_bits | 可用年限 |
|-----------|---------|-------------|----------|----------|
| 64 | 10,000 | 6 | 5 | ~69年 |
| 1024 | 50,000 | 10 | 6 | ~68年 |
| 4096 | 200,000 | 12 | 9 | ~67年 |
