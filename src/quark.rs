use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum QuarkType {
    Up,
    Down,
    Charm,
    Strange,
    Top,
    Bottom,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hadron {
    pub id: String,
    pub composition: Vec<QuarkType>,
    pub kind: String,
}

pub fn forge_hadron(quarks: Vec<QuarkType>) -> Option<Hadron> {
    let kind = match quarks.as_slice() {
        [QuarkType::Up, QuarkType::Up, QuarkType::Down] => "Proton",
        [QuarkType::Up, QuarkType::Down, QuarkType::Down] => "Neutron",
        _ => return None,
    };
    let id = format!("{:?}", quarks);
    Some(Hadron {
        id,
        composition: quarks,
        kind: kind.to_string(),
    })
}

pub fn random_quark() -> QuarkType {
    let types = [
        QuarkType::Up,
        QuarkType::Down,
        QuarkType::Charm,
        QuarkType::Strange,
        QuarkType::Top,
        QuarkType::Bottom,
    ];
    types.choose(&mut rand::thread_rng()).unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proton_combination_valid() {
        let h = forge_hadron(vec![QuarkType::Up, QuarkType::Up, QuarkType::Down]);
        assert!(h.is_some());
        assert_eq!(h.unwrap().kind, "Proton");
    }

    #[test]
    fn neutron_combination_valid() {
        let h = forge_hadron(vec![QuarkType::Up, QuarkType::Down, QuarkType::Down]);
        assert!(h.is_some());
        assert_eq!(h.unwrap().kind, "Neutron");
    }

    #[test]
    fn invalid_combination_fails() {
        let h = forge_hadron(vec![QuarkType::Charm, QuarkType::Top, QuarkType::Bottom]);
        assert!(h.is_none());
    }
}
