use crate::lex::StateID;
use crate::utils::unique_id_factory::UniqueIDFactory;

/// 递增
pub struct IncrementalStateIDFactory {
    counter: StateID,
}

impl IncrementalStateIDFactory {
    pub fn new(t: StateID) -> Self {
        Self { counter: t }
    }
}

impl Iterator for IncrementalStateIDFactory {
    type Item = StateID;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_id())
    }
}

impl UniqueIDFactory<StateID> for IncrementalStateIDFactory {
    fn peek(&self) -> StateID {
        self.counter
    }

    fn next_id(&mut self) -> StateID {
        let cp = self.counter;
        self.counter += 1;
        cp
    }

    fn skip_id(&mut self, cnt: usize) {
        self.counter += cnt;
    }
}
