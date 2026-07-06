use std::collections::HashSet;

/// Maximum cumulative serialized size of relevance-selected context items (chars).
/// Leaves headroom for the system prompt + the LLM response under typical context windows.
pub const INPUT_BUDGET_CHARS: usize = 8000;

/// Tokenize a string into lowercased identifier tokens.
///
/// Splits on whitespace, then on `_`, then on CamelCase transitions.
/// "OrderItem track" -> ["order", "item", "track"].
pub fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for word in s.split_whitespace() {
        let snake_parts: Vec<&str> = word.split('_').collect();
        for part in snake_parts {
            // Split CamelCase: "OrderItem" -> ["Order", "Item"] -> ["order", "item"]
            let mut cur = String::new();
            for ch in part.chars() {
                if ch.is_uppercase() && !cur.is_empty() {
                    let lower = cur.to_lowercase();
                    if !lower.is_empty() {
                        tokens.push(lower);
                    }
                    cur = String::new();
                }
                cur.push(ch);
            }
            if !cur.is_empty() {
                let lower = cur.to_lowercase();
                if !lower.is_empty() {
                    tokens.push(lower);
                }
            }
        }
    }
    tokens
}

/// A context candidate with its searchable tokens and serialized form.
pub struct Candidate {
    /// Human-readable label for debugging (e.g. "model:Order").
    #[allow(dead_code)]
    pub label: String,
    /// Lowercased identifier tokens extracted from the candidate's content.
    pub tokens: HashSet<String>,
    /// The text chunk that goes into the prompt if this candidate is selected.
    pub serialized: String,
    /// Tie-break priority: projections=3 > models=2 > routes=1 > schema=0.
    pub tier: u8,
}

/// Select context chunks relevant to the description within the INPUT_BUDGET_CHARS budget.
///
/// Score = |description_tokens ∩ candidate.tokens| (set intersection cardinality).
/// Sort descending by (score, tier). Keep candidates while cumulative serialized.len()
/// <= INPUT_BUDGET_CHARS. Zero-score candidates are included only if budget permits.
pub fn select_relevant(description: &str, mut candidates: Vec<Candidate>) -> Vec<String> {
    let desc_tokens: HashSet<String> = tokenize(description).into_iter().collect();

    // Score each candidate
    let mut scored: Vec<(usize, &Candidate)> = candidates
        .iter()
        .map(|c| {
            let score = desc_tokens.intersection(&c.tokens).count();
            (score, c)
        })
        .collect();

    // Sort descending: first by score, then by tier
    scored.sort_by(|(score_a, a), (score_b, b)| {
        score_b.cmp(score_a).then_with(|| b.tier.cmp(&a.tier))
    });

    let mut result = Vec::new();
    let mut cumulative = 0usize;

    for (score, candidate) in &scored {
        let chunk_len = candidate.serialized.len();
        if cumulative + chunk_len <= INPUT_BUDGET_CHARS {
            result.push(candidate.serialized.clone());
            cumulative += chunk_len;
        } else if *score == 0 {
            // Zero-score items beyond budget are dropped entirely
            break;
        }
        // Non-zero score items that exceed budget are skipped (could be large schema chunks)
    }

    // Clear candidates to drop borrowed data
    drop(scored);
    candidates.clear();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relevance_tokenize_splits_camel_and_snake() {
        let tokens = tokenize("OrderItem track customer_orders");
        assert!(tokens.contains(&"order".to_string()), "tokens: {tokens:?}");
        assert!(tokens.contains(&"item".to_string()), "tokens: {tokens:?}");
        assert!(tokens.contains(&"track".to_string()), "tokens: {tokens:?}");
        assert!(
            tokens.contains(&"customer".to_string()),
            "tokens: {tokens:?}"
        );
        assert!(tokens.contains(&"orders".to_string()), "tokens: {tokens:?}");
    }

    #[test]
    fn relevance_scores_intersection() {
        let candidates = vec![
            Candidate {
                label: "model:Order".to_string(),
                tokens: ["order".to_string()].into_iter().collect(),
                serialized: "Order model".to_string(),
                tier: 2,
            },
            Candidate {
                label: "model:Invoice".to_string(),
                tokens: ["invoice".to_string()].into_iter().collect(),
                serialized: "Invoice model".to_string(),
                tier: 2,
            },
        ];

        let selected = select_relevant("order customer", candidates);

        // Order should be selected (score 1), Invoice may or may not be (score 0)
        // But Order must appear first if both selected
        assert!(!selected.is_empty());
        assert_eq!(selected[0], "Order model");
    }

    #[test]
    fn relevance_budget_drops_low_score() {
        // Create a budget scenario where only a small number of chars fit
        // Use INPUT_BUDGET_CHARS-level test by creating items that together exceed budget
        let high_score_serialized = "a".repeat(100); // small high-score item
        let low_score_serialized = "b".repeat(INPUT_BUDGET_CHARS); // fills entire budget

        let candidates = vec![
            Candidate {
                label: "high:Order".to_string(),
                tokens: ["order".to_string()].into_iter().collect(),
                serialized: high_score_serialized.clone(),
                tier: 2,
            },
            Candidate {
                label: "low:Invoice".to_string(),
                tokens: ["invoice".to_string()].into_iter().collect(),
                serialized: low_score_serialized.clone(),
                tier: 2,
            },
        ];

        // description matches "order" only
        let selected = select_relevant("order customer", candidates);

        // high-score item fits; low-score item (zero overlap with "order customer") is after
        // budget is consumed by high-score item + remaining budget is 8000-100=7900
        // low-score item is 8000 chars so it won't fit
        assert!(selected.contains(&high_score_serialized));
        assert!(!selected.contains(&low_score_serialized));
    }
}
