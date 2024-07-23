use serde::{Deserialize, Serialize};

use crate::{
    battle::{
        command::Command,
        component::{BelongsTo, Component, Meta, ObjType, Parts, Pos},
        heroes::{imp, imp_bomber, imp_summoner, toxic_imp, Hero, HeroObject},
        scenario::Scenario,
        Id, PlayerId, State, TileType,
    },
    map::{HexMap, PosHex},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Level {
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
}

impl Level {
    pub fn from_index(i: i32) -> Self {
        match i {
            0 => Level::Level0,
            1 => Level::Level1,
            2 => Level::Level2,
            3 => Level::Level3,
            4 => Level::Level4,
            5 => Level::Level5,
            _ => unimplemented!(),
        }
    }

    pub fn state(&self) -> State {
        match self {
            Level::Level0 => state_level0(),
            Level::Level1 => state_level1(),
            Level::Level2 => state_level2(),
            Level::Level3 => state_level3(),
            Level::Level4 => state_level4(),
            Level::Level5 => state_level5(),
        }
    }

    pub fn scenario(&self) -> Scenario {
        let scenario = match self {
            Level::Level0 => {
                r###"
        {"map_radius":3,"players_count":2,"rocky_tiles_count":0,"tiles":{},"randomized_objects":[{"owner":1,"typename":"imp","line":"Front","count":1}],"objects":[]}
        "###
            }
            Level::Level1 => {
                r###"
        {"map_radius":3,"players_count":2,"rocky_tiles_count":0,"tiles":{},"randomized_objects":[{"owner":1,"typename":"imp","line":"Front","count":3}],"objects":[]}
        "###
            }
            Level::Level2 => {
                r###"
        {"map_radius":3,"players_count":2,"rocky_tiles_count":0,"tiles":{},"randomized_objects":[{"owner":1,"typename":"imp","line":"Front","count":3},{"owner":1,"typename":"imp_bomber","line":"Back","count":1}],"objects":[]}
        "###
            }
            Level::Level3 => {
                r###"
        {"map_radius":3,"players_count":2,"rocky_tiles_count":0,"tiles":{},"randomized_objects":[{"owner":1,"typename":"imp","line":"Front","count":3},{"owner":1,"typename":"imp_bomber","line":"Back","count":1},{"owner":1,"typename":"toxic_imp","line":"Middle","count":1}],"objects":[]}
        "###
            }
            Level::Level4 => {
                r###"
        {"map_radius":3,"players_count":2,"rocky_tiles_count":0,"tiles":{},"randomized_objects":[{"owner":1,"typename":"imp","line":"Front","count":3},{"owner":1,"typename":"imp_bomber","line":"Back","count":1},{"owner":1,"typename":"toxic_imp","line":"Middle","count":2}],"objects":[]}
        "###
            }

            Level::Level5 => {
                r###"
        {"map_radius":3,"players_count":2,"rocky_tiles_count":0,"tiles":{},"randomized_objects":[{"owner":1,"typename":"imp","line":"Front","count":3},{"owner":1,"typename":"imp_bomber","line":"Back","count":1},{"owner":1,"typename":"toxic_imp","line":"Middle","count":2},{"owner":1,"typename":"imp_summoner","line":"Back","count":1}],"objects":[]}
        "###
            }
        };

        serde_json::from_str(scenario).unwrap()
    }

    pub fn fake_heros(&self) -> Vec<HeroObject> {
        match self {
            Level::Level0 => vec![HeroObject::new(Hero::Spearman, 1)],
            Level::Level1 => vec![
                HeroObject::new(Hero::Spearman, 1),
                HeroObject::new(Hero::Swordsman, 1),
            ],
            Level::Level2 => vec![
                HeroObject::new(Hero::HeavySpearman, 1),
                HeroObject::new(Hero::Swordsman, 1),
            ],
            Level::Level3 => vec![
                HeroObject::new(Hero::HeavySpearman, 1),
                HeroObject::new(Hero::EliteSwordsman, 1),
                HeroObject::new(Hero::Hammerman, 1),
            ],
            Level::Level4 => vec![
                HeroObject::new(Hero::Alchemist, 1),
                HeroObject::new(Hero::Swordsman, 1),
                HeroObject::new(Hero::EliteSpearman, 1),
            ],
            Level::Level5 => vec![
                HeroObject::new(Hero::HeavySpearman, 1),
                HeroObject::new(Hero::EliteSwordsman, 1),
                HeroObject::new(Hero::HeavyHammerman, 1),
            ],
        }
    }
}

pub fn state_level0() -> State {
    let mut parts = Parts::new();

    let id_imp = Id(0);
    parts.next_obj_id = Id(1);

    {
        for c in imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp, Pos(PosHex { q: 1, r: -2 }));
        parts.meta.insert(
            id_imp,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp, BelongsTo(PlayerId(1)));
    }

    let map = HexMap {
        tiles: vec![TileType::Plain; 49],
        size: 7,
        radius: 3,
    };

    State {
        parts,
        map,
        players_count: 2,
        player_id: PlayerId(0),
        battle_result: None,
        level: Level::Level0,

        #[cfg(feature = "debug")]
        turn_id: 0,
        #[cfg(feature = "debug")]
        commands: vec![],
        #[cfg(feature = "debug")]
        heroes: vec![],
    }
}

