use rstris::figure_pos::*;
use rstris::find_path::*;
use rstris::find_placement::*;
use rstris::movement::*;
use rstris::playfield::*;
use rstris::pos_dir::*;

use crate::game::Game;

use crate::utils::*;

pub trait ComputerType {
    fn init_eval(&mut self, pf: &Playfield, avail_placings: usize);
    fn eval_placing(&mut self, figure_pos: &FigurePos, pf: &Playfield) -> f32;
}

struct EvalPosition {
    pos: PosDir,
    eval: f32,
}

pub struct ComputerPlayer<T>
where
    T: ComputerType,
{
    com_type: T,
    moves_per_down_step: f32,
    path_per_height: Vec<Vec<MoveAndTime>>,
    last_figure: Option<FigurePos>,
}

impl<T> ComputerPlayer<T>
where
    T: ComputerType,
{
    pub fn new(moves_per_down_step: f32, com_type: T) -> Self {
        ComputerPlayer {
            moves_per_down_step,
            com_type,
            path_per_height: Vec::new(),
            last_figure: None,
        }
    }

    fn figure_move_event(&mut self, game: &mut Game, ticks: u64, fig_pos: &FigurePos) {
        let last_y = match self.last_figure {
            Some(ref last_fig_pos) => last_fig_pos.get_position().get_y(),
            None => -1,
        };
        let y = fig_pos.get_position().get_y();
        if y > last_y && y < self.path_per_height.len() as i32 {
            let moves = self.path_per_height[y as usize].clone();
            let time_between_moves = game.get_down_step_time() / (moves.len() + 1) as u64;
            let mut movement_time = ticks;
            for move_and_time in &moves {
                movement_time += time_between_moves;
                game.add_move(move_and_time.movement.clone(), movement_time);
            }
        }
    }

    fn new_figure_event(&mut self, _ticks: u64, pf: &Playfield, fig_pos: &FigurePos) {
        // Find all possible positions where figure can be placed
        let avail_placing = find_placement_quick(&pf, fig_pos);

        // Evaluate all placings to find the best one
        self.com_type.init_eval(&pf, avail_placing.len());
        let mut eval_placing: Vec<EvalPosition> = vec![];
        for p in &avail_placing {
            let eval_pos = FigurePos::new(fig_pos.get_figure().clone(), *p);
            let eval = self.com_type.eval_placing(&eval_pos, &pf);
            let eval_pos = EvalPosition { pos: *p, eval };
            eval_placing.push(eval_pos);
        }
        eval_placing.sort_by(|a, b| b.eval.partial_cmp(&a.eval).unwrap());

        // Find a path to first (and best) available placing
        let mut path = Vec::new();
        for eval_pos in &eval_placing {
            path = find_path(
                &pf,
                &fig_pos.get_figure(),
                &fig_pos.get_position(),
                &eval_pos.pos,
                self.moves_per_down_step,
            );
            if !path.is_empty() {
                break;
            }
        }

        self.path_per_height.clear();
        if !path.is_empty() {
            path.reverse();

            // Convert the path from being in exact Movements to
            // describe the sideways/rotational movements per height level
            self.path_per_height = path_to_per_height(path);
            console_log!(
                "Found path for figure {} ({} available placements)",
                fig_pos.get_figure().get_name(),
                avail_placing.len()
            );
        }
    }

    pub fn act_on_game(&mut self, game: &mut Game, ticks: u64) {
        let current_figure = game.get_current_figure().clone();
        if self.last_figure != current_figure {
            // Figure has changed since last call
            if let Some(ref fig_pos) = current_figure {
                if self.last_figure == None {
                    // Test if new figure
                    self.new_figure_event(ticks, game.get_playfield(), fig_pos);
                    self.figure_move_event(game, ticks, fig_pos);
                } else {
                    self.figure_move_event(game, ticks, fig_pos);
                }
            }
            self.last_figure = current_figure.clone();
        }
    }
}

fn path_to_per_height(path: Vec<(Movement, u64)>) -> Vec<Vec<MoveAndTime>> {
    let mut moves: Vec<Vec<MoveAndTime>> = Vec::new();
    let mut current_level: Vec<MoveAndTime> = Vec::new();
    for &(ref movement, time) in &path {
        if *movement == Movement::MoveDown {
            moves.push(current_level);
            current_level = Vec::new();
        } else {
            current_level.push(MoveAndTime {
                movement: movement.clone(),
                time,
            });
        }
    }
    if !current_level.is_empty() {
        moves.push(current_level);
    }
    moves
}
