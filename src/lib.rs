#[macro_use]
mod utils;

mod computer_player;

mod draw;
mod game;
mod jitter_computer;

use rstris::block::*;
use rstris::figure::*;
use rstris::playfield::Playfield;

use std::f64;
use wasm_bindgen::prelude::*;

use crate::computer_player::*;
use crate::game::*;

use crate::jitter_computer::*;
use crate::utils::*;

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

#[wasm_bindgen]
pub struct GameContext {
    game: Game,
    computer_player: ComputerPlayer<JitterComputer>,
    draw: draw::Draw,
}

#[wasm_bindgen]
impl GameContext {
    pub fn new(canvas_id: &str, width: u32, height: u32) -> Self {
        set_panic_hook();
        let figure_list = init_figures();
        let pf = Playfield::new("Playfield 1", width, height);
        console_log!("Create game context (draw on: {})", canvas_id);
        GameContext {
            game: Game::new(pf, figure_list, 10),
            computer_player: ComputerPlayer::new(2.0, JitterComputer::new()),
            draw: draw::Draw::new(canvas_id, width, height),
        }
    }

    pub fn update(&mut self, ticks: f64) {
        let ticks: u64 = (ticks.round() as i64) as u64;
        self.computer_player.act_on_game(&mut self.game, ticks);
        self.game.update(ticks);
    }

    fn block_color(id: u8) -> (f32, f32, f32, f32) {
        match id {
            0 => (0.2, 0.1, 0.1, 1.0),
            1 => (0.2, 0.5, 0.3, 1.0),
            2 => (0.4, 0.2, 0.1, 1.0),
            3 => (0.4, 0.2, 0.5, 1.0),
            4 => (0.2, 0.1, 0.6, 1.0),
            5 => (0.6, 0.2, 0.1, 1.0),
            6 => (0.2, 0.6, 0.3, 1.0),
            7 => (0.7, 0.2, 0.6, 1.0),
            _ => (0.2, 0.2, 0.3, 1.0),
        }
    }

    pub fn draw(&mut self) {
        let pf = self.game.playfield();
        for y in 0..pf.height() as i32 {
            for x in 0..pf.width() as i32 {
                let block = pf.get_block((x, y).into());
                if let Block::Set(ref id) = block {
                    self.draw
                        .set_block(x as u32, y as u32, Self::block_color(*id));
                } else {
                    self.draw
                        .set_block(x as u32, y as u32, Self::block_color(0));
                }
            }
        }
        match self.game.current_figure() {
            Some((ref fig, pos)) => {
                let face = fig.face(pos.dir());
                for (x, y, id) in face {
                    self.draw.set_block(
                        (i32::from(*x) + pos.x()) as u32,
                        (i32::from(*y) + pos.y()) as u32,
                        Self::block_color(*id),
                    );
                }
            }
            None => {}
        }
        self.draw.draw_blocks();
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
