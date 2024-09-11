use idgen_rs::{id_generator_options::IdGeneratorOptions, yit_id_helper};
use chrono::{DateTime, Local, TimeZone, Utc}; 


fn main() {
    // 雪花算法创建 IdGeneratorOptions 对象，可在构造函数中输入 WorkerId：
    let mut options = IdGeneratorOptions::new(1); // 1 是 worker id
    options.worker_id_bit_length = 10; // 默认值6，限定 WorkerId 最大值为2^6-1，即默认最多支持64个节点。
    options.seq_bit_length = 6; // 默认值6，限制每毫秒生成的ID个数。若生成速度超过5万个/秒，建议加大 SeqBitLength 到 10。
    let base_time: DateTime<Utc> = Utc.with_ymd_and_hms(2023, 3, 13, 3, 3, 3)
        .single()
        .expect("Failed to create DateTime<Utc>");
    options.base_time = base_time.timestamp_millis(); 

    // 保存参数（务必调用，否则参数设置不生效）：
    yit_id_helper::set_id_generator(options);
    // 初始化后，在任何需要生成ID的地方，调用以下方法：
    let new_id = yit_id_helper::next_id();
    println!("new_id: {}", new_id);

    let time = yit_id_helper::extract_time(new_id);
    let datetime: DateTime<Local> = time.into(); 
    println!("time: {}", datetime.format("%Y-%m-%d %H:%M:%S"));
}