pub fn state_level1() -> State {
    let mut parts = Parts::new();

    let id_imp_0 = Id(0);
    let id_imp_1 = Id(1);
    let id_imp_2 = Id(2);
    parts.next_obj_id = Id(3);

    {
        for c in imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_0, x.clone());
                    parts.strength.insert(id_imp_1, x.clone());
                    parts.strength.insert(id_imp_2, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_0, x.clone());
                    parts.agent.insert(id_imp_1, x.clone());
                    parts.agent.insert(id_imp_2, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_0, x.clone());
                    parts.blocker.insert(id_imp_1, x.clone());
                    parts.blocker.insert(id_imp_2, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_0, Pos(PosHex { q: 1, r: -2 }));
        parts.pos.insert(id_imp_1, Pos(PosHex { q: 0, r: -1 }));
        parts.pos.insert(id_imp_2, Pos(PosHex { q: 2, r: -3 }));

        parts.meta.insert(
            id_imp_0,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_1,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_2,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_0, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_1, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_2, BelongsTo(PlayerId(1)));
    }

    let map = HexMap {
        tiles: vec![TileType::Plain; 49],
        size: 7,
        radius: 3,
    };

    State {
        parts,
        map,
        players_count: 2,
        player_id: PlayerId(0),
        battle_result: None,
        level: Level::Level1,

        #[cfg(feature = "debug")]
        turn_id: 0,
        #[cfg(feature = "debug")]
        commands: vec![],
        #[cfg(feature = "debug")]
        heroes: vec![],
    }
}

