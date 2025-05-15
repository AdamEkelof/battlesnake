use rocket::futures::future::ok;

use crate::logic::{collision_with_body, collision_with_snakes, get_safe_moves, out_of_bounds};
use crate::{Battlesnake, Board, Coord, GameInfo};
use std::collections::{HashMap, VecDeque};
use std::fmt::Display;

#[derive(Copy, Clone)]
pub enum Movement {
    Up,
    Down,
    Left,
    Right,
    None,
}
impl Movement {
    pub fn all() -> Vec<Movement> {
        vec![Self::Up, Self::Down, Self::Left, Self::Right]
    }
}
impl Display for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Movement::Up => String::from("up"),
            Movement::Down => String::from("down"),
            Movement::Left => String::from("left"),
            Movement::Right => String::from("right"),
            Movement::None => String::from("no movement made somehow"),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub struct SimpleBoard {
    pub food: Vec<Coord>,
    pub snakes: Vec<Option<SimpleSnake>>,
    team: [usize; 2],
    opps: [usize; 2],
}
impl SimpleBoard {
    pub fn from(board: &Board, game_info: &GameInfo) -> Self {
        let mut simple_board = SimpleBoard {
            food: board.food.clone(),
            snakes: Vec::new(),
            team: [0; 2],
            opps: [0; 2],
        };
        let mut friendly_count = 0;
        let mut enemy_count = 0;
        for (idx, snake) in board.snakes.iter().enumerate() {
            simple_board.snakes.push(Some(SimpleSnake::from(snake)));
            if game_info.agent_ids.contains(&snake.id) {
                simple_board.team[friendly_count] = idx;
                friendly_count += 1;
            } else {
                simple_board.opps[enemy_count] = idx;
                enemy_count += 1;
            }
        }
        simple_board
    }

    // fn evaluate_team(&self, our_team: bool) -> usize {
    //     let mut v = 0;
    //     for snake in self.snakes.iter() {
    //         if snake.our_team == our_team {
    //             v += snake.evaluate_value();
    //         }
    //     }
    //     v
    // }

    pub fn heuristic(&self) -> i32 {
        if self.snakes.len() == 0 {
            return 0;
        }
        let mut v = 0;
        let mut dead_snake_count = 0;
        // lägg in så man är 1 längre än motståndare
        for f_idx in self.team {
            match &self.snakes[f_idx] {
                Some(snake) => {
                    v += snake.body.len() as i32;
                    if snake.health < 50 {
                        v -= 1;
                    }
                }
                None => {
                    dead_snake_count += 1;
                    v -= 10;
                }
            }
        }
        if dead_snake_count == 2 {
            return i32::MIN;
        }
        dead_snake_count = 0;
        for e_idx in self.opps {
            match &self.snakes[e_idx] {
                Some(snake) => {
                    v -= snake.body.len() as i32;
                    if snake.health < 50 {
                        v += 1;
                    }
                }
                None => {
                    dead_snake_count += 1;
                    v += 10;
                }
            }
        }
        if dead_snake_count == 2 {
            return i32::MAX;
        }
        v
    }

    // fn flood_fill(&self) -> HashMap<&SimpleSnake, i32> {
    //     let mut v = HashMap::new();
    //     let mut queue: Vec<(&SimpleSnake, Coord)> = self
    //         .snakes
    //         .iter()
    //         .map(|x| (x, x.body[0]))
    //         .collect();
    //     queue.sort_by(|a, b| b.0.body.len().cmp(&a.0.body.len()));
    //     let mut queue = VecDeque::from(queue);
    //     v
    // }

    // This could be using team instead of index and then do the combined moves
    pub fn simulate_move(&self, our_team: bool) -> Vec<((Movement, Movement), Self)> {
        let idx = if our_team { self.team } else { self.opps };
        let mut moves = Vec::new();
        let mut alive = 0;
        for i in idx {
            if let Some(snake) = &self.snakes[i] {
                alive = i;
                moves.push(snake.get_safe_moves(self));
            }
        }
        let mut simulations = Vec::new();
        if moves.len() == 2 {
            let team_moves: Vec<(Movement, Movement)> = cartesian_move(&moves[0], &moves[1])
                .map(|(&m1, &m2)| (m1, m2))
                .collect();
            for m in team_moves {
                let mut next_pos = [
                    self.snakes[idx[0]].as_ref().unwrap().next_position(m.0),
                    self.snakes[idx[1]].as_ref().unwrap().next_position(m.1),
                ];
                if next_pos[0] == next_pos[1] {
                    continue;
                }
                let mut next_board = self.clone();
                next_board.snakes[idx[0]]
                    .as_mut()
                    .unwrap()
                    .body
                    .push_front(next_pos[0]);
                next_board.snakes[idx[1]]
                    .as_mut()
                    .unwrap()
                    .body
                    .push_front(next_pos[1]);
                for pos in next_board.food.iter() {
                    if pos == &next_pos[0] {
                        next_board.snakes[idx[0]].as_mut().unwrap().body.pop_back();
                    } else if pos == &next_pos[1] {
                        next_board.snakes[idx[1]].as_mut().unwrap().body.pop_back();
                    }
                }
                next_board
                    .food
                    .retain(|f| f != &next_pos[0] && f != &next_pos[1]);
                simulations.push((m, next_board));
            }
        } else {
            // use alive not idx
            for &m in moves[0].iter() {
                let next_pos = self.snakes[alive].as_ref().unwrap().next_position(m);
                let mut next_board = self.clone();
                next_board.snakes[alive]
                    .as_mut()
                    .unwrap()
                    .body
                    .push_front(next_pos);
                for pos in next_board.food.iter() {
                    if pos == &next_pos {
                        next_board.snakes[alive].as_mut().unwrap().body.pop_back();
                    }
                }
                next_board.food.retain(|f| f != &next_pos);
                simulations.push(((m, Movement::Down), next_board))
            }
        }
        simulations
    }

