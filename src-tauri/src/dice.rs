//! Dice notation parser and evaluator.
//!
//! Supported forms (case-insensitive, whitespace tolerated):
//!   d20        2d6+4      2d20kh        4d6kl3     1d8-1      12
//! `kh`/`kl` keep the highest/lowest N dice (default 1) — advantage/disadvantage.

use rand::Rng;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct DiceGroup {
    pub count: i64,
    pub sides: i64,
    pub keep: Option<(Keep, i64)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keep {
    High,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub groups: Vec<(DiceGroup, i64)>, // (group, sign)
    pub modifier: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeptGroup {
    pub rolls: Vec<i64>,
    pub kept: Option<Vec<i64>>,
    pub sides: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollResult {
    pub notation: String,
    pub total: i64,
    pub modifier: i64,
    pub groups: Vec<KeptGroup>,
}

pub fn parse(expr: &str) -> Result<Expr, String> {
    let s: String = expr.chars().filter(|c| !c.is_whitespace()).collect();
    let s = s.to_lowercase();
    if s.is_empty() {
        return Err("Empty dice expression".to_string());
    }
    let bytes = s.as_bytes();
    let mut i = 0usize;
    let mut groups: Vec<(DiceGroup, i64)> = Vec::new();
    let mut modifier: i64 = 0;
    let mut saw_term = false;

    while i < bytes.len() {
        let mut sign = 1i64;
        if bytes[i] == b'+' || bytes[i] == b'-' {
            if bytes[i] == b'-' {
                sign = -1;
            }
            i += 1;
            if !saw_term {
                return Err(format!("Leading sign in “{expr}”"));
            }
        }
        if i >= bytes.len() {
            return Err(format!("Trailing operator in “{expr}”"));
        }

        // Optional count.
        let count_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        let has_count = i > count_start;
        let count: i64 = if has_count {
            s[count_start..i].parse().map_err(|_| "Number too large")?
        } else {
            1
        };

        if i < bytes.len() && bytes[i] == b'd' {
            i += 1;
            let sides_start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i == sides_start {
                return Err(format!("Missing die size in “{expr}”"));
            }
            let sides: i64 = s[sides_start..i]
                .parse()
                .map_err(|_| "Number too large".to_string())?;

            let mut keep = None;
            if s[i..].starts_with("kh") || s[i..].starts_with("kl") {
                let kind = if bytes[i] == b'k' && bytes[i + 1] == b'h' {
                    Keep::High
                } else {
                    Keep::Low
                };
                i += 2;
                let n_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let n: i64 = if i > n_start {
                    s[n_start..i].parse().map_err(|_| "Number too large")?
                } else {
                    1
                };
                keep = Some((kind, n));
            }

            if count < 1 || count > 100 {
                return Err(format!("Die count must be 1–100 in “{expr}”"));
            }
            if sides < 1 || sides > 10_000 {
                return Err(format!("Die size must be 1–10000 in “{expr}”"));
            }
            if let Some((_, n)) = keep {
                if n < 1 || n > count {
                    return Err(format!("Keep count must be 1–{count} in “{expr}”"));
                }
            }
            groups.push((DiceGroup { count, sides, keep }, sign));
            saw_term = true;
        } else if has_count {
            if count.abs() > 1_000_000 {
                return Err("Constant too large".to_string());
            }
            modifier += sign * count;
            saw_term = true;
        } else {
            return Err(format!("Unexpected “{}” in “{expr}”", &s[i..i.min(s.len())]));
        }
    }

    if !saw_term && modifier == 0 {
        return Err(format!("No dice in “{expr}”"));
    }
    Ok(Expr { groups, modifier })
}

fn eval_with(expr: &Expr, roll: &mut dyn FnMut(i64) -> i64) -> RollResult {
    let mut total: i64 = 0;
    let mut out_groups = Vec::new();
    for (g, sign) in &expr.groups {
        let mut rolls = Vec::with_capacity(g.count as usize);
        for _ in 0..g.count {
            rolls.push(roll(g.sides));
        }
        let kept = match g.keep {
            None => None,
            Some((Keep::High, n)) => {
                let mut sorted = rolls.clone();
                sorted.sort_unstable_by(|a, b| b.cmp(a));
                Some(sorted[..n as usize].to_vec())
            }
            Some((Keep::Low, n)) => {
                let mut sorted = rolls.clone();
                sorted.sort_unstable();
                Some(sorted[..n as usize].to_vec())
            }
        };
        let sum: i64 = kept.as_ref().unwrap_or(&rolls).iter().sum::<i64>();
        total += sign * sum;
        out_groups.push(KeptGroup {
            rolls: rolls.clone(),
            kept,
            sides: g.sides,
        });
    }
    total += expr.modifier;
    RollResult {
        notation: String::new(),
        total,
        modifier: expr.modifier,
        groups: out_groups,
    }
}

pub fn roll(expr: &str) -> Result<RollResult, String> {
    let parsed = parse(expr)?;
    let mut result = eval_with(&parsed, &mut |sides| rand::rng().random_range(1..=sides));
    result.notation = normalize_notation(&parsed);
    Ok(result)
}

/// Canonicalized notation with explicit die count: `2D6+4` → `2d6+4`, `d20` → `1d20`.
pub fn normalize_notation(expr: &Expr) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (g, sign) in &expr.groups {
        let mut s = match *sign {
            s if s < 0 => "-".to_string(),
            _ if parts.is_empty() => String::new(),
            _ => "+".to_string(),
        };
        s.push_str(&g.count.to_string());
        s.push('d');
        s.push_str(&g.sides.to_string());
        if let Some((kind, n)) = g.keep {
            s.push_str(match kind {
                Keep::High => "kh",
                Keep::Low => "kl",
            });
            if n != 1 {
                s.push_str(&n.to_string());
            }
        }
        parts.push(s);
    }
    if expr.modifier != 0 || parts.is_empty() {
        let m = expr.modifier;
        if m != 0 || parts.is_empty() {
            parts.push(if m >= 0 && !parts.is_empty() {
                format!("+{m}")
            } else {
                m.to_string()
            });
        }
    }
    parts.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic fake die: cycles a fixed sequence.
    fn seq(rolls: &[i64]) -> impl FnMut(i64) -> i64 + '_ {
        let idx = std::cell::Cell::new(0usize);
        move |_sides| {
            let v = rolls[idx.get() % rolls.len()];
            idx.set(idx.get() + 1);
            v
        }
    }

    #[test]
    fn parses_basics() {
        let e = parse("2d6+4").unwrap();
        assert_eq!(e.groups.len(), 1);
        assert_eq!(e.groups[0].0.count, 2);
        assert_eq!(e.groups[0].0.sides, 6);
        assert_eq!(e.modifier, 4);

        let e = parse("d20").unwrap();
        assert_eq!(e.groups[0].0.count, 1);
        assert_eq!(e.groups[0].0.sides, 20);

        let e = parse("2 D 8 - 1").unwrap();
        assert_eq!(e.groups[0].0.sides, 8);
        assert_eq!(e.modifier, -1);
    }

    #[test]
    fn parses_keep_notation() {
        let e = parse("2d20kh").unwrap();
        assert_eq!(e.groups[0].0.keep, Some((Keep::High, 1)));
        let e = parse("4D6KL3").unwrap();
        assert_eq!(e.groups[0].0.keep, Some((Keep::Low, 3)));
        assert_eq!(normalize_notation(&e), "4d6kl3");
    }

    #[test]
    fn evaluates_sums_and_keeps() {
        let e = parse("3d1+2").unwrap(); // d1 always rolls 1
        let r = eval_with(&e, &mut |s| {
            assert_eq!(s, 1);
            1
        });
        assert_eq!(r.total, 5);
        assert_eq!(r.modifier, 2);

        let e = parse("2d20kh+0").unwrap();
        let r = eval_with(&e, &mut seq(&[3, 18]));
        assert_eq!(r.total, 18);
        assert_eq!(r.groups[0].kept, Some(vec![18]));

        let e = parse("4d6kl3").unwrap();
        let r = eval_with(&e, &mut seq(&[6, 6, 1, 2]));
        assert_eq!(r.total, 9); // 2 + 1 + 6? no: sorted [1,2,6,6] lowest 3 = 9
    }

    #[test]
    fn evaluates_negative_groups() {
        let e = parse("2d1-1d1").unwrap();
        let r = eval_with(&e, &mut seq(&[1, 1, 1]));
        assert_eq!(r.total, 1);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("").is_err());
        assert!(parse("d").is_err());
        assert!(parse("2d").is_err());
        assert!(parse("2d6+").is_err());
        assert!(parse("+2d6").is_err());
        assert!(parse("0d6").is_err());
        assert!(parse("3d6kh9").is_err());
        assert!(parse("2d6x").is_err());
    }

    #[test]
    fn real_rolls_are_in_range() {
        for _ in 0..200 {
            let r = roll("1d20+5").unwrap();
            assert!((6..=25).contains(&r.total));
        }
        for _ in 0..100 {
            let r = roll("2d20kh+3").unwrap();
            assert!((4..=23).contains(&r.total));
        }
    }

    #[test]
    fn normalizes_notation() {
        assert_eq!(normalize_notation(&parse("2D6+4").unwrap()), "2d6+4");
        assert_eq!(normalize_notation(&parse("d20").unwrap()), "1d20");
        assert_eq!(normalize_notation(&parse("1d6+1").unwrap()), "1d6+1");
        assert_eq!(normalize_notation(&parse("2d20kh").unwrap()), "2d20kh");
        assert_eq!(normalize_notation(&parse("12").unwrap()), "12");
    }
}
