use serde::{Deserialize, Serialize};

use crate::battle::{
    component::{Component, ObjType},
    Phase, PosHex, PushStrength, Rounds, Strength,
};

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum Duration {
    Forever,
    Rounds(Rounds),
}

impl Duration {
    pub fn is_over(self) -> bool {
        match self {
            Duration::Rounds(n) => n.0 == 0,
            Duration::Forever => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Timed {
    pub duration: Duration,
    pub phase: Phase,
    pub effect: Lasting,
}

/// Instant effects
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::From)]
pub enum Effect {
    Create(Create),
    Kill(Kill),
    Vanish,
    Stun,
    Heal(Heal),
    Wound(Wound),
    Knockback(Knockback),
    FlyOff(FlyOff), // TODO: flying boulders should make some damage
    Throw(Throw),
    Dodge(Dodge),
    Bloodlust,
}

impl Effect {
    pub fn to_str(&self) -> &str {
        match *self {
            Effect::Create(_) => "Create",
            Effect::Kill(_) => "Kill",
            Effect::Vanish => "Vanish",
            Effect::Stun => "Stun",
            Effect::Heal(_) => "Heal",
            Effect::Wound(_) => "Wound",
            Effect::Knockback(_) => "Knockback",
            Effect::FlyOff(_) => "Fly off",
            Effect::Throw(_) => "Throw",
            Effect::Dodge(_) => "Dodge",
            Effect::Bloodlust => "Bloodlust",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Lasting {
    Poison,
    Stun,
    Bloodlust,
}

impl Lasting {
    pub fn title(&self) -> &str {
        match *self {
            Lasting::Poison => "Poison",
            Lasting::Stun => "Stun",
            Lasting::Bloodlust => "Bloodlust",
        }
    }

    pub fn description(&self) -> Vec<String> {
        match self {
            Lasting::Poison => vec![
                "Removes one strength every turn.".into(),
                "Doesn't kill: ends if only one strength is left.".into(),
            ],
            Lasting::Stun => vec!["Removes all Actions/Moves/Jokers every turn.".into()],
            Lasting::Bloodlust => vec!["Gives three additional Jokers every turn.".into()],
        }
    }
}

// TODO: Move `armor_break` to a separate effect?
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wound {
    pub damage: Strength,
    pub attacker_pos: Option<PosHex>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kill {
    pub attacker_pos: Option<PosHex>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Heal {
    pub strength: Strength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Create {
    pub pos: PosHex,
    pub prototype: ObjType,
    pub components: Vec<Component>,
    pub is_teleported: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FlyOff {
    pub from: PosHex,
    pub to: PosHex,
    pub strength: PushStrength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Throw {
    pub from: PosHex,
    pub to: PosHex,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dodge {
    pub attacker_pos: PosHex,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Knockback {
    pub from: PosHex,
    pub to: PosHex,
    pub strength: PushStrength,
}