    fn kill_snakes(&mut self) {
        let mut kill_idxs = Vec::new();
        for (i, o_snake) in self.snakes.iter().enumerate() {
            if let Some(snake) = o_snake {
                if snake.health == 0
                    || snake.collision_with_snakes(&self, Movement::None)
                    || snake.collision_with_body(Movement::None)
                {
                    kill_idxs.push(i);
                    continue;
                }
            }
        }
        for idx in kill_idxs {
            self.snakes[idx] = None;
        }
    }
}

// Galenskap hehe
fn cartesian_move<'a>(
    v1: &'a [Movement],
    v2: &'a [Movement],
) -> impl Iterator<Item = (&'a Movement, &'a Movement)> + 'a {
    v1.iter().flat_map(move |m| std::iter::repeat(m).zip(v2))
}

#[derive(Debug, Clone)]
pub struct SimpleSnake {
    health: i32,
    body: VecDeque<Coord>,
}

impl SimpleSnake {
    pub fn from(snake: &Battlesnake) -> Self {
        SimpleSnake {
            health: snake.health,
            body: VecDeque::from(snake.body.clone()),
        }
    }
    // fn evaluate_value(&self) -> usize {
    //     self.body.len() * self.health
    // }

    fn get_safe_moves(&self, simple_board: &SimpleBoard) -> Vec<Movement> {
        let mut m_v = Movement::all();
        let head = &self.body[0];
        let neck = &self.body[1];
        if neck.x < head.x {
            let idx = m_v
                .iter()
                .position(|m| matches!(m, Movement::Left))
                .unwrap();
            m_v.remove(idx);
        }
        if neck.x > head.x {
            let idx = m_v
                .iter()
                .position(|m| matches!(m, Movement::Right))
                .unwrap();
            m_v.remove(idx);
        }
        if neck.y < head.y {
            let idx = m_v
                .iter()
                .position(|m| matches!(m, Movement::Down))
                .unwrap();
            m_v.remove(idx);
        }
        if neck.y > head.y {
            let idx = m_v.iter().position(|m| matches!(m, Movement::Up)).unwrap();
            m_v.remove(idx);
        }

        m_v.retain(|&m| {
            !simple_out_of_bounds(&head, &m)
                && !self.collision_with_body(m)
                && !self.collision_with_snakes(simple_board, m)
        });

        m_v
    }

    fn next_position(&self, movement: Movement) -> Coord {
        let head = &self.body[0];
        match movement {
            Movement::Up => Coord {
                x: head.x,
                y: head.y + 1,
            },
            Movement::Down => Coord {
                x: head.x,
                y: head.y - 1,
            },
            Movement::Left => Coord {
                x: head.x - 1,
                y: head.y,
            },
            Movement::Right => Coord {
                x: head.x + 1,
                y: head.y,
            },
            Movement::None => head.clone(),
        }
    }

    fn collision_with_body(&self, movement: Movement) -> bool {
        let next_pos = self.next_position(movement);
        self.body.iter().any(|b| b == &next_pos)
    }

    fn collision_with_snakes(&self, simple_board: &SimpleBoard, movement: Movement) -> bool {
        let next_pos = self.next_position(movement);
        simple_board.snakes.iter().any(|s| {
            s.as_ref()
                .map_or(false, |snake| snake.body.contains(&next_pos))
        })
    }
}

fn simple_out_of_bounds(coord: &Coord, movement: &Movement) -> bool {
    match movement {
        Movement::Up => coord.y == 10,
        Movement::Down => coord.y == 0,
        Movement::Left => coord.x == 0,
        Movement::Right => coord.x == 10,
        Movement::None => false,
    }
}

// fn search(board: &Board, game_info: &GameInfo, our_snake: &Battlesnake) {
//     let simple_snakes: Vec<SimpleSnake> = board.snakes.iter().map(|snake| SimpleSnake::from(snake, game_info.agent_ids.contains(&snake.id))).collect();
//     let simple_board = SimpleBoard::from(board, simple_snakes);
//     let our_snake_idx = simple_board.snakes.iter().enumerate().position(|(i,s)| s.body[0] == our_snake.head).expect("Our snake not found");
//     let diff = simple_board.oliver_heuristic();
//     // We want to make sure the opposing team does not catch up, i.e. minimize their score
//     if diff > 0 {
//
//     } else { // Minmax
//
//     }
//     //let alpha = MAX_VALUE;
//     //let beta = -MAX_VALUE;
//     //for (m1, m2, new_board) in root_board.simulate_move(our_team).iter() {
//     //    inner_search(new_board, 1, alpha, beta, !our_team);
//     //}
// }
//
// fn inner_search(
//     board: &SimpleBoard,
//     depth: u32,
//     alpha: i32,
//     beta: i32,
//     our_team: bool,
// ) -> Vec<usize> {
//     if depth == MAX_DEPTH {
//         return board.evaluate_board();
//     }
//     let mut values = Vec::new();
//     for (_, new_board) in board.simulate_move(our_team).iter() {
//         values.push(inner_search(new_board, depth + 1, alpha, beta, !our_team));
//     }
//     match our_team {
//         true => values.iter().max_by(|a, b| a[0].cmp(&b[0])),
//         false => values.iter().max_by(|a, b| a[1].cmp(&b[1])),
//     }
// }
