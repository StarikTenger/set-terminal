use colored::{Color as TermColor, Colorize};

use crate::card::{Card, Color, Shading, Shape};

const COLUMNS: usize = 4;
const CARD_INNER_WIDTH: usize = 7;

fn glyph(shape: Shape, shading: Shading) -> char {
    match (shape, shading) {
        (Shape::Diamond, Shading::Solid) => '◆',
        (Shape::Diamond, Shading::Striped) => '⬖',
        (Shape::Diamond, Shading::Empty) => '◇',
        (Shape::Squiggle, Shading::Solid) => '■',
        (Shape::Squiggle, Shading::Striped) => '◧',
        (Shape::Squiggle, Shading::Empty) => '□',
        (Shape::Oval, Shading::Solid) => '●',
        (Shape::Oval, Shading::Striped) => '◐',
        (Shape::Oval, Shading::Empty) => '○',
    }
}

fn term_color(color: Color) -> TermColor {
    match color {
        Color::Red => TermColor::Red,
        Color::Green => TermColor::Green,
        Color::Purple => TermColor::Magenta,
    }
}

fn card_content(card: &Card) -> String {
    let g = glyph(card.shape, card.shading);
    match card.number.count() {
        1 => format!("   {g}   "),
        2 => format!("  {g} {g}  "),
        _ => format!(" {g} {g} {g} "),
    }
}

pub fn render_board(cards: &[Card]) {
    for (row_idx, chunk) in cards.chunks(COLUMNS).enumerate() {
        let border_top: String = "┌".to_string() + &"─".repeat(CARD_INNER_WIDTH) + "┐";
        let border_bottom: String = "└".to_string() + &"─".repeat(CARD_INNER_WIDTH) + "┘";

        let tops: Vec<String> = chunk.iter().map(|_| border_top.clone()).collect();
        println!("{}", tops.join(" "));

        let mids: Vec<String> = chunk
            .iter()
            .map(|c| {
                let content = card_content(c).color(term_color(c.color)).to_string();
                format!("│{content}│")
            })
            .collect();
        println!("{}", mids.join(" "));

        let bottoms: Vec<String> = chunk.iter().map(|_| border_bottom.clone()).collect();
        println!("{}", bottoms.join(" "));

        let labels: Vec<String> = chunk
            .iter()
            .enumerate()
            .map(|(col, _)| {
                let index = row_idx * COLUMNS + col + 1;
                format!("{:^width$}", format!("[{index}]"), width = CARD_INNER_WIDTH + 2)
            })
            .collect();
        println!("{}", labels.join(" "));
    }
}
