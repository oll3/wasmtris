use rand;

use std::collections::BinaryHeap;
use std::collections::HashMap;

use rstris::figure::*;
use rstris::figure_pos::*;
use rstris::movement::*;
use rstris::playfield::*;
use rstris::pos_dir::*;

use crate::utils::*;

struct MoveQueue {
    // Queues of moves to be executed
    queue: BinaryHeap<MoveAndTime>,

    // Keep track of when last move was dequeued
    last_move_time: HashMap<Movement, u64>,
}

impl MoveQueue {
    fn new() -> Self {
        MoveQueue {
            queue: BinaryHeap::new(),
            last_move_time: HashMap::new(),
        }
    }

    fn add_move(&mut self, movement: Movement, ticks: u64) {
        let move_time = MoveAndTime {
            movement,
            time: ticks,
        };
        self.queue.push(move_time);
    }

    pub fn pop_next_move(&mut self, ticks: u64) -> Option<MoveAndTime> {
        if let Some(move_and_time) = self.queue.peek() {
            if move_and_time.time <= ticks {
                self.last_move_time
                    .insert(move_and_time.movement.clone(), move_and_time.time);
                return self.queue.pop();
            }
        }
        None
    }

    pub fn time_last_move(&self, movement: &Movement) -> u64 {
        if let Some(time) = self.last_move_time.get(movement) {
            *time
        } else {
            0
        }
    }

    pub fn time_since_move(&self, ticks: u64, movement: &Movement) -> i64 {
        ticks as i64 - self.time_last_move(movement) as i64
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

pub struct Game {
    pf: Playfield,
    down_step_time: u64,

    // All available figures
    available_figures: Vec<Figure>,

    // Next figure to be played
    next_figure: Figure,

    // Current figure being played
    current_figure: Option<FigurePos>,

    game_over: bool,

    // Queues of moves to be executed
    move_queue: MoveQueue,
}

impl Game {
    pub fn new(pf: Playfield, available_figures: Vec<Figure>, down_step_time: u64) -> Self {
        Game {
            pf,
            down_step_time,
            next_figure: Self::randomize_figure(&available_figures).clone(),
            available_figures,
            current_figure: None,
            game_over: false,
            move_queue: MoveQueue::new(),
        }
    }

    fn randomize_figure(figures: &[Figure]) -> &Figure {
        let next_figure = (rand::random::<u8>() % figures.len() as u8) as usize;
        &figures[next_figure]
        //let t: usize = (random() * figures.len() as f32) as usize; //rand::random();
        /*        let next_figure = t % figures.len();
        &figures[next_figure]*/
    }

    pub fn get_playfield(&self) -> &Playfield {
        &self.pf
    }

    pub fn get_current_figure(&self) -> &Option<FigurePos> {
        &self.current_figure
    }

    pub fn get_next_figure(&self) -> &Figure {
        &self.next_figure
    }

    pub fn game_is_over(&self) -> bool {
        self.game_over
    }

    pub fn get_down_step_time(&self) -> u64 {
        self.down_step_time
    }

    pub fn add_move(&mut self, movement: Movement, ticks: u64) {
        self.move_queue.add_move(movement, ticks);
    }

    fn execute_move(&mut self, movement: Movement) {
        if let Some(mut fig_pos) = self.current_figure.take() {
            let test_pos = PosDir::apply_move(fig_pos.get_position(), &movement);
            let collision = fig_pos.get_figure().test_collision(&self.pf, &test_pos);
            if collision && movement == Movement::MoveDown {
                // Figure has landed
                fig_pos.place(&mut self.pf);
            } else {
                if !collision {
                    // Move was executed
                    fig_pos.set_position(&test_pos);
                }
                self.current_figure = Some(fig_pos);
            }
        }
    }

    pub fn update(&mut self, ticks: u64) {
        if self.game_over {
            return;
        }
        if self.current_figure.is_some() {
            let time_since_down = self.move_queue.time_since_move(ticks, &Movement::MoveDown);
            if time_since_down >= self.down_step_time as i64 {
                // Let the figure fall
                self.add_move(Movement::MoveDown, ticks);
            }
            // Execute enqueued moves
            while let Some(move_and_time) = self.move_queue.pop_next_move(ticks) {
                self.execute_move(move_and_time.movement);
            }
        } else {
            self.move_queue.clear();

            // Throw away full lines
            let mut full_lines = self.pf.locked_lines();
            full_lines.sort();
            for line in &full_lines {
                self.pf.throw_line(*line);
            }

            // Place the next figure
            let next_figure = self.next_figure.clone();
            let pos = PosDir::new(((self.pf.width() / 2 - 1) as i32, 0, 0));
            if next_figure.test_collision(&self.pf, &pos) {
                console_log!("Game over");
                self.game_over = true;
            } else {
                self.next_figure = Self::randomize_figure(&self.available_figures).clone();
                let fig_pos = FigurePos::new(next_figure, pos);
                console_log!(
                    "New figure {} (next is {})",
                    fig_pos.get_figure().get_name(),
                    self.next_figure.get_name()
                );
                self.current_figure = Some(fig_pos);
            }
        }
    }
}
