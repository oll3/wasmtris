#[macro_use]
mod utils;

mod computer_player;

mod draw;
mod game;
use rstris::block::*;
use rstris::figure::*;
use rstris::figure_pos::*;
use rstris::playfield::*;
use rstris::position::*;

use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::computer_player::*;
use crate::game::*;
use crate::utils::*;

use web_sys;

macro_rules! bl {
    ($x:expr) => {
        match $x {
            0 => Block::Clear,
            _ => Block::Set($x),
        }
    };
}

//
// Build list of figures
//
#[allow(clippy::cognitive_complexity)]
fn init_figures() -> Vec<Figure> {
    let mut figure_list: Vec<Figure> = Vec::new();
    figure_list.push(Figure::new_from_face(
        "1",
        &[
            &[bl!(0), bl!(0), bl!(0)],
            &[bl!(1), bl!(1), bl!(1)],
            &[bl!(0), bl!(1), bl!(0)],
        ],
    ));
    figure_list.push(Figure::new_from_face(
        "2",
        &[
            &[bl!(0), bl!(0), bl!(0)],
            &[bl!(2), bl!(2), bl!(2)],
            &[bl!(0), bl!(0), bl!(2)],
        ],
    ));
    figure_list.push(Figure::new_from_face(
        "3",
        &[
            &[bl!(0), bl!(0), bl!(3)],
            &[bl!(3), bl!(3), bl!(3)],
            &[bl!(0), bl!(0), bl!(0)],
        ],
    ));
    figure_list.push(Figure::new_from_face(
        "4",
        &[&[bl!(4), bl!(4)], &[bl!(4), bl!(4)]],
    ));
    figure_list.push(Figure::new_from_face(
        "5",
        &[&[bl!(0), bl!(5), bl!(5)], &[bl!(5), bl!(5), bl!(0)]],
    ));
    figure_list.push(Figure::new_from_face(
        "6",
        &[&[bl!(6), bl!(6), bl!(0)], &[bl!(0), bl!(6), bl!(6)]],
    ));
    figure_list.push(Figure::new_from_face(
        "7",
        &[
            &[bl!(0), bl!(7), bl!(0)],
            &[bl!(0), bl!(7), bl!(0)],
            &[bl!(0), bl!(7), bl!(0)],
            &[bl!(0), bl!(7), bl!(0)],
        ],
    ));
    figure_list
}

fn get_max_figure_dimensions(figure_list: &[Figure]) -> (u32, u32) {
    let mut max_width: u32 = 0;
    let mut max_height: u32 = 0;
    for fig in figure_list {
        for face in fig.faces() {
            if face.width() as u32 > max_width {
                max_width = face.width() as u32;
            }
            if face.height() as u32 > max_height {
                max_height = face.height() as u32;
            }
        }
    }
    (max_width, max_height)
}

fn get_pf_row_jitter(pf: &Playfield) -> u32 {
    let mut jitter = 0;
    for y in 0..(pf.height() as i32) {
        let mut last_state = pf.block_is_set(Position::new((-1, y)));
        for x in 0..=(pf.width() as i32) {
            let state = pf.block_is_set(Position::new((x, y)));
            if last_state != state {
                last_state = state;
                jitter += 1;
            }
        }
    }
    jitter
}
fn get_pf_col_jitter(pf: &Playfield) -> u32 {
    let mut jitter = 0;
    for x in 0..(pf.width() as i32) {
        let mut last_state = pf.block_is_set(Position::new((x, 0)));
        for y in 0..((pf.height() + 1) as i32) {
            let state = pf.block_is_set(Position::new((x, y)));
            if last_state != state {
                last_state = state;
                jitter += 1;
            }
        }
    }
    jitter
}
fn get_pf_avg_height(pf: &Playfield) -> f32 {
    let mut total_height = 0;
    for x in 0..(pf.width() as i32) {
        for y in 0..(pf.height() as i32) {
            if pf.block_is_set(Position::new((x, y))) {
                total_height += pf.height() as i32 - y;
                break;
            }
        }
    }
    total_height as f32 / pf.width() as f32
}
fn get_pf_max_height(pf: &Playfield) -> i32 {
    let mut max_height = 0;
    for x in 0..(pf.width() as i32) {
        for y in 0..(pf.height() as i32) {
            let height = pf.height() as i32 - y;
            if pf.block_is_set(Position::new((x, y))) && height > max_height {
                max_height = height;
            }
        }
    }
    max_height
}

