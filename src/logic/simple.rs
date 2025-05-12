use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use rocket::form::validate::Contains;
use rocket::yansi::Paint;
use crate::{Battlesnake, Board, Coord};
use crate::logic::{collision_with_body, collision_with_snakes, get_safe_moves, out_of_bounds};

#[derive(Clone)]
enum Movement {
    Up,
    Down,
    Left,
    Right,
}
impl Movement {
    fn all() -> Vec<Movement> {
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
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub struct SimpleBoard {
    pub food: Vec<Coord>,
    pub snakes: Vec<SimpleSnake>,
}
impl SimpleBoard {
    pub fn from(board: &Board, snakes: Vec<SimpleSnake>) -> Self {
        SimpleBoard {
            food: board.food.clone(),
            snakes,
        }
    }
    fn evaluate_board(&self) -> Vec<usize> {
        let mut v = Vec::new();
        v.push(0);
        v.push(0);
        for snake in self.snakes.iter() {
            if snake.our_team {
                v[0] += snake.evaluate_value();
            } else {
                v[1] += snake.evaluate_value();
            }
        }
        v
    }
    // This could be using team instead of index and then do the combined moves
    fn simulate_move(&self, our_team: bool) -> Vec<(Movement, Movement, Self)> {
        let mut moves = Vec::new();
        for (idx, snake) in self.snakes.iter().enumerate().filter(|(i, s)| s.our_team == our_team) {
            moves.push((idx, snake.get_safe_moves(self)));
        }
        let snake_order = (moves[0].0, moves[1].0);
        let team_moves: Vec<(&Movement, &Movement)> = cartesian_move(&moves[0].1, &moves[1].1).collect();
        let mut simulations = Vec::new();
        for (m1, m2) in team_moves.iter() {
            let s1_np = self.snakes[snake_order.0].next_position(m1);
            let s2_np = self.snakes[snake_order.1].next_position(m2);
            if s1_np == s2_np {
                continue;
            }
            let mut new_board = self.clone();
            let mut s1_body = self.snakes[snake_order.0].body.clone();
            let mut s2_body = self.snakes[snake_order.1].body.clone();
            s1_body.push_front(s1_np);
            s2_body.push_front(s2_np);
            while let Some((idx, pos)) = new_board.food.iter().enumerate().next() {
                if pos == &s1_np {
                    new_board.food.remove(idx);
                    s1_body.pop_back();
                } else if pos == &s2_np { // can do else since s1_np != s2_np
                    new_board.food.remove(idx);
                    s2_body.pop_back();
                }
            }
            simulations.push(((*m1).clone(), (*m2).clone(), new_board));
        }
        simulations
    }
}

// Galenskap hehe
fn cartesian_move<'a>(v1: &'a[Movement], v2: &'a[Movement]) -> impl Iterator<Item = (&'a Movement, &'a Movement)> + 'a {
    v1.iter().flat_map(move |m| std::iter::repeat(m).zip(v2))
}

#[derive(Debug, Clone)]
pub struct SimpleSnake {
    our_team: bool,
    health: usize,
    body: VecDeque<Coord>,
}
impl SimpleSnake {
    pub fn from(snake: &Battlesnake, our_team: bool) -> Self {
        SimpleSnake {
            our_team,
            health: snake.health.clone() as usize,
            body: VecDeque::from(snake.body.clone());
        }
    }
    fn evaluate_value(&self) -> usize {
        self.body.len() * self.health
    }

    fn get_safe_moves(&self, simple_board: &SimpleBoard) -> Vec<Movement> {
        let mut m_v = Movement::all();
        let head = &self.body[0];
        let neck = &self.body[1];
        if neck.x < head.x  {
            let idx = m_v.iter().position(|m| matches!(m, Movement::Left)).unwrap();
            m_v.remove(idx);
        }
        if neck.x > head.x  {
            let idx = m_v.iter().position(|m| matches!(m, Movement::Right)).unwrap();
            m_v.remove(idx);
        }
        if neck.y < head.y  {
            let idx = m_v.iter().position(|m| matches!(m, Movement::Down)).unwrap();
            m_v.remove(idx);
        }
        if neck.y > head.y  {
            let idx = m_v.iter().position(|m| matches!(m, Movement::Up)).unwrap();
            m_v.remove(idx);
        }

        m_v.retain(|m| {!simple_out_of_bounds(&head, &m) && !self.collision_with_body(m) && !self.collision_with_snakes(simple_board, m)});

         m_v
    }

    fn next_position(&self, movement: &Movement) -> Coord {
        let head = &self.body[0];
        match movement {
            Movement::Up => {Coord {x: head.x, y:head.y + 1}}
            Movement::Down => {Coord {x: head.x, y:head.y - 1}}
            Movement::Left => {Coord {x: head.x - 1, y:head.y}}
            Movement::Right => {Coord {x: head.x + 1, y:head.y}}
        }
    }

    fn collision_with_body(&self, movement: &Movement) -> bool {
        let head = &self.body[0];
        let next_pos = self.next_position(movement);
        self.body.iter().any(|b| b == &next_pos)
    }

    fn collision_with_snakes(&self, simple_board: &SimpleBoard, movement: &Movement) -> bool {
        let head = &self.body[0];
        let next_pos = self.next_position(movement);
        simple_board.snakes.iter().any(|s| {
            s.body.iter().any(|b| b == &next_pos)
        })
    }
}

fn simple_out_of_bounds(coord: &Coord, movement: &Movement) -> bool {
    match movement {
        Movement::Up => {coord.y == 10}
        Movement::Down => {coord.y == 0}
        Movement::Left => {coord.x == 0}
        Movement::Right => {coord.x == 10}
    }
}


const MAX_DEPTH: u32 = 3;
const MAX_VALUE: i32 = 32000;

fn search(board: &Board) {
    let our_team = true;
    let root_board = SimpleBoard::from(board);
    let alpha = MAX_VALUE;
    let beta = -MAX_VALUE;
    for (m1, m2, new_board) in root_board.simulate_move(our_team).iter() {
        inner_search(new_board, 1, alpha, beta, !our_team);
    }
}

fn inner_search(
    board: &SimpleBoard,
    depth: u32,
    alpha: i32,
    beta: i32,
    our_team: bool,
) -> Vec<usize> {
    if depth == MAX_DEPTH {
        return board.evaluate_board();
    }
    let mut values = Vec::new();
    for (_, new_board) in board.simulate_move(our_team).iter() {
        values.push(inner_search(new_board, depth + 1, alpha, beta, !our_team));
    }
    match our_team {
        true => values.iter().max_by(|a, b| a[0].cmp(&b[0])),
        false => values.iter().max_by(|a, b| a[1].cmp(&b[1])),
    }
}
