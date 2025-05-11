use std::fmt::Display;
use rocket::yansi::Paint;
use crate::{Battlesnake, Board, Coord};
use crate::logic::get_safe_moves;

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
struct SimpleBoard {
    food: Vec<Coord>,
    snakes: Vec<SimpleSnake>,
}
impl SimpleBoard {
    fn from(board: &Board) -> Self {
        // TODO: select our team snakes and send that info to create SimpleSnake
        let snakes = board
            .snakes
            .iter()
            .map(|s| SimpleSnake::from(s, true))
            .collect();
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
    fn simulate_move(&self, our_team: bool) -> Vec<(Movement, Self)> {
        let mut moves = Vec::new();
        for snake in self.snakes.iter() {
            moves.push(get_safe_moves(self, snake));
        }
        cartesian_move(moves[0], moves[1]);
        let snake = self.snakes.get(idx).expect("bad index of snake");
        // TODO: make get_safe_moves work with simple classes instead maybe
        let moves = get_safe_moves(self, snake);
        let mut simulations = Vec::new();
        for m in moves.iter() {
            simulations.push((m.clone(), self.clone()));
            let board = simulations.last_mut().unwrap();
            // Apply the move to the board...
        }
        simulations
    }
}

// Galenskap hehe
fn cartesian_move<'a>(v1: &'a[Movement], v2: &'a[Movement]) -> impl Iterator<Item = (&'a Movement, &'a Movement)> + 'a {
    v1.iter().flat_map(move |m| std::iter::repeat(m).zip(v2))
}

#[derive(Debug, Clone)]
struct SimpleSnake {
    our_team: bool,
    health: usize,
    body: Vec<Coord>,
}
impl SimpleSnake {
    fn from(snake: &Battlesnake, our_team: bool) -> Self {
        SimpleSnake {
            our_team,
            health: snake.health.clone() as usize,
            body: snake.body.clone(),
        }
    }
    fn evaluate_value(&self) -> usize {
        self.body.len() * self.health
    }
}

const MAX_DEPTH: u32 = 3;
const MAX_VALUE: i32 = 32000;

fn search(board: &Board) {
    let our_team = true;
    let root_board = SimpleBoard::from(board);
    let alpha = MAX_VALUE;
    let beta = -MAX_VALUE;
    for (movement, new_board) in root_board.simulate_move(our_team).iter() {
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
        values.add(inner_search(new_board, depth + 1, alpha, beta, !our_team));
    }
    match our_team {
        true => values.iter().max_by(|a, b| a[0].cmp(&b[0])),
        false => values.iter().max_by(|a, b| a[1].cmp(&b[1])),
    }
}
