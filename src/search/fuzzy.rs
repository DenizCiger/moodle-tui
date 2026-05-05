fn is_boundary(ch: char) -> bool {
    matches!(ch, ' ' | '-' | '_' | '/' | '.')
}

#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub score: f64,
    /// Char positions in the candidate (not byte indices) consumed by the
    /// query. Sorted, unique, suitable for `highlight_spans`.
    pub indices: Vec<usize>,
}

pub fn fuzzy_match(query_raw: &str, candidate_raw: &str) -> Option<FuzzyMatch> {
    let query: Vec<char> = query_raw.to_lowercase().chars().collect();
    let candidate: Vec<char> = candidate_raw.to_lowercase().chars().collect();
    if query.is_empty() {
        return Some(FuzzyMatch { score: 0.0, indices: Vec::new() });
    }
    if candidate.is_empty() {
        return None;
    }

    let mut query_idx = 0;
    let mut previous_match_idx: Option<usize> = None;
    let mut score = 0.0_f64;
    let mut indices: Vec<usize> = Vec::with_capacity(query.len());

    for (idx, ch) in candidate.iter().enumerate() {
        if query_idx >= query.len() {
            break;
        }
        if *ch != query[query_idx] {
            continue;
        }

        score += 1.0;
        if previous_match_idx == Some(idx.wrapping_sub(1)) {
            score += 6.0;
        }
        let prev_char = if idx > 0 { candidate[idx - 1] } else { ' ' };
        if idx == 0 || is_boundary(prev_char) {
            score += 4.0;
        }
        if idx < 6 {
            score += (6 - idx) as f64 * 0.25;
        }
        indices.push(idx);
        previous_match_idx = Some(idx);
        query_idx += 1;
    }

    if query_idx != query.len() {
        return None;
    }

    score -= candidate.len() as f64 * 0.01;
    Some(FuzzyMatch { score, indices })
}

pub fn fuzzy_score(query_raw: &str, candidate_raw: &str) -> Option<f64> {
    fuzzy_match(query_raw, candidate_raw).map(|m| m.score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_zero_score() {
        assert_eq!(fuzzy_score("", "anything"), Some(0.0));
    }

    #[test]
    fn no_match_returns_none() {
        assert_eq!(fuzzy_score("xyz", "abc"), None);
    }

    #[test]
    fn prefix_outranks_middle() {
        let prefix = fuzzy_score("ma", "Mathematics").unwrap();
        let middle = fuzzy_score("ma", "Drama Class").unwrap();
        assert!(prefix > middle);
    }

    #[test]
    fn match_returns_consumed_indices() {
        let m = fuzzy_match("mth", "Mathematics").unwrap();
        assert_eq!(m.indices, vec![0, 2, 3]);
    }

    #[test]
    fn match_indices_track_unicode_chars() {
        let m = fuzzy_match("ab", "aäb").unwrap();
        assert_eq!(m.indices, vec![0, 2]);
    }
}
