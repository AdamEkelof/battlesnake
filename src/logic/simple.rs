//use rocket::futures::future::ok;

//use crate::logic::{collision_with_body, collision_with_snakes, get_safe_moves, out_of_bounds};
use crate::{Battlesnake, Board, Coord, GameInfo};
use std::collections::{/*HashMap,*/ VecDeque};
use std::fmt::Display;
use serde::{Serialize, Serializer};
use std::cell::Cell;
use log::info;

#[derive(Copy, Clone, Debug)]
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
impl Serialize for Movement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = match self {
            Movement::Up => String::from("up"),
            Movement::Down => String::from("down"),
            Movement::Left => String::from("left"),
            Movement::Right => String::from("right"),
            Movement::None => String::from("no movement made somehow"),
        };
        serializer.serialize_str(&str)
    }
}

#[derive(Debug, Clone)]
pub struct SimpleBoard {
    pub food: Vec<Coord>,
    pub snakes: Vec<Option<SimpleSnake>>,
    team: [usize; 2],
    opps: [usize; 2],
    pub stored_heuristic: Cell<Option<i32>>,
}
impl SimpleBoard {
    pub fn from(board: &Board, game_info: &GameInfo) -> Self {
        let mut simple_board = SimpleBoard {
            food: board.food.clone(),
            snakes: Vec::new(),
            team: [0; 2],
            opps: [0; 2],
            stored_heuristic: Cell::new(None),
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
        if let Some(v) = self.stored_heuristic.get() {
            info!("Using stored heuristic: {}", v);
            return v;
        }
        if self.snakes.len() == 0 {
            self.stored_heuristic.set(Some(0));
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
                    info!("Dead snake in our team");
                    dead_snake_count += 1;
                    v -= 10;
                }
            }
        }
        if dead_snake_count == 2 {
            self.stored_heuristic.set(Some(i32::MIN));
            info!("both snakes dead");
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
            self.stored_heuristic.set(Some(i32::MAX));
            return i32::MAX;
        }
        self.stored_heuristic.set(Some(v));
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
    pub fn simulate_move(&self, our_team: bool) -> Vec<([Movement; 2], Self)> {
        let idx = if our_team { self.team } else { self.opps };
        let mut moves = Vec::new();
        let mut alive = [false; 4];
        for i in idx {
            if let Some(snake) = &self.snakes[i] {
                alive[i] = true;
                let mut m = snake.get_safe_moves(self);
                if m.len() == 0 {
                    m.push(Movement::Down);
                }
                moves.push(m);
            } else {
                moves.push(vec![Movement::Down]);
            }
        }

        let mut simulations = Vec::new();
        let team_moves: Vec<[Movement; 2]> = cartesian_move(&moves[0], &moves[1]).collect();
        for m in team_moves {
            let next_pos = [
                if alive[idx[0]] {self.snakes[idx[0]].as_ref().unwrap().next_position(m[0])} else {Coord { x: -2, y: -1 }},
                if alive[idx[1]] {self.snakes[idx[1]].as_ref().unwrap().next_position(m[1])} else {Coord { x: -1, y: -2 }},
            ];
            if next_pos[0] == next_pos[1] {
                continue;
            }
            
            let mut next_board = self.clone();
            if alive[idx[0]] {
                next_board.snakes[idx[0]]
                    .as_mut()
                    .unwrap()
                    .body
                    .push_front(next_pos[0]);

                if !next_board.food.contains(&next_pos[0]) {
                    next_board.snakes[idx[0]].as_mut().unwrap().body.pop_back();
                }
            }
            if alive[idx[1]] {
                next_board.snakes[idx[1]]
                    .as_mut()
                    .unwrap()
                    .body
                    .push_front(next_pos[1]);

                if !next_board.food.contains(&next_pos[1]) {
                    next_board.snakes[idx[1]].as_mut().unwrap().body.pop_back();
                }
            }

            next_board
                .food
                .retain(|f| f != &next_pos[0] && f != &next_pos[1]);

            //info!("Simulating move: {:?} -> \n{}", m, next_board);              

            next_board.kill_snakes();

            //info!("Killed snakes: \n{}", next_board);

            simulations.push((m, next_board));
        }
        if simulations.len() == 0 {
            return vec![([Movement::Down; 2], self.clone())];
        }
        simulations
    }

    fn kill_snakes(&mut self) {
        let mut kill_idxs = Vec::new();
        for (i, o_snake) in self.snakes.iter().enumerate() {
            if let Some(snake) = o_snake {
                if snake.health == 0
                    || snake.collision_with_snakes(&self, Movement::None).1
                    || simple_out_of_bounds(&snake.body[0], &Movement::None)
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
impl std::fmt::Display for SimpleBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        /* build board representation string */
        let mut board: String = "\n|:---------:|".to_owned();
        for y in (0..11).rev() {
            board += "\n|";
            for x in 0..11 {
                let coord = Coord { x: x as i32, y: y as i32 };
                let piece: String = if self.food.contains(&coord) {
                    "f".to_string()
                } else if let Some(snake) = self.snakes.iter().filter_map(|s| s.as_ref()).find(|s| s.body.contains(&coord)) {
                    if snake.body[0] == coord {
                        "h".to_string()
                    } else {
                        "s".to_string()
                    }
                } else {
                    " ".to_string()
                };
                board += &piece;
            }
            board += "|";
        }
        board += "\n|:---------:|";

        write!(f, "{}", board)
    }
}

// Galenskap hehe
fn cartesian_move<'a>(
    v1: &'a [Movement],
    v2: &'a [Movement],
) -> impl Iterator<Item = [Movement; 2]> + 'a {
    v1.iter().flat_map(move |&m1| v2.iter().map(move |&m2| [m1, m2]))
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
                && !self.collision_with_snakes(simple_board, m).0
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

    /// Checks if the head of the snake is intersecting the body of another snake.
    /// Returns two bools, collision check and death check respectively.
    /// TODO: maybe only collisions that kill are interesting to report...
    fn collision_with_snakes(
        &self,
        simple_board: &SimpleBoard,
        movement: Movement,
    ) -> (bool, bool) {
        let next_pos = self.next_position(movement);
        let mut any_collision = false;
        let mut dead = false;
        simple_board.snakes.iter().filter(|s| {
            if let Some(snake) = s {
                snake != self
            } else {
                false
            }
        }).for_each(|s| {
            let collision = s
                .as_ref()
                .map_or(false, |snake| snake.body.contains(&next_pos));
            if collision {
                any_collision = true;
                // Only check length if collision is with the head, otherwise always dead
                if s.as_ref().unwrap().body[0] == next_pos {
                    if s.as_ref().unwrap().body.len() >= self.body.len() {
                        dead = true;
                    }
                } else {
                    dead = true;
                }
            }
        });
        (any_collision, dead)
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
