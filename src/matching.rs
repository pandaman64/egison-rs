use std::collections::VecDeque;

use super::*;

fn match_all<A, M, B>(target: A, matcher: M, clauses: VecDeque<Box<dyn MatchClause<A, M, B>>>) -> VecDeque<B> {
    let mut ret = VecDeque::new();
    for clause in clauses.into_iter() {
    }
    ret
}

fn process_mstate_all_dfs<Vs>(mut mstates: VecDeque<Box<dyn MState<Vs>>>) -> VecDeque<Vs> {
    match mstates.pop_front() {
        Some(ms) => {
            match ms.decompose_if_nil() {
                Some(head) => {
                    let tail = process_mstate_all_dfs(mstates);
                    tail.push_front(head);
                    tail
                },
                None => {
                    let mut head = ms.process();
                    head.extend(mstates);
                    process_mstate_all_dfs(head)
                }
            }
        }
        None => VecDeque::new(),
    }
}
