//! The pairing-list format of the bbpPairings and JaVaFo command lines,
//! which `co pair -p` also produces so the conformance harness can compare
//! engines line for line.
//!
//! The first line is the number of pairs; each following line is
//! `white black` by TRF id, with the pairing-allocated bye as `id 0`.

use co_core::{ByeKind, Pairing, PlayerId};
use std::collections::HashMap;

pub fn format_pairs(pairs: &[(u32, u32)]) -> String {
    let mut s = format!("{}\n", pairs.len());
    for (w, b) in pairs {
        s.push_str(&format!("{w} {b}\n"));
    }
    s
}

pub fn parse_pairs(text: &str) -> Result<Vec<(u32, u32)>, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let count: usize = lines
        .next()
        .ok_or("empty pairing output")?
        .parse()
        .map_err(|_| "first line must be the number of pairs")?;
    let pairs = lines
        .map(|l| {
            let mut it = l.split_whitespace().map(str::parse::<u32>);
            match (it.next(), it.next()) {
                (Some(Ok(w)), Some(Ok(b))) => Ok((w, b)),
                _ => Err(format!("invalid pair line '{l}'")),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if pairs.len() != count {
        return Err(format!("expected {count} pairs, found {}", pairs.len()));
    }
    Ok(pairs)
}

/// Games and the pairing-allocated bye of a round, as TRF-id pairs.
/// Requested byes and absences are not listed, as in bbpPairings.
pub fn round_to_pairs(pairings: &[Pairing], ids: &HashMap<PlayerId, u32>) -> Vec<(u32, u32)> {
    pairings
        .iter()
        .filter_map(|p| match *p {
            Pairing::Game { white, black, .. } => Some((ids[&white], ids[&black])),
            Pairing::Bye {
                player,
                kind: ByeKind::PairingAllocated,
            } => Some((ids[&player], 0)),
            Pairing::Bye { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let pairs = vec![(1, 4), (5, 2), (7, 0)];
        let text = format_pairs(&pairs);
        assert_eq!(text, "3\n1 4\n5 2\n7 0\n");
        assert_eq!(parse_pairs(&text).unwrap(), pairs);
        assert!(parse_pairs("2\n1 2\n").is_err());
    }
}
