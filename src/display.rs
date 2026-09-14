use colored::{Color as TermColor, Colorize};

use crate::card::{Card, Color, Shading, Shape};

const COLUMNS: usize = 4;
const CARD_INNER_WIDTH: usize = 7;

/// Maps the game's abstract card colors to concrete terminal colors, so
/// players can adjust the palette for their terminal theme or color vision.
pub struct Palette {
    pub red: TermColor,
    pub green: TermColor,
    pub purple: TermColor,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            red: TermColor::Red,
            green: TermColor::Green,
            purple: TermColor::Magenta,
        }
    }
}

/// Parses a hex color code ("#ff8800" or "ff8800") into an RGB triple.
fn parse_hex_code(s: &str) -> Option<(u8, u8, u8)> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Parses a terminal color name (e.g. "red", "bright_blue") or a hex RGB
/// code (e.g. "#ff8800") for use in a custom palette. Returns None for
/// unrecognized values.
pub fn parse_color_name(s: &str) -> Option<TermColor> {
    let s = s.trim();
    if let Some((r, g, b)) = parse_hex_code(s) {
        return Some(TermColor::TrueColor { r, g, b });
    }
    let color = match s.to_lowercase().as_str() {
        "black" => TermColor::Black,
        "red" => TermColor::Red,
        "green" => TermColor::Green,
        "yellow" => TermColor::Yellow,
        "blue" => TermColor::Blue,
        "magenta" | "purple" => TermColor::Magenta,
        "cyan" => TermColor::Cyan,
        "white" => TermColor::White,
        "bright_black" | "gray" | "grey" => TermColor::BrightBlack,
        "bright_red" => TermColor::BrightRed,
        "bright_green" => TermColor::BrightGreen,
        "bright_yellow" => TermColor::BrightYellow,
        "bright_blue" => TermColor::BrightBlue,
        "bright_magenta" => TermColor::BrightMagenta,
        "bright_cyan" => TermColor::BrightCyan,
        "bright_white" => TermColor::BrightWhite,
        _ => return None,
    };
    Some(color)
}

/// Parses a `--colors <red>,<green>,<purple>` value into a Palette.
/// Returns an error message naming the offending token on failure.
pub fn parse_palette(spec: &str) -> Result<Palette, String> {
    let parts: Vec<&str> = spec.split(',').collect();
    if parts.len() != 3 {
        return Err(format!(
            "--colors needs 3 comma-separated colors (red,green,purple), got \"{spec}\""
        ));
    }
    let mut colors = [TermColor::Red; 3];
    for (slot, part) in colors.iter_mut().zip(parts.iter()) {
        *slot = parse_color_name(part)
            .ok_or_else(|| format!("unknown color \"{part}\" (use a name like \"red\" or a hex code like \"#ff8800\")"))?;
    }
    Ok(Palette {
        red: colors[0],
        green: colors[1],
        purple: colors[2],
    })
}

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

fn term_color(color: Color, palette: &Palette) -> TermColor {
    match color {
        Color::Red => palette.red,
        Color::Green => palette.green,
        Color::Purple => palette.purple,
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

pub fn render_board(cards: &[Card], palette: &Palette, new_indices: &[usize], highlight: bool) {
    for (row_idx, chunk) in cards.chunks(COLUMNS).enumerate() {
        let border_top: String = "┌".to_string() + &"─".repeat(CARD_INNER_WIDTH) + "┐";
        let border_bottom: String = "└".to_string() + &"─".repeat(CARD_INNER_WIDTH) + "┘";

        let tops: Vec<String> = chunk.iter().map(|_| border_top.clone()).collect();
        println!("{}", tops.join(" "));

        let mids: Vec<String> = chunk
            .iter()
            .map(|c| {
                let content = card_content(c).color(term_color(c.color, palette)).to_string();
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
                let label = format!("{:^width$}", format!("[{index}]"), width = CARD_INNER_WIDTH + 2);
                if highlight && new_indices.contains(&(index - 1)) {
                    label.bold().yellow().to_string()
                } else {
                    label
                }
            })
            .collect();
        println!("{}", labels.join(" "));
    }
}
