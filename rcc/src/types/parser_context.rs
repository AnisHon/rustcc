use std::sync::mpsc;
use crate::err::global_err::GlobalError;

pub struct ParserContext {
    pub sync_token: bool,
    pub error_tx: mpsc::Sender<GlobalError>,
}

impl ParserContext {
    pub fn new(error_tx: mpsc::Sender<GlobalError>) -> Self {
        Self {
            sync_token: false,
            error_tx,
        }
    }
}