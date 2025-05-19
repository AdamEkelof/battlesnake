//use rocket::futures::future::ok;

//use crate::logic::{collision_with_body, collision_with_snakes, get_safe_moves, out_of_bounds};
use crate::{Battlesnake, Board, Coord, GameInfo};
use log::info;
use serde::{Serialize, Serializer};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display};

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

#[derive(Clone, Copy)]
pub struct SnakeMove {
    pub id: usize,
    pub mv: Movement,
}

impl Debug for SnakeMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Movement {} by {}", self.mv, self.id))
    }
}

#[derive(Debug, Clone)]
pub struct SimpleBoard {
    pub food: Vec<Coord>,
    pub snakes: Vec<Option<SimpleSnake>>,
    team: [usize; 2],
    opps: [usize; 2],
    pub stored_fast_heuristic: Cell<Option<i32>>,
    pub stored_flood_fill_heuristic: Cell<Option<i32>>,
}
impl SimpleBoard {
    pub fn from(board: &Board, game_info: &GameInfo) -> Self {
        let mut simple_board = SimpleBoard {
            food: board.food.clone(),
            snakes: Vec::new(),
            team: [10; 2],
            opps: [10; 2],
            stored_fast_heuristic: Cell::new(None),
            stored_flood_fill_heuristic: Cell::new(None),
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
        while friendly_count < 2 {
            simple_board.team[friendly_count] = simple_board.snakes.len();
            simple_board.snakes.push(None);
            friendly_count += 1;
        }
        while enemy_count < 2 {
            simple_board.opps[enemy_count] = simple_board.snakes.len();
            simple_board.snakes.push(None);
            enemy_count += 1;
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

    pub fn heuristic(&self, fast: bool) -> i32 {
        let fast_heuristic: i32;
        if let Some(v) = self.stored_fast_heuristic.get() {
            fast_heuristic = v;
        }
        else {
            fast_heuristic = self.fast_heuristic();
        }
        if fast || fast_heuristic == i32::MIN || fast_heuristic == i32::MAX {
            return fast_heuristic;
        }
        
        let flood_fill_heuristic: i32;
        if let Some(v) = self.stored_flood_fill_heuristic.get() {
            flood_fill_heuristic = v;
        } else {
            flood_fill_heuristic = self.flood_fill().len() as i32;
        }
        
        return fast_heuristic + flood_fill_heuristic;
    }

    fn fast_heuristic(&self) -> i32 {
        if self.snakes.len() == 0 {
            self.stored_fast_heuristic.set(Some(0));
            return 0;
        }
        let mut health_value: i32 = 0;
        let mut length_value: i32 = 0;
        let mut death_value: i32 = 0;
        let mut dead_snake_count = 0;
        // lägg in så man är 1 längre än motståndare
        for f_idx in self.team {
            if f_idx >= self.snakes.len() {
                // Index out of range, treat as None
                dead_snake_count += 1;
                death_value -= 1;
            } else {
                match &self.snakes[f_idx] {
                    Some(snake) => {
                        length_value += snake.body.len() as i32;
                        if snake.health < 20 {
                            health_value -= 20 - snake.health;
                        }
                    }
                    None => {
                        //info!("Dead snake in our team");
                        dead_snake_count += 1;
                        death_value -= 1;
                    }
                }
            }
        }
        if dead_snake_count == 2 {
            self.stored_fast_heuristic.set(Some(i32::MIN));
            info!("both snakes dead");
            return i32::MIN;
        }
        dead_snake_count = 0;
        for e_idx in self.opps {
            if e_idx >= self.snakes.len() {
                // Index out of range, treat as None
                dead_snake_count += 1;
                death_value += 1;
            } else {
                match &self.snakes[e_idx] {
                    Some(snake) => {
                        length_value -= snake.body.len() as i32;
                        if snake.health < 20 {
                            health_value += 20 - snake.health;
                        }
                    }
                    None => {
                        dead_snake_count += 1;
                        death_value += 1;
                    }
                }
            }
        }
        if dead_snake_count == 2 {
            self.stored_fast_heuristic.set(Some(i32::MAX));
            return i32::MAX;
        }
        let v = health_value * 1 + length_value * 8 + death_value * 20;
        self.stored_fast_heuristic.set(Some(v));
        v
    }

    fn flood_fill_heuristic(&self) -> i32 {
        let flood_fill = self.flood_fill();
        let mut sum_value = 0;
        let mut danger_value = 0;
        for f_idx in self.team {
            if f_idx >= self.snakes.len() {
                // Index out of range, treat as None
                continue;
            }
            match &self.snakes[f_idx] {
                Some(snake) => {
                    let ff_size = flood_fill.get(&f_idx).unwrap().len() as i32;
                    sum_value += ff_size;
                    if ff_size < snake.body.len() as i32 {
                        danger_value -= snake.body.len() as i32 - ff_size;
                    }
                }
                None => {}
            }
        }
        for e_idx in self.opps {
            if e_idx >= self.snakes.len() {
                // Index out of range, treat as None
                continue;
            }
            match &self.snakes[e_idx] {
                Some(snake) => {
                    let ff_size = flood_fill.get(&e_idx).unwrap().len() as i32;
                    sum_value -= ff_size;
                    if ff_size < snake.body.len() as i32 {
                        danger_value += snake.body.len() as i32 - ff_size;
                    }
                }
                None => {}
            }
        }
        let v = sum_value * 1 + danger_value * 4;
        self.stored_flood_fill_heuristic.set(Some(v));
        v
    }

    fn flood_fill(&self) -> HashMap<usize, Vec<Coord>> {
        let mut mapping = HashMap::new();
        let mut queue: Vec<(usize, Coord)> = self
            .snakes
            .iter()
            .enumerate()
            .filter(|(_, o)| o.is_some())
            .map(|(i, s)| (i, s.as_ref().unwrap().body[0]))
            .collect();
        queue.sort_by_key(|&(i, _)| self.snakes[i].as_ref().unwrap().body.len());
        let mut queue = VecDeque::from(queue);
        for &(idx, _) in queue.iter() {
            mapping.insert(idx, Vec::new());
        }
        let mut visited = [false; 121];
        while let Some((i, coord)) = queue.pop_front() {
            if coord.x < 0 || coord.x > 10 || coord.y < 0 || coord.y > 10 {
                continue;
            }
            let arr_idx = (coord.y * 11 + coord.x) as usize;
            if visited[arr_idx] {
                continue;
            }
            visited[arr_idx] = true;
            mapping.get_mut(&i).unwrap().push(coord);
            for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                let nx = coord.x + dx;
                let ny = coord.y + dy;
                if nx >= 0 && nx < 11 && ny >= 0 && ny < 11 {
                    queue.push_back((i, Coord { x: nx, y: ny }));
                }
            }
        }
        mapping
    }

    // This could be using team instead of index and then do the combined moves
    pub fn simulate_move(&self, our_team: bool) -> Vec<([SnakeMove; 2], Self)> {
        // reset stored heuristics since snakes have moved
        self.stored_fast_heuristic.set(None);
        self.stored_flood_fill_heuristic.set(None);

        let idx = if our_team { self.team } else { self.opps };
        let mut moves = Vec::new();
        let mut alive = [false; 4];
        for i in idx {
            if i >= self.snakes.len() { 
                moves.push(vec![SnakeMove {
                    id: i,
                    mv: Movement::Down,
                }]);
            } else if let Some(snake) = &self.snakes[i] {
                alive[i] = true;
                let mut m = snake.get_safe_moves(self);
                if m.len() == 0 {
                    m.push(Movement::Down);
                }
                moves.push(
                    m.iter()
                        .map(|&m| SnakeMove { id: i, mv: m })
                        .collect::<Vec<SnakeMove>>(),
                );
            } else {
                moves.push(vec![SnakeMove {
                    id: i,
                    mv: Movement::Down,
                }]);
            }
        }

        let mut simulations = Vec::new();
        let team_moves: Vec<[SnakeMove; 2]> = cartesian_move(&moves[0], &moves[1]).collect();
        for m in team_moves {
            let next_pos = [
                if idx[0] <= alive.len() && alive[idx[0]] {
                    self.snakes[idx[0]]
                        .as_ref()
                        .unwrap()
                        .next_position(m.iter().find(|mv| mv.id == idx[0]).unwrap().mv)
                } else {
                    Coord { x: -2, y: -1 }
                },
                if idx[1] <= alive.len() && alive[idx[1]] {
                    self.snakes[idx[1]]
                        .as_ref()
                        .unwrap()
                        .next_position(m.iter().find(|mv| mv.id == idx[1]).unwrap().mv)
                } else {
                    Coord { x: -1, y: -2 }
                },
            ];
            if next_pos[0] == next_pos[1] {
                continue;
            }

            let mut next_board = self.clone();
            if idx[0] <= alive.len() && alive[idx[0]] {
                next_board.snakes[idx[0]]
                    .as_mut()
                    .unwrap()
                    .body
                    .push_front(next_pos[0]);

                if !next_board.food.contains(&next_pos[0]) {
                    next_board.snakes[idx[0]].as_mut().unwrap().body.pop_back();
                }
            }
            if idx[1] <= alive.len() && alive[idx[1]] {
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

            if !our_team { next_board.kill_snakes(); }

            //info!("Killed snakes: \n{}", next_board);

            simulations.push((m, next_board));
        }

        // Det här behöver ersättas med lösning för att låta en leva om de "måste" huvudkrocka
        // Det eller att det inte finns några safe moves är enda sätten simulations kan ge 0 moves
        if simulations.len() == 0 {
            return vec![(
                [
                    SnakeMove {
                        id: idx[0],
                        mv: Movement::Down,
                    },
                    SnakeMove {
                        id: idx[1],
                        mv: Movement::Down,
                    },
                ],
                self.clone(),
            )];
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
                let coord = Coord {
                    x: x as i32,
                    y: y as i32,
                };
                let piece: String = if self.food.contains(&coord) {
                    "f".to_string()
                } else if let Some(snake) = self
                    .snakes
                    .iter()
                    .filter_map(|s| s.as_ref())
                    .find(|s| s.body.contains(&coord))
                {
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
    v1: &'a [SnakeMove],
    v2: &'a [SnakeMove],
) -> impl Iterator<Item = [SnakeMove; 2]> + 'a {
    let ret = v1
        .iter()
        .flat_map(move |&m1| v2.iter().map(move |&m2| [m1, m2]));
    //info!("Cartesian product from {:?} and {:?} = {:?}", v1, v2, ret.clone().collect::<Vec<_>>());
    ret
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
        for idx in simple_board.team {
            if let Some(snake) = &simple_board.snakes[idx] {
                if let Some(&pos) = snake.body.back() {
                    if pos == next_pos {
                        return (false, false);
                    }
                }
            }
        }
        for idx in simple_board.opps {
            if let Some(snake) = &simple_board.snakes[idx] {
                if let Some(&pos) = snake.body.back() {
                    if pos == next_pos {
                        let head = snake.body.front().unwrap();
                        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                            let nx = head.x + dx;
                            let ny = head.y + dy;
                            if nx >= 0 && nx < 11 && ny >= 0 && ny < 11 {
                                let new_coord = Coord { x: nx, y: ny };
                                if simple_board.food.contains(&new_coord) {
                                    return (true, true);
                                }
                            }
                        }
                    }
                }
            }
        }
        simple_board
            .snakes
            .iter()
            .filter(|s| {
                if let Some(snake) = s {
                    snake != self
                } else {
                    false
                }
            })
            .for_each(|s| {
                let collision = s
                    .as_ref()
                    .map_or(false, |snake| snake.body.contains(&next_pos));
                if collision{
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
