use rstris::figure_pos::*;
use rstris::playfield::*;

use crate::computer_player::*;

fn get_pf_row_jitter(pf: &Playfield) -> u32 {
    let mut jitter = 0;
    for row in pf.blocks().row_iter() {
        // For each row...
        let mut last_state = true;
        for block in row {
            // ...measure its jitter
            let state = block.is_set();
            if last_state != state {
                last_state = state;
                jitter += 1;
            }
        }
        if !last_state {
            jitter += 1;
        }
    }
    jitter
}
fn get_pf_col_jitter(pf: &Playfield) -> u32 {
    let mut jitter = 0;
    for x in 0..(pf.width() as i32) {
        // For each column...
        let mut last_state = false;
        for y in 0..(pf.height() as i32) {
            // ...measure its jitter
            let block = pf.blocks().get((x, y).into());
            let state = block.is_set();
            if last_state != state {
                last_state = state;
                jitter += 1;
            }
        }
        if !last_state {
            jitter += 1;
        }
    }
    jitter
}

fn get_pf_max_height(pf: &Playfield) -> u32 {
    for (y, row) in pf.blocks().row_iter().enumerate() {
        // For each row...
        if row.iter().any(|b| b.is_set()) {
            return y as u32;
        }
    }
    0
}

fn get_pf_avg_height(pf: &Playfield) -> f32 {
    let mut total_height = 0;
    for x in 0..(pf.width() as i32) {
        for y in 0..(pf.height() as i32) {
            let block = pf.blocks().get((x, y).into());
            if block.is_set() {
                total_height += pf.height() as i32 - y;
                break;
            }
        }
    }
    total_height as f32 / pf.width() as f32
}

pub struct JitterComputer {
    pre_col_jitter: i32,
    pre_row_jitter: i32,
    pre_voids: i32,
    pre_avg_height: f32,
    avg_height_factor: f32,
    pre_max_height: u32,
    pre_locked_lines: i32,
    pf: Option<Playfield>,
}
impl JitterComputer {
    pub fn new() -> Self {
        JitterComputer {
            pf: None,
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
        if self.pf.is_none() {
            self.pf = Some(pf.clone())
        }
        self.pre_voids = pf.count_voids() as i32;
        self.pre_col_jitter = get_pf_col_jitter(pf) as i32;
        self.pre_row_jitter = get_pf_row_jitter(pf) as i32;
        self.pre_avg_height = get_pf_avg_height(pf);
        self.pre_max_height = get_pf_max_height(pf);
        self.avg_height_factor = self.pre_avg_height / pf.height() as f32;
        self.pre_locked_lines = pf.count_locked_lines() as i32;
    }

    fn eval_placing(&mut self, fig_pos: &FigurePos, current_pf: &Playfield) -> f32 {
        if let Some(ref mut pf) = self.pf {
            pf.copy(current_pf);
            fig_pos.place(pf);
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
        } else {
            0.0
        }
    }
}
