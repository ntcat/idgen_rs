use idgen_rs::{options::IGOptions, id_helper};
use chrono::{DateTime, Local, TimeZone, Utc}; 

const WORKER_ID: u16 = 1;

fn main() {
    snow_flake_init(WORKER_ID); 

    // 初始化后，在任何需要生成ID的地方，调用以下方法：
    let new_id = id_helper::next_id();
    println!("new_id: {}", new_id);

    let time = id_helper::extract_time(new_id);
    let datetime: DateTime<Local> = time.into(); 
    println!("time: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
}

// 雪花算法初始化
pub fn snow_flake_init(worker_id: u16) {
    let mut options = IGOptions::new(worker_id); 
    options.worker_id_bit_length = 10; // 默认值6，限定 worker_id 最大值为2^6-1，即默认最多支持64个节点。
    options.seq_bit_length = 6; // 默认值6，限制每毫秒生成的ID个数。若生成速度超过5万个/秒，建议加大 seq_bit_length 到 10。
    let base_time = Utc
        .with_ymd_and_hms(2023, 3, 13, 3, 3, 3)
        .single()
        .expect("Failed to create DateTime<Utc>");
    options.base_time = base_time.timestamp_millis() ; // meilisearch 使用这个时间

    // 保存参数（务必调用，否则参数设置不生效）：
    id_helper::set_options(options);
    // 初始化后，在任何需要生成ID的地方，调用以下方法：
    // let new_id = id_helper::next_id();
    //  println!("new_id: {}", new_id);

}
