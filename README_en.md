# 🦜 idgen_rs
## Introduction
During the time I was writing GoLang, I used [idgen](https://github.com/yitter/idgenerator) , which satisfied my requirements for the Snowflake algorithm ID:

1. Globally unique, using worker_id to achieve distributed uniqueness.
2. Time-ordered, allowing me to extract the time when the ID was generated, which is very important to me.
3. Monotonically increasing, all digits, int64, without sacrificing performance due to string storage.

When I shifted my programming focus entirely to Rust, I naturally wanted to continue using idgen. However, I found that the author was somewhat lazy and used Rust to call a C library, rather than a pure Rust version. I managed to compile it on Linux, though it was troublesome. However, compilation failed on macOS, and I lost patience with it. Therefore, I decided to write a purely Rust version. Of course, it is based on idgen_go to ensure compatibility with this ID generation method.

idgen_rs is implemented based on the Snowflake algorithm and adds some features:

1. Supports multiple calculation methods, including traditional and drift algorithms.
2. Supports custom machine code bit length and sequence number bit length.
3. Supports custom maximum and minimum sequence numbers.
4. Supports custom maximum drift count.
5. Supports custom base time.


git clone https://github.com/ntcat/idgen_rs

```
fn main() {
    let mut options = IdGeneratorOptions::new(1); // 1 is the worker id
    options.worker_id_bit_length = 10; // Default value is 6, limiting the maximum value of `WorkerId` to 2^6 - 1, meaning up to 64 nodes by default.
    options.seq_bit_length = 6; // Default value is 6, limiting the number of IDs generated per millisecond. If the generation speed exceeds 50,000 IDs per second, consider increasing `SeqBitLength` to 10.
    let base_time: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 3, 13, 3, 3, 3)
        .single()
        .expect("Failed to create DateTime<Utc>");
    options.base_time = base_time.timestamp_millis();

    // Save parameters (this must be called, otherwise the settings will not take effect):
    yit_id_helper::set_id_generator(options);
    // After initialization, wherever an ID is needed, call the following method:
    let new_id = yit_id_helper::next_id();
    println!("new_id: {}", new_id);

    let time = yit_id_helper::extract_time(new_id);
    let datetime: DateTime<Local> = time.into();
    println!("time: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
}



pub struct IdGeneratorOptions {
    pub method: u16,                // Snowflake calculation method (1-drift algorithm | 2-traditional algorithm), default is 1
    pub base_time: i64,             // Base time (in milliseconds), cannot exceed the current system time
    pub worker_id: u16,             // Machine code, must be set externally, maximum value is 2^WorkerIdBitLength - 1
    pub worker_id_bit_length: u8,   // Machine code bit length, default value is 6, range is [1, 15] (requirement: SequenceBitLength + WorkerIdBitLength should not exceed 22)
    pub seq_bit_length: u8,         // Sequence number bit length, default value is 6, range is [3, 21] (requirement: SequenceBitLength + WorkerIdBitLength should not exceed 22)
    pub max_seq_number: u32,        // Maximum sequence number (inclusive), range is [MinSeqNumber, 2^SeqBitLength - 1], default value is 0, indicating the maximum sequence number is the maximum value (2^SeqBitLength - 1]
    pub min_seq_number: u32,        // Minimum sequence number (inclusive), default value is 5, range is [5, MaxSeqNumber], the first five sequence numbers (0-4) of each millisecond are reserved, where 1-4 are reserved for clock skew adjustments, and 0 is reserved for manual resets
    pub top_over_cost_count: u32,   // Maximum drift count (inclusive), default is 2000, recommended range is 500-10000 (dependent on computing capabilities)
}

```