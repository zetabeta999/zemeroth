use serde::{Deserialize, Serialize};

use crate::{
    battle::{
        command::{self},
        component::{
            Abilities, Agent, BelongsTo, Blocker, Component, Meta, ObjType, Parts, Pos, Strength,
        },
        event::Event,
        execute,
        heroes::{prototype_for, Hero, HeroObject},
        scenario::{self, Scenario},
        state::apply::apply,
        Id, PlayerId, TileType,
    },
    game::Level,
    map,
    utils::SimpleRng,
};

#[cfg(feature = "debug")]
use crate::battle::command::Command;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BattleResult {
    pub winner_id: PlayerId,
    pub survivor_types: Vec<ObjType>,
    pub level: Level,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub parts: Parts,
    pub map: map::HexMap<TileType>,
    pub players_count: i32,

    pub player_id: PlayerId,
    pub battle_result: Option<BattleResult>,
    pub level: Level,

    #[cfg(feature = "debug")]
    pub commands: Vec<Vec<Command>>,
    #[cfg(feature = "debug")]
    pub turn_id: usize,
    #[cfg(feature = "debug")]
    pub heroes: Vec<HeroObject>,
}

impl State {
    pub fn new(
        scenario: Scenario,
        level: Level,
        rng: &mut SimpleRng,
        #[cfg(feature = "event")] cb: execute::Cb,
    ) -> Self {
        let mut this = Self {
            map: map::HexMap::new(scenario.map_radius),
            player_id: PlayerId(0),
            parts: Parts::new(),
            battle_result: None,
            #[cfg(feature = "debug")]
            commands: vec![vec![]],
            #[cfg(feature = "debug")]
            turn_id: 0,
            #[cfg(feature = "debug")]
            heroes: vec![],
            level,
            players_count: scenario.players_count,
        };
        this.create_terrain(&scenario, rng);
        this.create_objects(
            &scenario,
            rng,
            #[cfg(feature = "event")]
            cb,
        );

        this
    }

    // TODO: Handle Scenario::exact_tiles
    fn create_terrain(&mut self, scenario: &Scenario, rng: &mut SimpleRng) {
        for _ in 0..scenario.rocky_tiles_count {
            let pos = match scenario::random_free_pos(self, rng) {
                Some(pos) => pos,
                None => continue,
            };
            self.map.set_tile(pos, TileType::Rocks);
        }
    }

    // TODO: Handle Scenario::objects
    fn create_objects(
        &mut self,
        scenario: &Scenario,
        rng: &mut SimpleRng,
        #[cfg(feature = "event")] cb: execute::Cb,
    ) {
        let player_id_initial = self.player_id();
        for group in scenario.randomized_objects.clone() {
            if let Some(player_id) = group.owner {
                self.set_player_id(player_id);
            }
            for _ in 0..group.count {
                let pos = match scenario::random_pos(self, group.owner, group.line, rng) {
                    Some(pos) => pos,
                    None => {
                        println!("Can't find the position");
                        continue;
                    }
                };
                let command = command::Create {
                    prototype: group.typename.clone(),
                    pos,
                    owner: group.owner,
                }
                .into();
                execute::execute(
                    self,
                    &command,
                    rng,
                    #[cfg(feature = "event")]
                    cb,
                )
                .expect("Can't create an object");
            }
        }
        for group in scenario.objects.clone() {
            if let Some(player_id) = group.owner {
                self.set_player_id(player_id);
            }
            let command = command::Create {
                prototype: group.typename.clone(),
                pos: group.pos,
                owner: group.owner,
            }
            .into();
            execute::execute(
                self,
                &command,
                rng,
                #[cfg(feature = "event")]
                cb,
            )
            .expect("Can't create an object");
        }
        self.set_player_id(player_id_initial);
    }

    pub fn create_heroes(
        &mut self,
        heros: &[HeroObject],
        rng: &mut SimpleRng,
        #[cfg(feature = "event")] cb: execute::Cb,
    ) {
        for group in heros {
            let hero = Hero::from_index(group.index);
            let name = hero.name();
            let line = hero.line();

            for _ in 0..group.count {
                let pos = match scenario::random_pos(self, Some(PlayerId(0)), Some(line), rng) {
                    Some(pos) => pos,
                    None => {
                        println!("Can't find the position");
                        continue;
                    }
                };
                let command = command::Create {
                    prototype: ObjType(name.clone()),
                    pos,
                    owner: Some(PlayerId(0)),
                }
                .into();
                execute::execute(
                    self,
                    &command,
                    rng,
                    #[cfg(feature = "event")]
                    cb,
                )
                .expect("Can't create an object");
            }
        }

        #[cfg(feature = "debug")]
        {
            self.heroes = heros.to_vec();
        }
    }

    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    pub fn next_player_id(&self) -> PlayerId {
        let current_player_id = PlayerId(self.player_id().0 + 1);
        if current_player_id.0 < self.players_count() {
            current_player_id
        } else {
            PlayerId(0)
        }
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn strength(&self, id: &Id) -> &Strength {
        self.parts().strength.get(id).unwrap()
    }

    pub fn belongs_to(&self, id: &Id) -> &BelongsTo {
        self.parts().belongs_to.get(id).unwrap()
    }

    pub fn agent(&self, id: &Id) -> &Agent {
        self.parts().agent.get(id).unwrap()
    }

    pub fn abilities(&self, id: &Id) -> &Abilities {
        self.parts().abilities.get(id).unwrap()
    }

    pub fn pos(&self, id: &Id) -> &Pos {
        self.parts().pos.get(id).unwrap()
    }

    pub fn meta(&self, id: &Id) -> &Meta {
        self.parts().meta.get(id).unwrap()
    }

    pub fn blocker(&self, id: &Id) -> &Blocker {
        self.parts().blocker.get(id).unwrap()
    }

    pub fn map(&self) -> &map::HexMap<TileType> {
        &self.map
    }

    pub fn players_count(&self) -> i32 {
        self.players_count
    }

    pub(crate) fn prototype_for(&self, name: &ObjType) -> Vec<Component> {
        prototype_for(&name.0)
    }

    pub fn battle_result(&self) -> &Option<BattleResult> {
        &self.battle_result
    }
}

/// Public mutators. Be careful with them!
impl State {
    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    pub(crate) fn set_player_id(&mut self, new_value: PlayerId) {
        self.player_id = new_value;
    }

    pub(crate) fn set_battle_result(&mut self, result: BattleResult) {
        self.battle_result = Some(result);
    }

    pub(crate) fn alloc_id(&mut self) -> Id {
        self.parts.alloc_id()
    }

    pub fn apply(&mut self, event: &Event) {
        apply(self, event);
    }
}
