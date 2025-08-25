

#[derive(Debug, Clone)]
pub enum LRAction {
    Reduce(usize), // 规约 推导式ID
    Shift(usize),  // 移入 状态ID
    End(usize),    // 结束规约 推导式ID
    Error          // 出错Error
}

// action matrix -> base next check
const ACTION_BASE: [Option<usize>; 8] = [Some(0),Some(8),Some(10),Some(6),Some(2),Some(1),Some(4),Some(2),];

const ACTION_NEXT: [Option<usize>; 89] = [LRAction::Error,LRAction::End(0),LRAction::Error,LRAction::End(3),LRAction::Error,LRAction::End(3),LRAction::Error,LRAction::End(1),LRAction::Error,LRAction::End(1),LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Reduce(0),LRAction::Reduce(0),LRAction::Reduce(3),LRAction::Reduce(3),LRAction::Reduce(3),LRAction::Reduce(3),LRAction::Reduce(1),LRAction::Reduce(1),LRAction::Reduce(1),LRAction::Reduce(1),LRAction::Shift(4),LRAction::Shift(5),LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Shift(1),LRAction::Shift(1),LRAction::Shift(1),LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,LRAction::Error,];

const ACTION_CHECK: [Option<usize>; 89] = [None,Some(0),None,Some(7),None,Some(6),None,Some(3),None,Some(1),None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,Some(0),Some(0),Some(7),Some(7),Some(6),Some(6),Some(3),Some(3),Some(1),Some(1),Some(2),Some(2),None,None,None,None,None,None,None,None,None,None,None,None,None,None,None,Some(0),Some(5),Some(4),None,None,None,None,None,];

// goto matrix -> base next check
const GOTO_BASE: [Option<usize>; 8] = [Some(0),None,None,None,Some(3),Some(2),None,None,];

const GOTO_NEXT: [Option<usize>; 5] = [Some(2),Some(3),None,Some(7),Some(6),];

const GOTO_CHECK: [Option<usize>; 5] = [Some(0),Some(0),None,Some(5),Some(4),];


// rule_id -> length for reduce
pub const RULE_LENS: [usize; 5] = [3,3,1,0,1,];


/// action_table[state][token]
pub fn get_action(state: usize, token: usize) -> LRAction {
    let base = ACTION_BASE[state_id];
    if base.is_none() {
        return LRAction::Error
    }

    let idx = base.unwrap() + class_id;

    let check = ACTION_CHECK[idx];

    if check.is_none() {
        return None
    }

    if check.unwrap() == state_id {
        ACTION_NEXT[idx]
    } else {
        None
    }
}

/// action_table[state][rule]
pub fn get_goto(state: usize, rule: usize) -> LRAction {
    let base = GOTO_BASE[state_id];
    if base.is_none() {
        return LRAction::Error
    }

    let idx = base.unwrap() + class_id;

    let check = GOTO_CHECK[idx];

    if check.is_none() {
        return None
    }

    if check.unwrap() == state_id {
        GOTO_NEXT[idx]
    } else {
        None
    }
}

/// action_code[state](params)
pub fn exec_action<T>(rule: usize, mut value: T, value_stack: Vec<T>) {
    match rule {
                0 => { value = value_stack[0] + value_stack[2]; },
            
                1 => { value = value_stack[0] - value_stack[2]; },
            _ => unreachable!()
    };
    value
}

