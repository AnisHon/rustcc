use crate::lex::types::token_kind::TokenKind::*;
use crate::lex::types::token_kind::TokenKind;
pub const INIT_STATE: usize = 47;

static BASE: [Option<usize>; 48] = [
    None, Some(19), Some(18), Some(11), Some(17), Some(10), Some(3), Some(5), Some(16), Some(9), Some(15), 
    Some(6), Some(14), Some(5), Some(13), Some(12), Some(4), Some(0), None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, 
];

static NEXT: [usize; 130] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    1, 0, 0, 0, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11, 37, 43, 36, 46, 0, 40, 0, 0, 0, 0, 12, 13, 14, 15, 
    16, 17, 38, 39, 25, 28, 29, 32, 33, 41, 44, 34, 30, 27, 31, 35, 42, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 18, 0, 19, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    0, 0, 21, 22, 23, 24, 0, 0, 26, 
];

static CHECK: [Option<usize>; 130] = [
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, Some(17), 
    None, None, None, Some(17), Some(17), None, Some(17), Some(17), Some(17), Some(17), Some(17), Some(17), 
    Some(17), Some(17), Some(6), Some(3), Some(16), Some(7), None, Some(5), None, None, None, None, Some(17), 
    Some(17), Some(17), Some(17), Some(17), Some(17), Some(6), Some(6), Some(13), Some(11), Some(11), 
    Some(9), Some(9), Some(5), Some(3), Some(15), Some(14), Some(12), Some(10), Some(8), Some(4), Some(2), 
    Some(1), None, None, None, None, None, None, None, None, None, None, Some(17), None, Some(17), Some(17), 
    None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, 
    None, None, None, None, None, None, None, None, None, None, None, Some(17), Some(17), Some(17), Some(17), 
    None, None, Some(13), 
];

static ROW_ID: [usize; 48] = [
    0, 1, 2, 3, 0, 0, 4, 5, 0, 6, 7, 8, 0, 0, 9, 10, 11, 0, 0, 0, 12, 0, 13, 0, 0, 0, 0, 0, 0, 14, 0, 
    0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 17, 
];

pub static STATES: [Option<TokenKind>; 48] = [
    Some(Ne),
    Some(Amp),
    Some(Percent),
    Some(Pipe),
    Some(LParen),
    Some(RParen),
    Some(Star),
    Some(Plus),
    Some(Comma),
    Some(Minus),
    Some(Dot),
    Some(Slash),
    Some(Colon),
    Some(Semi),
    Some(Lt),
    Some(Assign),
    Some(Gt),
    Some(Question),
    Some(LBracket),
    Some(RBracket),
    Some(Caret),
    Some(LBrace),
    Some(Caret),
    Some(RBrace),
    Some(Tilde),
    Some(PipeEq),
    Some(Or),
    Some(CaretEq),
    Some(Ge),
    Some(Shr),
    Some(ShrEq),
    Some(Eq),
    Some(Shl),
    Some(Le),
    Some(ShlEq),
    Some(SlashEq),
    Some(Ellipsis),
    Some(Dec),
    Some(MinusEq),
    Some(Arrow),
    Some(Inc),
    Some(PlusEq),
    Some(StarEq),
    Some(And),
    Some(AmpEq),
    Some(PercentEq),
    None,
    None,
];


pub fn find_next(state_id: usize, chr: char) -> Option<usize> {
    let row_id = ROW_ID[state_id];
    let class_id = chr as usize;
    let base = BASE[row_id]?;

    let idx = base + class_id;
    let check = CHECK[idx]?;

    if check == row_id {
        Some(NEXT[idx])
    } else {
        None
    }
}