pub fn state_level2() -> State {
    let mut parts = Parts::new();

    let id_imp_0 = Id(0);
    let id_imp_1 = Id(1);
    let id_imp_2 = Id(2);
    let id_imp_3 = Id(3);
    parts.next_obj_id = Id(4);

    {
        for c in imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_0, x.clone());
                    parts.strength.insert(id_imp_1, x.clone());
                    parts.strength.insert(id_imp_2, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_0, x.clone());
                    parts.agent.insert(id_imp_1, x.clone());
                    parts.agent.insert(id_imp_2, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_0, x.clone());
                    parts.blocker.insert(id_imp_1, x.clone());
                    parts.blocker.insert(id_imp_2, x);
                }
                _ => unreachable!(),
            }
        }
        parts.pos.insert(id_imp_0, Pos(PosHex { q: 1, r: -2 }));
        parts.pos.insert(id_imp_1, Pos(PosHex { q: 0, r: -1 }));
        parts.pos.insert(id_imp_2, Pos(PosHex { q: 2, r: -3 }));

        parts.meta.insert(
            id_imp_0,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_1,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_2,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_0, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_1, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_2, BelongsTo(PlayerId(1)));
    }

    {
        for c in imp_bomber() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_3, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_3, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_3, x);
                }
                Component::Abilities(x) => {
                    parts.abilities.insert(id_imp_3, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_3, Pos(PosHex { q: 3, r: 0 }));
        parts.meta.insert(
            id_imp_3,
            Meta {
                name: ObjType("imp_bomber".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp_3, BelongsTo(PlayerId(1)));
    }

    let map = HexMap {
        tiles: vec![TileType::Plain; 49],
        size: 7,
        radius: 3,
    };

    State {
        parts,
        map,
        players_count: 2,
        player_id: PlayerId(0),
        battle_result: None,
        level: Level::Level2,

        #[cfg(feature = "debug")]
        turn_id: 0,
        #[cfg(feature = "debug")]
        commands: vec![],
        #[cfg(feature = "debug")]
        heroes: vec![],
    }
}

pub fn state_level3() -> State {
    let mut parts = Parts::new();

    let id_imp_0 = Id(0);
    let id_imp_1 = Id(1);
    let id_imp_2 = Id(2);
    let id_imp_3 = Id(3);
    let id_imp_4 = Id(4);

    parts.next_obj_id = Id(5);

    {
        for c in imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_0, x.clone());
                    parts.strength.insert(id_imp_1, x.clone());
                    parts.strength.insert(id_imp_2, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_0, x.clone());
                    parts.agent.insert(id_imp_1, x.clone());
                    parts.agent.insert(id_imp_2, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_0, x.clone());
                    parts.blocker.insert(id_imp_1, x.clone());
                    parts.blocker.insert(id_imp_2, x);
                }
                _ => unreachable!(),
            }
        }
        parts.pos.insert(id_imp_0, Pos(PosHex { q: 1, r: -2 }));
        parts.pos.insert(id_imp_1, Pos(PosHex { q: 0, r: -1 }));
        parts.pos.insert(id_imp_2, Pos(PosHex { q: 2, r: -3 }));

        parts.meta.insert(
            id_imp_0,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_1,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_2,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_0, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_1, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_2, BelongsTo(PlayerId(1)));
    }

    {
        for c in imp_bomber() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_3, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_3, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_3, x);
                }
                Component::Abilities(x) => {
                    parts.abilities.insert(id_imp_3, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_3, Pos(PosHex { q: 3, r: 0 }));
        parts.meta.insert(
            id_imp_3,
            Meta {
                name: ObjType("imp_bomber".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp_3, BelongsTo(PlayerId(1)));
    }

    {
        for c in toxic_imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_4, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_4, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_4, x);
                }
                Component::PassiveAbilities(x) => {
                    parts.passive_abilities.insert(id_imp_4, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_4, Pos(PosHex { q: 2, r: 1 }));
        parts.meta.insert(
            id_imp_4,
            Meta {
                name: ObjType("toxic_imp".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp_4, BelongsTo(PlayerId(1)));
    }

    let map = HexMap {
        tiles: vec![TileType::Plain; 49],
        size: 7,
        radius: 3,
    };

    State {
        parts,
        map,
        players_count: 2,
        player_id: PlayerId(0),
        battle_result: None,
        level: Level::Level3,

        #[cfg(feature = "debug")]
        turn_id: 0,
        #[cfg(feature = "debug")]
        commands: vec![],
        #[cfg(feature = "debug")]
        heroes: vec![],
    }
}

pub fn state_level4() -> State {
    let mut parts = Parts::new();

    let id_imp_0 = Id(0);
    let id_imp_1 = Id(1);
    let id_imp_2 = Id(2);
    let id_imp_3 = Id(3);
    let id_imp_4 = Id(4);
    let id_imp_5 = Id(5);

    parts.next_obj_id = Id(6);

    {
        for c in imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_0, x.clone());
                    parts.strength.insert(id_imp_1, x.clone());
                    parts.strength.insert(id_imp_2, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_0, x.clone());
                    parts.agent.insert(id_imp_1, x.clone());
                    parts.agent.insert(id_imp_2, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_0, x.clone());
                    parts.blocker.insert(id_imp_1, x.clone());
                    parts.blocker.insert(id_imp_2, x);
                }
                _ => unreachable!(),
            }
        }
        parts.pos.insert(id_imp_0, Pos(PosHex { q: 1, r: -2 }));
        parts.pos.insert(id_imp_1, Pos(PosHex { q: 0, r: -1 }));
        parts.pos.insert(id_imp_2, Pos(PosHex { q: 2, r: -3 }));

        parts.meta.insert(
            id_imp_0,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_1,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_2,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_0, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_1, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_2, BelongsTo(PlayerId(1)));
    }

    {
        for c in imp_bomber() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_3, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_3, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_3, x);
                }
                Component::Abilities(x) => {
                    parts.abilities.insert(id_imp_3, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_3, Pos(PosHex { q: 3, r: 0 }));
        parts.meta.insert(
            id_imp_3,
            Meta {
                name: ObjType("imp_bomber".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp_3, BelongsTo(PlayerId(1)));
    }

    {
        for c in toxic_imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_4, x.clone());
                    parts.strength.insert(id_imp_5, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_4, x.clone());
                    parts.agent.insert(id_imp_5, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_4, x.clone());
                    parts.blocker.insert(id_imp_5, x);
                }
                Component::PassiveAbilities(x) => {
                    parts.passive_abilities.insert(id_imp_4, x.clone());
                    parts.passive_abilities.insert(id_imp_5, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_4, Pos(PosHex { q: 2, r: 1 }));
        parts.pos.insert(id_imp_5, Pos(PosHex { q: 2, r: -1 }));

        parts.meta.insert(
            id_imp_4,
            Meta {
                name: ObjType("toxic_imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_5,
            Meta {
                name: ObjType("toxic_imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_4, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_5, BelongsTo(PlayerId(1)));
    }

    let map = HexMap {
        tiles: vec![TileType::Plain; 49],
        size: 7,
        radius: 3,
    };

    State {
        parts,
        map,
        players_count: 2,
        player_id: PlayerId(0),
        battle_result: None,
        level: Level::Level4,

        #[cfg(feature = "debug")]
        turn_id: 0,
        #[cfg(feature = "debug")]
        commands: vec![],
        #[cfg(feature = "debug")]
        heroes: vec![],
    }
}

pub fn state_level5() -> State {
    let mut parts = Parts::new();

    let id_imp_0 = Id(0);
    let id_imp_1 = Id(1);
    let id_imp_2 = Id(2);
    let id_imp_3 = Id(3);
    let id_imp_4 = Id(4);
    let id_imp_5 = Id(5);
    let id_imp_6 = Id(6);

    parts.next_obj_id = Id(7);

    {
        for c in imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_0, x.clone());
                    parts.strength.insert(id_imp_1, x.clone());
                    parts.strength.insert(id_imp_2, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_0, x.clone());
                    parts.agent.insert(id_imp_1, x.clone());
                    parts.agent.insert(id_imp_2, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_0, x.clone());
                    parts.blocker.insert(id_imp_1, x.clone());
                    parts.blocker.insert(id_imp_2, x);
                }
                _ => unreachable!(),
            }
        }
        parts.pos.insert(id_imp_0, Pos(PosHex { q: 1, r: -2 }));
        parts.pos.insert(id_imp_1, Pos(PosHex { q: 0, r: -1 }));
        parts.pos.insert(id_imp_2, Pos(PosHex { q: 2, r: -3 }));

        parts.meta.insert(
            id_imp_0,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_1,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_2,
            Meta {
                name: ObjType("imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_0, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_1, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_2, BelongsTo(PlayerId(1)));
    }

    {
        for c in imp_bomber() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_3, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_3, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_3, x);
                }
                Component::Abilities(x) => {
                    parts.abilities.insert(id_imp_3, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_3, Pos(PosHex { q: 3, r: 0 }));
        parts.meta.insert(
            id_imp_3,
            Meta {
                name: ObjType("imp_bomber".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp_3, BelongsTo(PlayerId(1)));
    }

    {
        for c in toxic_imp() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_4, x.clone());
                    parts.strength.insert(id_imp_5, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_4, x.clone());
                    parts.agent.insert(id_imp_5, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_4, x.clone());
                    parts.blocker.insert(id_imp_5, x);
                }
                Component::PassiveAbilities(x) => {
                    parts.passive_abilities.insert(id_imp_4, x.clone());
                    parts.passive_abilities.insert(id_imp_5, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_4, Pos(PosHex { q: 2, r: 1 }));
        parts.pos.insert(id_imp_5, Pos(PosHex { q: 2, r: -1 }));

        parts.meta.insert(
            id_imp_4,
            Meta {
                name: ObjType("toxic_imp".to_string()),
            },
        );
        parts.meta.insert(
            id_imp_5,
            Meta {
                name: ObjType("toxic_imp".to_string()),
            },
        );

        parts.belongs_to.insert(id_imp_4, BelongsTo(PlayerId(1)));
        parts.belongs_to.insert(id_imp_5, BelongsTo(PlayerId(1)));
    }

    {
        for c in imp_summoner() {
            match c {
                Component::Strength(x) => {
                    parts.strength.insert(id_imp_6, x);
                }
                Component::Agent(x) => {
                    parts.agent.insert(id_imp_6, x);
                }
                Component::Blocker(x) => {
                    parts.blocker.insert(id_imp_6, x);
                }
                Component::Abilities(x) => {
                    parts.abilities.insert(id_imp_6, x);
                }
                Component::PassiveAbilities(x) => {
                    parts.passive_abilities.insert(id_imp_6, x);
                }
                Component::Summoner(x) => {
                    parts.summoner.insert(id_imp_6, x);
                }
                _ => unreachable!(),
            }
        }

        parts.pos.insert(id_imp_6, Pos(PosHex { q: 3, r: -1 }));
        parts.meta.insert(
            id_imp_6,
            Meta {
                name: ObjType("imp_summoner".to_string()),
            },
        );
        parts.belongs_to.insert(id_imp_6, BelongsTo(PlayerId(1)));
    }

    let map = HexMap {
        tiles: vec![TileType::Plain; 49],
        size: 7,
        radius: 3,
    };

    State {
        parts,
        map,
        players_count: 2,
        player_id: PlayerId(0),
        battle_result: None,
        level: Level::Level5,

        #[cfg(feature = "debug")]
        turn_id: 0,
        #[cfg(feature = "debug")]
        commands: vec![],
        #[cfg(feature = "debug")]
        heroes: vec![],
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub level: Level,
    pub commands: Vec<Vec<Command>>,
    pub heroes: Vec<HeroObject>,
}

#[cfg(not(feature = "event"))]
#[cfg(test)]
mod test {
    use crate::{
        battle::State,
        game::{
            state_level0, state_level1, state_level2, state_level3, state_level4, state_level5,
            Level,
        },
        utils::SimpleRng,
    };

    #[test]
    fn test_level_0() {
        let level = Level::Level0;
        let mut rng = SimpleRng::seed_from_u32(0);
        let state = State::new(level.scenario(), level, &mut rng);
        println!("{:?}", state.parts().pos);
        assert_eq!(state_level0(), state);
    }

    #[test]
    fn test_level_1() {
        let level = Level::Level1;
        let mut rng = SimpleRng::seed_from_u32(0);
        let state = State::new(level.scenario(), level, &mut rng);
        println!("{:?}", state.parts().pos);
        assert_eq!(state_level1(), state);
    }

    #[test]
    fn test_level_2() {
        let level = Level::Level2;
        let mut rng = SimpleRng::seed_from_u32(0);
        let state = State::new(level.scenario(), level, &mut rng);
        println!("{:?}", state.parts().pos);
        assert_eq!(state_level2(), state);
    }

    #[test]
    fn test_level_3() {
        let level = Level::Level3;
        let mut rng = SimpleRng::seed_from_u32(0);
        let state = State::new(level.scenario(), level, &mut rng);
        println!("{:?}", state.parts().pos);
        assert_eq!(state_level3(), state);
    }

    #[test]
    fn test_level_4() {
        let level = Level::Level4;
        let mut rng = SimpleRng::seed_from_u32(0);
        let state = State::new(level.scenario(), level, &mut rng);
        println!("{:?}", state.parts().pos);
        assert_eq!(state_level4(), state);
    }

    #[test]
    fn test_level_5() {
        let level = Level::Level5;
        let mut rng = SimpleRng::seed_from_u32(0);
        let state = State::new(level.scenario(), level, &mut rng);
        println!("{:?}", state.parts().pos);
        assert_eq!(state_level5(), state);
    }
}
