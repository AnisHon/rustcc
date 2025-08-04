pub trait UniqueIDFactory<T>: Iterator {
    fn peek(&self) -> T;

    fn next_id(&mut self) -> T;

    fn skip_id(&mut self, cnt: usize);
}
