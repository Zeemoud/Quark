use crate::quark::{Hadron, QuarkType};
use sha2::{Digest, Sha256};

pub struct Validator {
    pub address: String,
    pub staked_hadrons: Vec<Hadron>,
    pub quarks_reward: Vec<QuarkType>,
    pub seed: u64,
}

impl Validator {
    pub fn weight(&self) -> u64 {
        let raw: u64 = self
            .staked_hadrons
            .iter()
            .map(|h| match h.kind.as_str() {
                "Proton" => 3,
                "Neutron" => 2,
                _ => 1,
            })
            .sum();
        raw.min(30) // plafond anti-concentration : un seul validateur ne peut peser plus que 30
    }
}

pub fn select_validator<'a>(
    validators: &'a [Validator],
    previous_hash: &str,
) -> Option<&'a Validator> {
    if validators.is_empty() {
        return None;
    }
    let total_weight: u64 = validators.iter().map(|v| v.weight() + 1).sum();

    let mut hasher = Sha256::new();
    hasher.update(previous_hash.as_bytes());
    for v in validators {
        hasher.update(v.seed.to_le_bytes());
    }
    let hash = hasher.finalize();
    let combined = u64::from_le_bytes(hash[0..8].try_into().unwrap());
    let mut pick = combined % total_weight;

    for v in validators {
        let w = v.weight() + 1;
        if pick < w {
            return Some(v);
        }
        pick -= w;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_zero_hadrons() {
        let v = Validator {
            address: "a".into(),
            staked_hadrons: vec![],
            quarks_reward: vec![],
            seed: 42,
        };
        assert_eq!(v.weight(), 0);
    }

    #[test]
    fn select_validator_empty_returns_none() {
        let validators: Vec<Validator> = vec![];
        assert!(select_validator(&validators, "abc").is_none());
    }

    #[test]
    fn select_validator_single_returns_it() {
        let validators = vec![Validator {
            address: "a".into(),
            staked_hadrons: vec![],
            quarks_reward: vec![],
            seed: 42,
        }];
        let selected = select_validator(&validators, "abc");
        assert_eq!(selected.unwrap().address, "a");
    }
}
