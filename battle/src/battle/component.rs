use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    battle::{
        self,
        ability::{Ability, PassiveAbility, RechargeableAbility},
        effect::Timed,
        Attacks, Id, Jokers, MovePoints, Moves, Phase, PlayerId, Rounds,
    },
    map,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pos(pub map::PosHex);

/// Blocks the whole tile. Two blocker objects can't coexist in one tile.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Blocker {
    #[serde(default)]
    pub weight: battle::Weight,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Strength {
    #[serde(default)]
    pub base_strength: battle::Strength,

    pub strength: battle::Strength,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ObjType(pub String);

impl From<&str> for ObjType {
    fn from(s: &str) -> Self {
        ObjType(s.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub name: ObjType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BelongsTo(pub PlayerId);

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Eq, Hash)]
pub enum WeaponType {
    Slash,
    Smash,
    Pierce,
    Claw,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    // dynamic
    pub moves: Moves,
    pub attacks: Attacks,
    pub jokers: Jokers,

    // static
    pub attack_strength: battle::Strength,
    pub attack_distance: i32,
    pub attack_accuracy: battle::Accuracy,
    pub weapon_type: WeaponType,

    pub move_points: MovePoints,
    pub reactive_attacks: Attacks,

    #[serde(default)]
    pub base_moves: Moves,

    #[serde(default)]
    pub base_attacks: Attacks,

    #[serde(default)]
    pub base_jokers: Jokers,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Abilities(pub Vec<RechargeableAbility>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PassiveAbilities(pub Vec<PassiveAbility>);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Effects(pub Vec<Timed>);

// TODO: Move to `ability` mod?
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlannedAbility {
    pub rounds: Rounds,
    pub phase: Phase,
    pub ability: Ability,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Schedule {
    pub planned: Vec<PlannedAbility>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summoner {
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, derive_more::From)]
pub enum Component {
    Pos(Pos),
    Strength(Strength),
    Meta(Meta),
    BelongsTo(BelongsTo),
    Agent(Agent),
    Blocker(Blocker),
    Abilities(Abilities),
    PassiveAbilities(PassiveAbilities),
    Effects(Effects),
    Schedule(Schedule),
    Summoner(Summoner),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parts {
    pub strength: HashMap<Id, Strength>,
    pub pos: HashMap<Id, Pos>,
    pub meta: HashMap<Id, Meta>,
    pub belongs_to: HashMap<Id, BelongsTo>,
    pub agent: HashMap<Id, Agent>,
    pub blocker: HashMap<Id, Blocker>,
    pub abilities: HashMap<Id, Abilities>,
    pub passive_abilities: HashMap<Id, PassiveAbilities>,
    pub effects: HashMap<Id, Effects>,
    pub schedule: HashMap<Id, Schedule>,
    pub summoner: HashMap<Id, Summoner>,
    pub next_obj_id: Id,
}

impl Parts {
    pub fn new() -> Self {
        Self {
            strength: HashMap::new(),
            pos: HashMap::new(),
            meta: HashMap::new(),
            belongs_to: HashMap::new(),
            agent: HashMap::new(),
            blocker: HashMap::new(),
            abilities: HashMap::new(),
            passive_abilities: HashMap::new(),
            effects: HashMap::new(),
            schedule: HashMap::new(),
            summoner: HashMap::new(),
            next_obj_id: Default::default(),
        }
    }

    pub fn alloc_id(&mut self) -> Id {
        let id = self.next_obj_id;
        self.next_obj_id.0 += 1;
        id
    }

    pub fn is_exist(&self, id: &Id) -> bool {
        self.blocker.get(id).is_some()
    }

    pub fn remove(&mut self, id: &Id) {
        self.strength.remove(id);
        self.pos.remove(id);
        self.meta.remove(id);
        self.belongs_to.remove(id);
        self.agent.remove(id);
        self.blocker.remove(id);
        self.abilities.remove(id);
        self.passive_abilities.remove(id);
        self.effects.remove(id);
        self.schedule.remove(id);
        self.summoner.remove(id);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Prototypes(pub HashMap<ObjType, Vec<Component>>);

fn init_component(component: &mut Component) {
    match component {
        Component::Agent(agent) => {
            agent.base_moves = agent.moves;
            agent.base_attacks = agent.attacks;
            agent.base_jokers = agent.jokers;
        }
        Component::Strength(strength) => {
            strength.base_strength = strength.strength;
        }
        _ => {}
    }
}

impl Prototypes {
    pub fn from_str(s: &str) -> Self {
        let mut prototypes: Prototypes = ron::de::from_str(s).expect("Can't parse the prototypes");
        prototypes.init_components();
        prototypes
    }

    pub fn init_components(&mut self) {
        for components in self.0.values_mut() {
            for component in components {
                init_component(component);
            }
        }
    }
}
