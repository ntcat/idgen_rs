# 🦜 idgen_rs

## Introduction

**idgen_rs** is a pure Rust implementation of a high-performance **Snowflake ID generator**.

Snowflake is Twitter's open-source distributed ID generation algorithm. The core idea is to divide a 64-bit ID into multiple parts:
- **Sign bit** (1 bit): Always 0
- **Timestamp** (41 bits): Milliseconds relative to base time
- **Worker ID**: Identifies the generating node
- **Sequence**: Incrementing number within the same millisecond

**Key Features:**
1. **Lock-free design** - Uses atomic operations (CAS) instead of Mutex for extreme concurrency
2. **Globally unique** - Worker ID ensures distributed uniqueness
3. **Time-ordered** - Extract generation time from ID
4. **Monotonically increasing** - All digits, u64, database-friendly
5. **Business semantics** - Auto-calculate bit length based on cluster size
6. **Clock rollback tolerance** - Smart handling of NTP sync-induced clock drift

## Performance Comparison

| Implementation | Single Core QPS | 4-Thread QPS | Lock Type |
|----------------|-----------------|--------------|-----------|
| Original (Mutex) | ~0.5M | ~2M | Global mutex |
| **This version (Atomic)** | ~2M+ | ~8M+ | Lock-free CAS |

> Benchmark: `next_id` ~488ns per ID, throughput ~2.05M IDs/sec (single core)

## Installation

```toml
[dependencies]
idgen_rs = "0.2.0"
```

### Optional Features

```toml
# Performance metrics
idgen_rs = { version = "0.2.0", features = ["metrics"] }

# Encoding support (base62 + base64)
idgen_rs = { version = "0.2.0", features = ["encodings"] }

# All features
idgen_rs = { version = "0.2.0", features = ["metrics", "encodings"] }
```

## Quick Start

### Method 1: Quick Init (Recommended, Simplest)

```rust
use idgen_rs::snowflake_init;

fn main() {
    snowflake_init(1);
    let id = id_helper::next_id();
}
```

### Method 2: Business Semantics Init (Advanced)

```rust
use idgen_rs::id_helper;

fn main() {
    // Just specify cluster size and target QPS, bit length is auto-calculated
    id_helper::init_with_capacity(1, 1024, 50_000);
    
    let id = id_helper::next_id();
    println!("{}", id);
}
```

### Method 3: Builder Pattern

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

## Full Example

```rust
use idgen_rs::{init_with_capacity, id_helper};

fn main() {
    init_with_capacity(1, 1024, 50_000);

    // Single ID
    let id = id_helper::next_id();
    println!("ID: {}", id);

    // Batch generation
    let ids = id_helper::next_ids(5);
    println!("Batch: {:?}", ids);

    // Parse info
    let info = id_helper::extract_id_info(id);
    println!("worker_id={}, sequence={}", info.worker_id, info.sequence);
    
    // UTC time
    if let Some(utc) = id_helper::extract_time_utc(id) {
        println!("{}", utc.format("%Y-%m-%d %H:%M:%S UTC"));
    }
}
```

## API Reference

| Function | Description |
|----------|-------------|
| `init_with_capacity(worker_id, max_nodes, max_qps)` | Business semantics init (recommended) |
| `snowflake_init(worker_id)` | Quick init |
| `set_options(options)` | Set options |
| `get_options()` | Get config |
| `is_initialized()` | Check initialization status |
| `next_id()` | Generate single ID (lock-free) |
| `try_next_id()` | Generate ID, returns Result |
| `next_ids(count)` | Batch generate IDs |
| `extract_time(id)` | Extract SystemTime |
| `extract_time_utc(id)` | Extract DateTime<Utc> (returns Option) |
| `extract_id_info(id)` | Parse full ID info |
| `extract_id_infos(ids)` | Batch parse |

## Encoding Support (encodings feature)

```rust
use idgen_rs::{snowflake_init, IdEncoding, IdDecoding};

fn main() {
    snowflake_init(1);
    let id = id_helper::next_id();
    
    // Encode
    println!("base62: {}", id.to_base62());
    println!("base64: {}", id.to_base64());
    println!("url-safe: {}", id.to_base64_url());
    
    // Decode
    let decoded = u64::from_base62("1sMDF0VgTg").unwrap();
}
```

## Performance Metrics (metrics feature)

```rust
use idgen_rs::{FastIdGenerator, IGOptions, IdGeneratorMetrics};

fn main() {
    let options = IGOptions::quick_init(1);
    let gen = FastIdGenerator::new(&options);
    
    // Generate IDs...
    gen.next_id();
    
    // Get metrics
    let metrics: IdGeneratorMetrics = gen.metrics();
    println!("Total generated: {}", metrics.total_generated);
    println!("Clock rollback: {}", metrics.clock_backwards_count);
}
```

## Internal Implementation

Uses `AtomicU64` to pack `(timestamp << bits) | sequence`, with `compare_exchange_weak` for lock-free updates:

```rust
// Pseudocode
if now == last_timestamp {
    // Same millisecond: CAS increment sequence
    new_state = (now << seq_bits) | (old_seq + 1);
} else if now > last_timestamp {
    // New millisecond: CAS update timestamp, reset sequence
    new_state = now << seq_bits;
} else {
    // Clock rollback: spin for small drift, sleep for large drift
}
```

## Capacity Reference

| max_nodes | max_qps | worker_bits | seq_bits | Years Available |
|-----------|---------|-------------|----------|-----------------|
| 64 | 10,000 | 6 | 5 | ~69 years |
| 1024 | 50,000 | 10 | 6 | ~68 years |
| 4096 | 200,000 | 12 | 9 | ~67 years |
