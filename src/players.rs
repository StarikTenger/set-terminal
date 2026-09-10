/// Validates a player name: must be non-empty and not start with a digit
/// (so it can never be confused with a card index in a claim).
pub fn validate_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Player name cannot be empty.".to_string());
    }
    if name.chars().next().unwrap().is_ascii_digit() {
        return Err(format!("Player name \"{name}\" cannot start with a number."));
    }
    Ok(())
}

/// A registry of players in team mode, tracking each player's score
/// (number of Sets found). Names are matched case-insensitively and by
/// unambiguous prefix.
pub struct Players {
    names: Vec<String>,
    scores: Vec<u32>,
}

impl Players {
    pub fn new() -> Self {
        Players {
            names: Vec::new(),
            scores: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Registers a new player. Rejects invalid names and names that
    /// already exist (case-insensitively).
    pub fn register(&mut self, name: &str) -> Result<(), String> {
        validate_name(name)?;
        let name = name.trim();
        if self
            .names
            .iter()
            .any(|n| n.eq_ignore_ascii_case(name))
        {
            return Err(format!("Player \"{name}\" is already registered."));
        }
        self.names.push(name.to_string());
        self.scores.push(0);
        Ok(())
    }

    /// Resolves a name or unambiguous case-insensitive prefix to a
    /// registered player's index. An exact case-insensitive match always
    /// wins over a prefix match, so a name that is itself a prefix of
    /// another registered name still resolves unambiguously.
    pub fn find_by_prefix(&self, query: &str) -> Result<usize, String> {
        let query = query.trim();
        if query.is_empty() {
            return Err("Please enter a player name.".to_string());
        }

        if let Some(idx) = self.names.iter().position(|n| n.eq_ignore_ascii_case(query)) {
            return Ok(idx);
        }

        let query_lower = query.to_lowercase();
        let matches: Vec<usize> = self
            .names
            .iter()
            .enumerate()
            .filter(|(_, n)| n.to_lowercase().starts_with(&query_lower))
            .map(|(i, _)| i)
            .collect();

        match matches.len() {
            0 => Err(format!("No registered player matches \"{query}\".")),
            1 => Ok(matches[0]),
            _ => {
                let names: Vec<&str> = matches.iter().map(|&i| self.names[i].as_str()).collect();
                Err(format!(
                    "\"{query}\" matches multiple players ({}); be more specific.",
                    names.join(", ")
                ))
            }
        }
    }

    pub fn add_score(&mut self, idx: usize) {
        self.scores[idx] += 1;
    }

    pub fn name(&self, idx: usize) -> &str {
        &self.names[idx]
    }

    pub fn score(&self, idx: usize) -> u32 {
        self.scores[idx]
    }

    /// Returns (name, score) pairs sorted by score descending, ties broken
    /// alphabetically, for display as an end-of-game scoreboard.
    pub fn scoreboard(&self) -> Vec<(&str, u32)> {
        let mut rows: Vec<(&str, u32)> = (0..self.names.len())
            .map(|i| (self.names[i].as_str(), self.score(i)))
            .collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_name_starting_with_digit() {
        assert!(validate_name("1Alice").is_err());
    }

    #[test]
    fn rejects_empty_name() {
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn accepts_normal_name() {
        assert!(validate_name("Alice").is_ok());
    }

    #[test]
    fn rejects_duplicate_registration_case_insensitively() {
        let mut players = Players::new();
        players.register("Alice").unwrap();
        assert!(players.register("alice").is_err());
    }

    #[test]
    fn finds_player_by_case_insensitive_prefix() {
        let mut players = Players::new();
        players.register("Alice").unwrap();
        players.register("Bob").unwrap();
        assert_eq!(players.find_by_prefix("al").unwrap(), 0);
        assert_eq!(players.find_by_prefix("AL").unwrap(), 0);
        assert_eq!(players.find_by_prefix("bob").unwrap(), 1);
    }

    #[test]
    fn exact_match_wins_over_ambiguous_prefix() {
        let mut players = Players::new();
        players.register("Al").unwrap();
        players.register("Albert").unwrap();
        assert_eq!(players.find_by_prefix("al").unwrap(), 0);
    }

    #[test]
    fn ambiguous_prefix_is_rejected() {
        let mut players = Players::new();
        players.register("Alice").unwrap();
        players.register("Alistair").unwrap();
        assert!(players.find_by_prefix("ali").is_err());
    }

    #[test]
    fn unknown_prefix_is_rejected() {
        let mut players = Players::new();
        players.register("Alice").unwrap();
        assert!(players.find_by_prefix("zzz").is_err());
    }

    #[test]
    fn scoreboard_sorts_by_score_descending_then_name() {
        let mut players = Players::new();
        players.register("Alice").unwrap();
        players.register("Bob").unwrap();
        players.register("Cara").unwrap();
        players.add_score(1); // Bob: 1
        players.add_score(2); // Cara: 1
        players.add_score(2); // Cara: 2
        assert_eq!(
            players.scoreboard(),
            vec![("Cara", 2), ("Bob", 1), ("Alice", 0)]
        );
    }
}
