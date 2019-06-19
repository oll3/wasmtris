use rstris::block::*;
use rstris::figure_pos::*;
use rstris::playfield::*;
use rstris::position::*;

pub fn draw_block(
    ctx: &mut web_sys::CanvasRenderingContext2d,
    block: &Block,
    x: f64,
    y: f64,
    size: f64,
) {
    let x1 = size * x;
    let y1 = size * y;
    match block {
        Block::Set(ref id) => match id {
            0 => ctx.set_fill_style(&("rgba(20, 20, 30, 1)".into())),
            1 => ctx.set_fill_style(&("rgba(50, 60, 30, 1)".into())),
            2 => ctx.set_fill_style(&("rgba(100, 180, 30, 1)".into())),
            3 => ctx.set_fill_style(&("rgba(190, 20, 230, 1)".into())),
            4 => ctx.set_fill_style(&("rgba(230, 40, 30, 1)".into())),
            5 => ctx.set_fill_style(&("rgba(10, 120, 30, 1)".into())),
            6 => ctx.set_fill_style(&("rgba(10, 120, 130, 1)".into())),
            7 => ctx.set_fill_style(&("rgba(211, 68, 0, 1)".into())),
            _ => ctx.set_fill_style(&("rgba(0, 0, 0, 1)".into())),
        },
        Block::Clear => ctx.set_fill_style(&("rgba(90, 186, 243, 1)".into())),
    }
    ctx.fill_rect(x1, y1, size, size);
    ctx.stroke_rect(x1, y1, size, size);
}

pub fn draw_playfield(ctx: &mut web_sys::CanvasRenderingContext2d, pf: &Playfield) {
    ctx.set_stroke_style(&("rgba(170, 190, 180, 1)".into()));
    //ctx.set_fill_style(&("rgba(20, 20, 30, 1)".into()));
    ctx.set_line_width(0.5);
    for y in 0..pf.height() {
        for x in 0..pf.width() {
            let block = pf.get_block(Position::from((x as i32, y as i32)));
            draw_block(ctx, block, x as f64, y as f64, 32.0);
        }
    }
}

pub fn draw_figure(ctx: &mut web_sys::CanvasRenderingContext2d, figure_pos: &Option<FigurePos>) {
    match figure_pos {
        Some(ref fig_pos) => {
            let face = fig_pos.get_face();
            for y in 0..face.height() as i32 {
                for x in 0..face.width() as i32 {
                    let block = face.get((x, y).into());
                    if block.is_set() {
                        draw_block(
                            ctx,
                            block,
                            (x + fig_pos.get_position().x) as f64,
                            (y + fig_pos.get_position().y) as f64,
                            32.0,
                        );
                    }
                }
            }
        }
        None => {}
    }
}
