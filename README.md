# 🦜 idgen_rs

## 介绍
写 golang 的那段时间一直用 [idgen](https://github.com/yitter/idgenerator) ,它很好的满足了，我对 snowfalke 算法 id的要求：
1. 全局唯一，使用 worker_id 可以实现分布式唯一。
2. 时间有序，甚至我可以从 id中抽取时生成 id 的时间，这个对我很重要。
3. 单调递增，全数字，int64,不需要为存储字符串损失性能。 

当我将编程方向全面转向 rust 时，自然我还是用 idgen.但是我发现作者比较懒，用的是 rust调用C 库。不是纯粹的 rust版本。
我尝试在 linux 环境下编译，虽然麻烦，但是还是成功了。但是在 mac 环境下编译失败，我失去使用它的耐心。决定自己写一个纯纯的 rust 版本。
当然它是基于 idgen_go,以保证这个 id生成方式的兼容性。

idgen_rs，基于 snowflake 算法实现的，并且增加了一些功能：
1. 支持多种计算方法，包括传统算法和漂移算法
2. 支持自定义机器码位长和序列数位长
3. 支持自定义最大序列数和最小序列数
4. 支持自定义最大漂移次数
5. 支持自定义基础时间

git clone https://github.com/ntcat/idgen_rs

```
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

//参考资料：

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

```