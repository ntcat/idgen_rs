use idgen_rs::{snowflake_init, id_helper};
#[cfg(feature = "encodings")]
use idgen_rs::{IdEncoding, IdDecoding};
use chrono::{DateTime, Local};

const WORKER_ID: u16 = 1;

fn main() {
    // 方式1: 快速初始化（推荐，最简单）
    snowflake_init(WORKER_ID);

    // 方式2: 业务语义初始化（高级配置）
    // 自动计算位长：1024 节点集群，5万 QPS
    // init_with_capacity(WORKER_ID, 1024, 50_000);

    // 方式3: 使用 Builder 模式（完全自定义）
    // set_options(
    //     IGOptions::builder(WORKER_ID)
    //         .worker_id_bit_length(10)
    //         .seq_bit_length(10)
    //         .build()
    // );

    println!("已初始化: {}", id_helper::is_initialized());

    // 单个 ID 生成
    let new_id = id_helper::next_id();
    println!("new_id: {}", new_id);

    // 编码解码（需启用 encodings feature）
    #[cfg(feature = "encodings")]
    {
        // 编码
        let encoded = new_id.to_base62();
        println!("base62: {}", encoded);
        let encoded = new_id.to_base64_url();
        println!("base64 url: {}", encoded);

        // 解码
        if let Ok(decoded) = u64::from_base62(&new_id.to_base62()) {
            println!("base62 解码: {}", decoded);
        }
        if let Ok(decoded) = u64::from_base64_url(&new_id.to_base64_url()) {
            println!("base64 url 解码: {}", decoded);
        }
    }

    // 批量生成
    let ids = id_helper::next_ids(5);
    println!("批量生成: {:?}", ids);

    // 解析 ID 信息
    let info = id_helper::extract_id_info(new_id);
    println!("ID 信息: worker_id={}, sequence={}", info.worker_id, info.sequence);

    let datetime: DateTime<Local> = info.system_time.into();
    println!("time: {}", datetime.format("%Y-%m-%d %H:%M:%S"));

    // 使用 Utc 时区
    if let Some(utc_time) = id_helper::extract_time_utc(new_id) {
        println!("UTC time: {}", utc_time.format("%Y-%m-%d %H:%M:%S UTC"));
    }
}
