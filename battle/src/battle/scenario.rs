use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    battle::{
        component::ObjType,
        state::{self, State},
        PlayerId, TileType,
    },
    map::{self, PosHex},
    utils::SimpleRng,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleType {
    Skirmish,
    CampaignNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectsGroup {
    pub owner: Option<PlayerId>,
    pub typename: ObjType,
    pub line: Option<Line>,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub owner: Option<PlayerId>,
    pub typename: ObjType,
    pub pos: PosHex,
}

// TODO: Split into `Scenario` (exact info) and `ScenarioTemplate`?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Scenario {
    pub map_radius: i32,
    pub players_count: i32,

    // TODO: rename it to `randomized_tiles` later (not only `TileType::Rocks`)
    pub rocky_tiles_count: i32,

    pub tiles: HashMap<PosHex, TileType>,

    pub randomized_objects: Vec<ObjectsGroup>,

    pub objects: Vec<Object>,
}

#[derive(Clone, Debug, derive_more::From)]
pub enum Error {
    MapIsTooSmall,
    PosOutsideOfMap(PosHex),
    NoPlayerAgents,
    NoEnemyAgents,
    UnsupportedPlayersCount(i32),
}

impl Scenario {
    pub fn check(&self) -> Result<(), Error> {
        if self.players_count != 2 {
            return Err(Error::UnsupportedPlayersCount(self.players_count));
        }
        if self.map_radius < 3 {
            return Err(Error::MapIsTooSmall);
        }
        let origin = PosHex { q: 0, r: 0 };
        for obj in &self.objects {
            let dist = map::distance_hex(origin, obj.pos);
            if dist > self.map_radius {
                return Err(Error::PosOutsideOfMap(obj.pos));
            }
        }
        let any_exact_player_agents = self
            .objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(0)));
        let any_random_player_agents = self
            .randomized_objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(0)));
        if !any_exact_player_agents && !any_random_player_agents {
            return Err(Error::NoPlayerAgents);
        }
        let any_exact_enemy_agents = self
            .objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(1)));
        let any_random_enemy_agents = self
            .randomized_objects
            .iter()
            .any(|obj| obj.owner == Some(PlayerId(1)));
        if !any_exact_enemy_agents && !any_random_enemy_agents {
            return Err(Error::NoEnemyAgents);
        }
        Ok(())
    }
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            map_radius: 5,
            players_count: 2,
            rocky_tiles_count: 0,
            tiles: HashMap::new(),
            randomized_objects: Vec::new(),
            objects: Vec::new(),
        }
    }
}

pub fn random_free_pos(state: &State, rng: &mut SimpleRng) -> Option<PosHex> {
    let attempts = 30;
    let radius = state.map().radius();
    for _ in 0..attempts {
        let pos = PosHex {
            q: rng.gen_range(-radius, radius),
            r: rng.gen_range(-radius, radius),
        };
        if state::is_tile_plain_and_completely_free(state, pos) {
            return Some(pos);
        }
    }
    None
}

fn middle_range(min: i32, max: i32) -> (i32, i32) {
    assert!(min <= max);
    let size = max - min;
    let half = size / 2;
    let forth = size / 4;
    let min = half - forth;
    let mut max = half + forth;
    if min == max {
        max += 1;
    }
    (min, max)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Line {
    Any,
    Front,
    Middle,
    Back,
}

impl Line {
    pub fn to_range(self, radius: i32) -> (i32, i32) {
        match self {
            Line::Front => (radius / 2, radius + 1),
            Line::Middle => middle_range(0, radius),
            Line::Back => (0, radius / 2),
            Line::Any => (0, radius + 1),
        }
    }

    pub fn get_range(self, radius: i32) -> (i32, i32) {
        match (self, radius) {
            (Line::Back, 3) => (0, 1),
            (Line::Middle, 3) => (1, 2),
            (Line::Front, 3) => (1, 4),
            (Line::Any, 3) => (0, 4),
            (Line::Back, 4) => (0, 2),
            (Line::Middle, 4) => (1, 3),
            (Line::Front, 4) => (2, 5),
            (Line::Any, 4) => (0, 5),
            (Line::Back, 5) => (0, 2),
            (Line::Middle, 5) => (1, 3),
            (Line::Front, 5) => (2, 6),
            (Line::Any, 5) => (0, 6),
            (_, _) => unimplemented!(),
        }
    }
}

fn random_free_sector_pos(
    state: &State,
    player_id: PlayerId,
    line: Line,
    rng: &mut SimpleRng,
) -> Option<PosHex> {
    let attempts = 30;
    let radius = state.map().radius();
    let (min, max) = line.get_range(radius);
    for _ in 0..attempts {
        let q = radius - rng.gen_range(min, max);
        let pos = PosHex {
            q: match player_id.0 {
                0 => -q,
                1 => q,
                _ => unimplemented!(),
            },
            r: rng.gen_range(-radius, radius + 1),
        };

        if state::is_tile_completely_free(state, pos) {
            return Some(pos);
        }
    }
    None
}

pub fn random_pos(
    state: &State,
    owner: Option<PlayerId>,
    line: Option<Line>,
    rng: &mut SimpleRng,
) -> Option<PosHex> {
    match (owner, line) {
        (Some(player_id), Some(line)) => random_free_sector_pos(state, player_id, line, rng),
        _ => random_free_pos(state, rng),
    }
}

#[cfg(test)]
mod tests {

    use super::middle_range;

    #[test]
    fn test_middle_range() {
        assert_eq!(middle_range(0, 3), (1, 2));
        assert_eq!(middle_range(0, 4), (1, 3));
        assert_eq!(middle_range(0, 5), (1, 3));
        assert_eq!(middle_range(0, 6), (2, 4));
        assert_eq!(middle_range(0, 7), (2, 4));
        assert_eq!(middle_range(0, 8), (2, 6));
        assert_eq!(middle_range(0, 9), (2, 6));
        assert_eq!(middle_range(0, 10), (3, 7));
    }
}