struct JitterComputer {
    pre_col_jitter: i32,
    pre_row_jitter: i32,
    pre_voids: i32,
    pre_avg_height: f32,
    avg_height_factor: f32,
    pre_max_height: i32,
    pre_locked_lines: i32,
}
impl JitterComputer {
    fn new() -> Self {
        JitterComputer {
            pre_col_jitter: 0,
            pre_row_jitter: 0,
            pre_voids: 0,
            pre_avg_height: 0.0,
            avg_height_factor: 0.0,
            pre_max_height: 0,
            pre_locked_lines: 0,
        }
    }
}
impl ComputerType for JitterComputer {
    fn init_eval(&mut self, pf: &Playfield, _: usize) {
        self.pre_voids = pf.count_voids() as i32;
        self.pre_col_jitter = get_pf_col_jitter(pf) as i32;
        self.pre_row_jitter = get_pf_row_jitter(pf) as i32;
        self.pre_avg_height = get_pf_avg_height(pf);
        self.pre_max_height = get_pf_max_height(pf);
        self.avg_height_factor = self.pre_avg_height / pf.height() as f32;
        self.pre_locked_lines = pf.count_locked_lines() as i32;
    }

    fn eval_placing(&mut self, fig_pos: &FigurePos, pf: &Playfield) -> f32 {
        let mut pf = pf.clone();
        fig_pos.place(&mut pf);
        let avg_height = get_pf_avg_height(&pf);
        let mut full_lines = pf.locked_lines();
        full_lines.sort();

        let full_lines_score = if full_lines.len() >= 4 {
            // Great things!
            10.0
        } else if full_lines.len() == 1 {
            // Single full line - Not too bad but still a bit unnecessary
            -2.0
        } else if full_lines.len() >= 2 {
            // 2 or 3 lines should be avoided as long as the avarage playfield height is low
            let factor = 1.0 - (avg_height as f32 / pf.height() as f32);
            (4 - full_lines.len()) as f32 * -factor * 3.0
        } else {
            // No full lines - Don't care
            0.0
        };

        for line in &full_lines {
            pf.throw_line(*line);
        }

        let bottom_block = fig_pos.lowest_block() / 2;

        // Measure playfield jitter. Lower jitter is better.
        let col_jitter = get_pf_col_jitter(&pf) as i32 - self.pre_col_jitter;
        let row_jitter = get_pf_row_jitter(&pf) as i32 - self.pre_row_jitter;
        let jitter_score = -(col_jitter * 3 + row_jitter / 2);

        (bottom_block + jitter_score) as f32 + full_lines_score
    }
}

#[wasm_bindgen]
pub struct GameContext {
    game: Game,
    computer_player: ComputerPlayer<JitterComputer>,
    canvas_id: String,
}

#[wasm_bindgen]
impl GameContext {
    pub fn new(canvas_id: &str, width: u32, height: u32) -> Self {
        set_panic_hook();
        let figure_list = init_figures();
        let pf = Playfield::new("Playfield 1", width, height);
        console_log!("Create game context (draw on: {})", canvas_id);
        GameContext {
            game: Game::new(pf, figure_list, 0_10),
            computer_player: ComputerPlayer::new(2.0, JitterComputer::new()),
            canvas_id: canvas_id.to_owned(),
        }
    }

    pub fn update(&mut self, ticks: f64) {
        let ticks: u64 = (ticks.round() as i64) as u64;
        self.computer_player.act_on_game(&mut self.game, ticks);
        self.game.update(ticks);
    }

    pub fn draw(&mut self) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(&self.canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let mut ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        draw::draw_playfield(&mut ctx, self.game.get_playfield());
        draw::draw_figure(&mut ctx, self.game.get_current_figure());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
