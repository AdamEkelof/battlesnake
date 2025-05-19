use crate::{Board, /*Coord,*/ GameInfo};
use log::info;
use ordered_float::OrderedFloat;
use std::time::Instant;

// Define a tree node that can have many children
#[derive(Debug)]
struct TreeNode {
    value: f32,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(value: f32) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: TreeNode) {
        //info!("Adding child with value: {}", child.value);
        self.children.push(child);
    }

    fn print(&self, prefix: String, is_last: bool) {
        println!(
            "{}{}─ {}",
            prefix,
            if is_last { "└" } else { "├" },
            self.value
        );
        let new_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });

        let last_index = self.children.len().saturating_sub(1);
        for (i, child) in self.children.iter().enumerate() {
            child.print(new_prefix.clone(), i == last_index);
        }
    }
}

use crate::logic::simple::SimpleBoard;

use super::simple::SnakeMove;

pub fn search(board: &Board, game_info: &GameInfo) -> [SnakeMove; 2] {
    let start = Instant::now();
    let simple_board = SimpleBoard::from(board, game_info);
    let timeout: i32 = game_info.timeout as i32 * 1_000_000; // Convert milliseconds to nanoseconds
    let mut values = Vec::new();
    let mut moves = Vec::new();

    let mut best_value = f32::MIN;
    let simulations = simple_board.simulate_move(true);
    for (i, (move_pair, next_board)) in simulations.iter().enumerate() {
        let time: i32 = (timeout - start.elapsed().as_nanos() as i32) / (simulations.len() as i32 - i as i32);
        info!("Move {} time: {} (timeout: {} elapsed: {})", i, time, timeout, start.elapsed().as_nanos());

        let mut root = TreeNode::new(0.0);

        // minmax on enemies since this outer loop is on friendly
        let value = minmax_simple(
            &next_board,
            1,
            false,
            best_value,
            f32::MAX,
            1,
            2,
            time,
            &mut root,
        );
        //root.print(format!("{:?}:", move_pair), true);
        best_value = best_value.max(value);
        values.push(value);
        moves.push(move_pair);
    }
    let idx = values
        .iter()
        .enumerate()
        .max_by(|(_, v), (_, v2)| OrderedFloat(**v).cmp(&OrderedFloat(**v2)))
        .map(|(i, _)| i)
        .expect(&format!(
            "No best move found in values: {:?} for {} moves",
            values,
            simulations.len()
        ));
    *moves[idx]
}

fn minmax_simple(
    board: &SimpleBoard,
    depth: i32,
    our_team: bool,
    mut alpha: f32,
    mut beta: f32,
    heuristic_time: i32,
    return_time: i32,
    timeout: i32,
    parent: &mut TreeNode,
) -> f32 {
    let start = Instant::now();
    let mut node = TreeNode::new(0.0);
    if depth == 100 || heuristic_time + return_time >= timeout {
        //info!("Depth {} reached", depth);
        let h = board.heuristic();
        node.value = h;
        parent.add_child(node);
        return h;
    }

    let mut simulations = board.simulate_move(our_team);
    if our_team {
        simulations.sort_by_key(|s| OrderedFloat(-s.1.heuristic()));
    } else {
        simulations.sort_by_key(|s| OrderedFloat(s.1.heuristic()));
    }

    if let Some(sim) = simulations.first() {
        let h = sim.1.heuristic();
        if our_team && h == f32::MAX {
            //info!("Found max value at depth {}", depth);
            node.value = f32::MAX;
            parent.add_child(node);
            return f32::MAX;
        } else if !our_team && h == f32::MIN {
            //info!("Found min value at depth {}", depth);
            node.value = f32::MIN;
            parent.add_child(node);
            return f32::MIN;
        }
    }

    let mut best_value = if our_team { f32::MIN } else { f32::MAX };

    for (idx, (_, next_board)) in simulations.iter().enumerate() {
        let time_left = timeout - start.elapsed().as_nanos() as i32 - return_time;
        if time_left <= 0 {
            //info!("Ran out of time at depth {}", depth);
            let h = board.heuristic();
            // What is this ret for?
            let ret = if our_team {
                h.max(best_value)
            } else {
                h.min(best_value)
            };
            node.value = h;
            parent.add_child(node);
            return h;
        }

        let iterations_left = simulations.len() as i32 - idx as i32;
        let time_per_move = time_left / iterations_left;
        let value = minmax_simple(
            &next_board,
            depth + 1,
            !our_team,
            alpha,
            beta,
            heuristic_time,
            return_time,
            time_per_move,
            &mut node,
        );
        if our_team {
            best_value = best_value.max(value);
            alpha = alpha.max(best_value);
            if best_value >= beta {
                break;
            }
        } else {
            best_value = best_value.min(value);
            beta = beta.min(best_value);
            if best_value <= alpha {
                break;
            }
        }
    }

    //info!("Best value at depth {}: {}", depth, best_value);
    node.value = best_value;
    parent.add_child(node);
    best_value
}
