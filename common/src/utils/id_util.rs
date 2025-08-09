use crate::lex::StateID;
use crate::utils::unique_id_factory::UniqueIDFactory;

/// 递增
pub struct IncIDFactory {
    counter: usize,
}

impl IncIDFactory {
    pub fn new(t: StateID) -> Self {
        Self { counter: t }
    }
}

impl Iterator for IncIDFactory {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_id())
    }
}

impl UniqueIDFactory<usize> for IncIDFactory {
    fn peek(&self) -> usize {
        self.counter
    }

    fn next_id(&mut self) -> usize {
        let cp = self.counter;
        self.counter += 1;
        cp
    }

    fn skip_id(&mut self, cnt: usize) {
        self.counter += cnt;
    }
}
