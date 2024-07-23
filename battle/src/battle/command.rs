use serde::{Deserialize, Serialize};

use crate::{
    battle::{ability::Ability, component::ObjType, movement::Path, Id, PlayerId},
    map::PosHex,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::From)]
pub enum Command {
    Create(Create),
    Attack(Attack),
    MoveTo(MoveTo),
    EndTurn(EndTurn),
    UseAbility(UseAbility),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Create {
    pub owner: Option<PlayerId>,
    pub pos: PosHex,
    pub prototype: ObjType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attack {
    pub attacker_id: Id,
    pub target_id: Id,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveTo {
    pub id: Id,
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndTurn;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseAbility {
    pub id: Id,
    pub pos: PosHex,
    pub ability: Ability,
}
