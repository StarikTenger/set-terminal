#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Number {
    One,
    Two,
    Three,
}

impl Number {
    pub fn count(self) -> usize {
        match self {
            Number::One => 1,
            Number::Two => 2,
            Number::Three => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    Red,
    Green,
    Purple,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Diamond,
    Squiggle,
    Oval,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shading {
    Solid,
    Striped,
    Empty,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Card {
    pub number: Number,
    pub color: Color,
    pub shape: Shape,
    pub shading: Shading,
}

const NUMBERS: [Number; 3] = [Number::One, Number::Two, Number::Three];
const COLORS: [Color; 3] = [Color::Red, Color::Green, Color::Purple];
const SHAPES: [Shape; 3] = [Shape::Diamond, Shape::Squiggle, Shape::Oval];
const SHADINGS: [Shading; 3] = [Shading::Solid, Shading::Striped, Shading::Empty];

pub fn full_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(81);
    for &number in &NUMBERS {
        for &color in &COLORS {
            for &shape in &SHAPES {
                for &shading in &SHADINGS {
                    deck.push(Card {
                        number,
                        color,
                        shape,
                        shading,
                    });
                }
            }
        }
    }
    deck
}

fn all_same_or_all_different<T: PartialEq>(a: T, b: T, c: T) -> bool {
    (a == b && b == c) || (a != b && b != c && a != c)
}

pub fn is_set(a: &Card, b: &Card, c: &Card) -> bool {
    all_same_or_all_different(a.number, b.number, c.number)
        && all_same_or_all_different(a.color, b.color, c.color)
        && all_same_or_all_different(a.shape, b.shape, c.shape)
        && all_same_or_all_different(a.shading, b.shading, c.shading)
}

/// Board slots may be empty (`None`) in game modes that leave gaps instead of
/// refilling immediately, so Set-finding skips over them.
pub fn find_any_set(cards: &[Option<Card>]) -> Option<(usize, usize, usize)> {
    for i in 0..cards.len() {
        let Some(a) = cards[i] else { continue };
        for j in (i + 1)..cards.len() {
            let Some(b) = cards[j] else { continue };
            for k in (j + 1)..cards.len() {
                let Some(c) = cards[k] else { continue };
                if is_set(&a, &b, &c) {
                    return Some((i, j, k));
                }
            }
        }
    }
    None
}

pub fn count_sets(cards: &[Option<Card>]) -> usize {
    find_all_sets(cards).len()
}

/// Returns the board indices (0-based) of every valid Set currently on the
/// board, e.g. for `/cheat`.
pub fn find_all_sets(cards: &[Option<Card>]) -> Vec<(usize, usize, usize)> {
    let mut sets = Vec::new();
    for i in 0..cards.len() {
        let Some(a) = cards[i] else { continue };
        for j in (i + 1)..cards.len() {
            let Some(b) = cards[j] else { continue };
            for k in (j + 1)..cards.len() {
                let Some(c) = cards[k] else { continue };
                if is_set(&a, &b, &c) {
                    sets.push((i, j, k));
                }
            }
        }
    }
    sets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_has_81_unique_cards() {
        let deck = full_deck();
        assert_eq!(deck.len(), 81);
    }

    #[test]
    fn detects_valid_set_all_same() {
        let c = Card {
            number: Number::One,
            color: Color::Red,
            shape: Shape::Diamond,
            shading: Shading::Solid,
        };
        assert!(is_set(&c, &c, &c));
    }

    #[test]
    fn detects_valid_set_all_different() {
        let a = Card {
            number: Number::One,
            color: Color::Red,
            shape: Shape::Diamond,
            shading: Shading::Solid,
        };
        let b = Card {
            number: Number::Two,
            color: Color::Green,
            shape: Shape::Squiggle,
            shading: Shading::Striped,
        };
        let c = Card {
            number: Number::Three,
            color: Color::Purple,
            shape: Shape::Oval,
            shading: Shading::Empty,
        };
        assert!(is_set(&a, &b, &c));
    }

    #[test]
    fn rejects_invalid_set() {
        let a = Card {
            number: Number::One,
            color: Color::Red,
            shape: Shape::Diamond,
            shading: Shading::Solid,
        };
        let b = Card {
            number: Number::Two,
            color: Color::Red,
            shape: Shape::Squiggle,
            shading: Shading::Striped,
        };
        let c = Card {
            number: Number::Three,
            color: Color::Purple,
            shape: Shape::Oval,
            shading: Shading::Empty,
        };
        // color is Red, Red, Purple -> two same, one different -> invalid
        assert!(!is_set(&a, &b, &c));
    }

    #[test]
    fn every_pair_has_exactly_one_completion() {
        let deck = full_deck();
        let a = deck[3];
        let b = deck[40];
        let completions: Vec<_> = deck.iter().filter(|c| is_set(&a, &b, c)).collect();
        assert_eq!(completions.len(), 1);
    }
}
