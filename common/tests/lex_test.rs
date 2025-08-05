use common::lex::DFA;

#[test]
fn test_dfa() {
    // 'a(ab|cd)ff*'

    let mut dfa = DFA::new(0, 6, 128); // 你懂得ASCII 0-127
    dfa.add_transition((0, 'a' as usize, 5))
        .add_transition((5, 'a' as usize, 3))
        .add_transition((5, 'c' as usize, 4))
        .add_transition((3, 'b' as usize, 2))
        .add_transition((4, 'd' as usize, 2))
        .add_transition((2, 'f' as usize, 1))
        .add_transition((1, 'f' as usize, 1));

    dfa.get_meta_mut(1).terminate = true;

    let mut state = dfa.get_init_state();
    let match_str = "asabffffff";
    for c in match_str.chars() {
        let c = c as usize;

        state = match dfa.find_next(state, c) {
            Some(s) => s,
            None => {
                println!("Nope");
                break;
            }
        }
    }

    println!("{:?}", dfa.get_meta(state));
}

#[test]
fn test_nfa() {
    // let mut nfa = NFA::new(0);
    // nfa.add_state(0, StateMeta { terminate: false })
    //     .add_state(1, StateMeta { terminate: false })
    //     .add_state(2, StateMeta { terminate: false })
    //     .add_state(3, StateMeta { terminate: false })
    //     .add_state(4, StateMeta { terminate: false })
    //     .add_state(5, StateMeta { terminate: false })
    //     .add_state(6, StateMeta { terminate: false });
    //
    // nfa.add_edge(0, ClassID(1), 0)
    //     .add_edge(0, ClassID(1), 1)
    //     .add_edge(0, ClassID(2), 2)
    //     .add_edge(0, ClassID(2), 3)
    //     .add_edge(0, ClassID(3), 4)
    //     .add_edge(0, ClassID(3), 5);
    // println!("{:?}", nfa.get_symbols(0));
    // println!("{:?}", nfa.find_next(0, ClassID(1)));
}
