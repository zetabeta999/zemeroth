use serde::{Deserialize, Serialize};

use crate::battle::{
    self,
    ability::{self, PassiveAbility, RechargeableAbility},
    component::{Abilities, Agent, Blocker, PassiveAbilities, Strength, Summoner, WeaponType},
    Attacks, Jokers, MovePoints, Moves, Weight,
};

use super::{component::Component, scenario::Line};

#[derive(Debug, Clone, Copy)]
pub enum Hero {
    Spearman = 0,
    EliteSpearman = 1,
    HeavySpearman = 2,
    Swordsman = 3,
    EliteSwordsman = 4,
    HeavySwordsman = 5,
    Hammerman = 6,
    HeavyHammerman = 7,
    Alchemist = 8,
    Healer = 9,
    Firer = 10,
}

impl Hero {
    pub fn from_index(i: u8) -> Self {
        match i {
            0 => Hero::Spearman,
            1 => Hero::EliteSpearman,
            2 => Hero::HeavySpearman,
            3 => Hero::Swordsman,
            4 => Hero::EliteSwordsman,
            5 => Hero::HeavySwordsman,
            6 => Hero::Hammerman,
            7 => Hero::HeavyHammerman,
            8 => Hero::Alchemist,
            9 => Hero::Healer,
            10 => Hero::Firer,
            _ => unimplemented!(),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "spearman" => Hero::Spearman,
            "elite_spearman" => Hero::EliteSpearman,
            "heavy_spearman" => Hero::HeavySpearman,
            "swordsman" => Hero::Swordsman,
            "elite_swordsman" => Hero::EliteSwordsman,
            "heavy_swordsman" => Hero::HeavySwordsman,
            "hammerman" => Hero::Hammerman,
            "heavy_hammerman" => Hero::HeavyHammerman,
            "alchemist" => Hero::Alchemist,
            "healer" => Hero::Healer,
            "firer" => Hero::Firer,
            _ => unimplemented!(),
        }
    }

    pub fn to_index(&self) -> u8 {
        *self as u8
    }

    pub fn name(&self) -> String {
        match self {
            Hero::Spearman => "spearman".to_owned(),
            Hero::EliteSpearman => "elite_spearman".to_owned(),
            Hero::HeavySpearman => "heavy_spearman".to_owned(),
            Hero::Swordsman => "swordsman".to_owned(),
            Hero::EliteSwordsman => "elite_swordsman".to_owned(),
            Hero::HeavySwordsman => "heavy_swordsman".to_owned(),
            Hero::Hammerman => "hammerman".to_owned(),
            Hero::HeavyHammerman => "heavy_hammerman".to_owned(),
            Hero::Alchemist => "alchemist".to_owned(),
            Hero::Healer => "healer".to_owned(),
            Hero::Firer => "firer".to_owned(),
        }
    }

