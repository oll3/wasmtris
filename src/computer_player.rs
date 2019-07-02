use rstris::figure::Figure;
use rstris::find_path::*;
use rstris::find_placement::*;
use rstris::movement::Movement;
use rstris::playfield::Playfield;
use rstris::position::Position;

use crate::game::Game;

pub trait ComputerType {
    fn init_eval(&mut self, pf: &Playfield, avail_placings: usize);
    fn eval_placing(&mut self, pf: &Playfield, fig: &Figure, pos: Position) -> f32;
}

struct EvalPosition {
    pos: Position,
    eval: f32,
}

pub struct ComputerPlayer<T>
where
    T: ComputerType,
{
    com_type: T,
    moves_per_down_step: f32,
    last_figure: Option<(Figure, Position)>,

    // Some cache variables
    avail_placings: Vec<Position>,
    find_path: FindPath,
    eval_placing: Vec<EvalPosition>,
    path: Vec<Movement>,
    moves_per_level: Vec<(i32, Movement)>,
}

impl<T> ComputerPlayer<T>
where
    T: ComputerType,
{
    pub fn new(moves_per_down_step: f32, com_type: T) -> Self {
        ComputerPlayer {
            moves_per_down_step,
            com_type,
            moves_per_level: Vec::new(),
            last_figure: None,
            eval_placing: Vec::new(),
            path: Vec::new(),
            find_path: FindPath::default(),
            avail_placings: Vec::new(),
        }
    }

    fn figure_move_event(&mut self, game: &mut Game, ticks: u64, _fig: &Figure, pos: Position) {
        let last_y = match self.last_figure {
            Some((_, ref last_fig_pos)) => last_fig_pos.y(),
            None => -1,
        };
        let y = pos.y();
        if y > last_y {
            let mut move_time = 0;
            while !self.moves_per_level.is_empty() && y == self.moves_per_level[0].0 {
                let movement = self.moves_per_level.remove(0);
                game.add_move(movement.1, ticks + move_time);
                move_time += (game.down_step_time() as f32 / self.moves_per_down_step) as u64;
            }
        }
    }

    fn new_figure_event(&mut self, _ticks: u64, pf: &Playfield, fig: &Figure, pos: Position) {
        // Find all possible positions where figure can be placed
        self.avail_placings.clear();
        find_placement(&mut self.avail_placings, &pf, fig);

        // Evaluate all placings to find the best one
        self.com_type.init_eval(&pf, self.avail_placings.len());
        self.eval_placing.clear();
        for avail_pos in &self.avail_placings {
            let eval = self.com_type.eval_placing(&pf, &fig, *avail_pos);
            let eval_pos = EvalPosition {
                pos: *avail_pos,
                eval,
            };
            self.eval_placing.push(eval_pos);
        }
        self.eval_placing
            .sort_by(|a, b| b.eval.partial_cmp(&a.eval).unwrap());

        // Find a path to first (and best) available placing
        self.path.clear();
        for eval_pos in &self.eval_placing {
            self.find_path.search(
                &mut self.path,
                &pf,
                fig,
                pos,
                eval_pos.pos,
                self.moves_per_down_step,
            );
            if !self.path.is_empty() {
                break;
            }
        }

        self.moves_per_level.clear();
        if !self.path.is_empty() {
            self.path.reverse();

            // Convert the path from being in exact Movements to
            // describe the sideways/rotational movements per height level
            path_to_moves_per_level(&mut self.moves_per_level, &self.path);
        }
    }

    pub fn act_on_game(&mut self, game: &mut Game, ticks: u64) {
        if self.last_figure != *game.current_figure() {
            // Figure has changed since last call
            let current_figure = game.current_figure().clone();
            if let Some((ref fig, pos)) = current_figure {
                if self.last_figure == None {
                    // Test if new figure
                    self.new_figure_event(ticks, game.playfield(), fig, pos);
                    self.figure_move_event(game, ticks, fig, pos);
                } else {
                    self.figure_move_event(game, ticks, fig, pos);
                }
            }
            self.last_figure = current_figure;
        }
    }
}

fn path_to_moves_per_level(moves: &mut Vec<(i32, Movement)>, path: &[Movement]) {
    moves.clear();
    let mut level: i32 = 0;
    for movement in path {
        if *movement == Movement::MoveDown {
            level += 1;
        } else {
            moves.push((level, *movement));
        }
    }
}
