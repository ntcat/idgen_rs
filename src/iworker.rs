pub trait IWorker {
    fn next_id(&mut self) -> i64;
}