    pub fn line(&self) -> Line {
        match self {
            Hero::Spearman | Hero::EliteSpearman | Hero::HeavySpearman => Line::Middle,
            Hero::Swordsman
            | Hero::EliteSwordsman
            | Hero::HeavySwordsman
            | Hero::Hammerman
            | Hero::HeavyHammerman => Line::Front,
            Hero::Alchemist | Hero::Firer | Hero::Healer => Line::Back,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeroObject {
    pub index: u8,
    pub count: u8,
}

impl HeroObject {
    pub fn new(hero: Hero, count: u8) -> Self {
        Self {
            index: hero.to_index(),
            count,
        }
    }

    pub fn new_index(index: u8, count: u8) -> Self {
        Self { index, count }
    }
}

pub fn prototype_for(name: &str) -> Vec<Component> {
    match name {
        "spearman" => spearman(),
        "elite_spearman" => elite_spearman(),
        "heavy_spearman" => heavy_spearman(),
        "swordsman" => swordsman(),
        "elite_swordsman" => elite_swordsman(),
        "heavy_swordsman" => heavy_swordsman(),
        "hammerman" => hammerman(),
        "heavy_hammerman" => heavy_hammerman(),
        "alchemist" => alchemist(),
        "healer" => healer(),
        "firer" => firer(),
        "imp" => imp(),
        "imp_bomber" => imp_bomber(),
        "toxic_imp" => toxic_imp(),
        "imp_summoner" => imp_summoner(),
        "bomb_poison" => bomb_poison(),
        "bomb_push" => bomb_push(),
        "bomb_demonic" => bomb_demonic(),
        "bomb_damage" => bomb_damage(),
        "bomb_fire" => bomb_fire(),
        "poison_cloud" => poison_cloud(),
        "fire" => fire(),
        "boulder" => boulder(),
        "spike_trap" => spike_trap(),

        _ => unreachable!(),
    }
}

pub fn spearman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(0),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 2,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Pierce,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(2),
            base_moves: Moves(0),
            base_attacks: Attacks(0),
            base_jokers: Jokers(1),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::LongJump,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn elite_spearman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(1),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 2,
            attack_accuracy: battle::Accuracy(5),
            weapon_type: WeaponType::Pierce,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(2),
            base_moves: Moves(0),
            base_attacks: Attacks(1),
            base_jokers: Jokers(1),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::LongJump,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn heavy_spearman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Heavy,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(5),
            strength: battle::Strength(5),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(1),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 2,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Pierce,
            move_points: MovePoints(1),
            reactive_attacks: Attacks(2),
            base_moves: Moves(0),
            base_attacks: Attacks(1),
            base_jokers: Jokers(1),
        }),
    ]
}

pub fn swordsman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(0),
            attack_strength: battle::Strength(1),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Slash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(1),
            base_moves: Moves(1),
            base_attacks: Attacks(1),
            base_jokers: Jokers(0),
        }),
    ]
}

pub fn elite_swordsman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(1),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Slash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(1),
            base_moves: Moves(0),
            base_attacks: Attacks(1),
            base_jokers: Jokers(1),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::Rage,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn heavy_swordsman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Heavy,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(5),
            strength: battle::Strength(5),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(1),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Slash,
            move_points: MovePoints(1),
            reactive_attacks: Attacks(1),
            base_moves: Moves(0),
            base_attacks: Attacks(1),
            base_jokers: Jokers(1),
        }),
    ]
}

pub fn hammerman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(0),
            attack_strength: battle::Strength(2),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(3),
            weapon_type: WeaponType::Smash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(1),
            base_moves: Moves(1),
            base_attacks: Attacks(2),
            base_jokers: Jokers(0),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::Club,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn heavy_hammerman() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Heavy,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(5),
            strength: battle::Strength(5),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(1),
            jokers: Jokers(1),
            attack_strength: battle::Strength(3),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(3),
            weapon_type: WeaponType::Smash,
            move_points: MovePoints(1),
            reactive_attacks: Attacks(0),
            base_moves: Moves(0),
            base_attacks: Attacks(1),
            base_jokers: Jokers(1),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::Club,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn alchemist() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(0),
            attack_strength: battle::Strength(1),
            attack_distance: 0,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Slash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(0),
            base_moves: Moves(1),
            base_attacks: Attacks(1),
            base_jokers: Jokers(0),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::Heal,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn healer() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(4),
            strength: battle::Strength(4),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(0),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 0,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Slash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(0),
            base_moves: Moves(1),
            base_attacks: Attacks(0),
            base_jokers: Jokers(1),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::GreatHeal,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn firer() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(4),
            strength: battle::Strength(4),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(1),
            jokers: Jokers(1),
            attack_strength: battle::Strength(1),
            attack_distance: 0,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Slash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(0),
            base_moves: Moves(0),
            base_attacks: Attacks(1),
            base_jokers: Jokers(1),
        }),
        Component::Abilities(Abilities(vec![
            RechargeableAbility {
                ability: ability::Ability::Bomb,
                status: ability::Status::Ready,
            },
            RechargeableAbility {
                ability: ability::Ability::BombFire,
                status: ability::Status::Ready,
            },
        ])),
    ]
}

pub fn imp() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(3),
            strength: battle::Strength(3),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(0),
            attack_strength: battle::Strength(1),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(3),
            weapon_type: WeaponType::Claw,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(1),
            base_moves: Moves(1),
            base_attacks: Attacks(1),
            base_jokers: Jokers(0),
        }),
    ]
}

pub fn imp_bomber() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(2),
            strength: battle::Strength(2),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(0),
            attack_strength: battle::Strength(1),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(3),
            weapon_type: WeaponType::Claw,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(0),
            base_moves: Moves(1),
            base_attacks: Attacks(1),
            base_jokers: Jokers(0),
        }),
        Component::Abilities(Abilities(vec![RechargeableAbility {
            ability: ability::Ability::BombDemonic,
            status: ability::Status::Ready,
        }])),
    ]
}

pub fn toxic_imp() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(2),
            strength: battle::Strength(2),
        }),
        Component::Agent(Agent {
            moves: Moves(1),
            attacks: Attacks(1),
            jokers: Jokers(0),
            attack_strength: battle::Strength(0),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(3),
            weapon_type: WeaponType::Claw,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(0),
            base_moves: Moves(1),
            base_attacks: Attacks(1),
            base_jokers: Jokers(0),
        }),
        Component::PassiveAbilities(PassiveAbilities(vec![PassiveAbility::PoisonAttack])),
    ]
}

pub fn imp_summoner() -> Vec<Component> {
    vec![
        Component::Blocker(Blocker {
            weight: Weight::Normal,
        }),
        Component::Strength(Strength {
            base_strength: battle::Strength(7),
            strength: battle::Strength(7),
        }),
        Component::Agent(Agent {
            moves: Moves(0),
            attacks: Attacks(0),
            jokers: Jokers(1),
            attack_strength: battle::Strength(2),
            attack_distance: 1,
            attack_accuracy: battle::Accuracy(4),
            weapon_type: WeaponType::Smash,
            move_points: MovePoints(2),
            reactive_attacks: Attacks(1),
            base_moves: Moves(0),
            base_attacks: Attacks(0),
            base_jokers: Jokers(1),
        }),
        Component::Summoner(Summoner { count: 2 }),
        Component::Abilities(Abilities(vec![
            RechargeableAbility {
                ability: ability::Ability::Summon,
                status: ability::Status::Ready,
            },
            RechargeableAbility {
                ability: ability::Ability::Bloodlust,
                status: ability::Status::Ready,
            },
        ])),
        Component::PassiveAbilities(PassiveAbilities(vec![PassiveAbility::Regenerate])),
    ]
}

pub fn bomb_demonic() -> Vec<Component> {
    vec![Component::Blocker(Blocker {
        weight: Weight::Normal,
    })]
}

pub fn bomb_poison() -> Vec<Component> {
    vec![Component::Blocker(Blocker {
        weight: Weight::Normal,
    })]
}

pub fn bomb_push() -> Vec<Component> {
    vec![Component::Blocker(Blocker {
        weight: Weight::Normal,
    })]
}

pub fn bomb_damage() -> Vec<Component> {
    vec![Component::Blocker(Blocker {
        weight: Weight::Normal,
    })]
}

pub fn bomb_fire() -> Vec<Component> {
    vec![Component::Blocker(Blocker {
        weight: Weight::Normal,
    })]
}

pub fn poison_cloud() -> Vec<Component> {
    vec![Component::PassiveAbilities(PassiveAbilities(vec![
        PassiveAbility::Poison,
    ]))]
}

pub fn fire() -> Vec<Component> {
    vec![Component::PassiveAbilities(PassiveAbilities(vec![
        PassiveAbility::Burn,
    ]))]
}

// map
pub fn boulder() -> Vec<Component> {
    vec![Component::Blocker(Blocker {
        weight: Weight::Heavy,
    })]
}

pub fn spike_trap() -> Vec<Component> {
    vec![Component::PassiveAbilities(PassiveAbilities(vec![
        PassiveAbility::SpikeTrap,
    ]))]
}
