use crate::{
    battle::{
        self,
        ability::{self, Ability, PassiveAbility},
        component::ObjType,
        effect, Id, PlayerId, TileType,
    },
    map::{self, PosHex},
};

pub use self::private::{BattleResult, State};

use super::BAD_ID;

pub mod apply;
pub mod private;

pub fn is_agent_belong_to(state: &State, player_id: PlayerId, id: Id) -> bool {
    state.belongs_to(&id).0 == player_id
}

pub fn is_tile_blocked(state: &State, pos: PosHex) -> bool {
    assert!(state.map().is_inboard(pos));
    for p in state.parts().pos.values() {
        if p.0 == pos {
            return true;
        }
    }

    false
}

pub fn is_tile_blocked_not_check_inboard(state: &State, pos: PosHex) -> bool {
    for p in state.parts().pos.values() {
        if p.0 == pos {
            return true;
        }
    }

    false
}

pub fn is_tile_plain_and_completely_free(state: &State, pos: PosHex) -> bool {
    if !state.map().is_inboard(pos) || state.map().tile(pos) != TileType::Plain {
        return false;
    }

    state.parts().pos.iter().all(|(_, x)| x.0 != pos)
}

pub fn is_tile_completely_free(state: &State, pos: PosHex) -> bool {
    if !state.map().is_inboard(pos) {
        return false;
    }

    state.parts().pos.iter().all(|(_, x)| x.0 != pos)
}

pub fn is_lasting_effect_over(state: &State, id: &Id, timed_effect: &effect::Timed) -> bool {
    if let effect::Lasting::Poison = timed_effect.effect {
        let strength = state.strength(id).strength;
        if strength.0 <= 1 {
            return true;
        }
    }
    timed_effect.duration.is_over()
}

/// Are there any enemy agents on the adjacent tiles?
pub fn check_enemies_around(state: &State, pos: PosHex, player_id: PlayerId) -> bool {
    for dir in map::dirs() {
        let neighbor_pos = map::Dir::get_neighbor_pos(pos, dir);
        if let Some(id) = agent_id_at_opt(state, neighbor_pos) {
            let neighbor_player_id = state.belongs_to(&id).0;
            if neighbor_player_id != player_id {
                return true;
            }
        }
    }
    false
}

pub fn ids_at(state: &State, pos: PosHex) -> Vec<Id> {
    let mut r: Vec<Id> = state
        .parts()
        .pos
        .iter()
        .filter(|(_, p)| p.0 == pos)
        .map(|(id, _)| *id)
        .collect();
    r.sort_by_key(|x| x.0);
    r
}

pub fn obj_with_passive_ability_at(
    state: &State,
    pos: PosHex,
    ability: PassiveAbility,
) -> Option<Id> {
    for id in ids_at(state, pos) {
        if let Some(abilities) = state.parts().passive_abilities.get(&id) {
            for &current_ability in &abilities.0 {
                if current_ability == ability {
                    return Some(id);
                }
            }
        }
    }
    None
}

pub fn blocker_id_at(state: &State, pos: PosHex) -> Id {
    blocker_id_at_opt(state, pos).unwrap()
}

pub fn blocker_id_at_opt(state: &State, pos: PosHex) -> Option<Id> {
    let ids = blocker_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn agent_id_at_opt(state: &State, pos: PosHex) -> Option<Id> {
    let ids = agent_ids_at(state, pos);
    if ids.len() == 1 {
        Some(ids[0])
    } else {
        None
    }
}

pub fn get_id_by_pos(state: &State, pos: PosHex) -> Id {
    for (i, p) in state.parts.pos.iter() {
        if p.0 == pos {
            return *i;
        }
    }
    Id(BAD_ID)
}

pub fn agent_ids_at(state: &State, pos: PosHex) -> Vec<Id> {
    let mut ids = vec![];
    for k in state.parts().agent.keys() {
        if state.pos(k).0 == pos {
            ids.push(*k)
        }
    }
    ids.sort_by_key(|x| x.0);
    ids
}

pub fn blocker_ids_at(state: &State, pos: PosHex) -> Vec<Id> {
    let mut ids = vec![];
    for k in state.parts().blocker.keys() {
        if state.pos(k).0 == pos {
            ids.push(*k)
        }
    }
    ids.sort_by_key(|x| x.0);
    ids
}

pub fn players_agent_ids(state: &State, player_id: PlayerId) -> Vec<Id> {
    let mut ids = vec![];
    for k in state.parts().agent.keys() {
        if state.belongs_to(k).0 == player_id {
            ids.push(*k)
        }
    }
    ids.sort_by_key(|x| x.0);
    ids
}

pub fn enemy_agent_ids(state: &State, player_id: PlayerId) -> Vec<Id> {
    let mut ids = vec![];
    for k in state.parts().agent.keys() {
        if state.belongs_to(k).0 != player_id {
            ids.push(*k)
        }
    }
    ids.sort_by_key(|x| x.0);
    ids
}

pub fn enemy_agent_ids_unsort(state: &State, player_id: PlayerId) -> Vec<Id> {
    let mut ids = vec![];
    for k in state.parts().agent.keys() {
        if state.belongs_to(k).0 != player_id {
            ids.push(*k)
        }
    }
    ids
}

pub fn enemy_agent_count(state: &State, player_id: PlayerId) -> u8 {
    let mut count = 0;
    for k in state.parts().agent.keys() {
        if state.belongs_to(k).0 != player_id {
            count += 1
        }
    }
    count
}

pub fn free_neighbor_positions(state: &State, origin: PosHex, count: usize) -> Vec<PosHex> {
    let mut positions = Vec::new();
    for dir in map::dirs().collect::<Vec<_>>() {
        let pos = map::Dir::get_neighbor_pos(origin, dir);
        if state.map().is_inboard(pos) && !is_tile_blocked(state, pos) {
            positions.push(pos);
            if positions.len() == count {
                break;
            }
        }
    }
    positions
}

pub fn sort_agent_ids_by_distance_to_enemies(state: &State, ids: &mut [Id]) {
    ids.sort_unstable_by_key(|&id| {
        let agent_player_id = state.belongs_to(&id).0;
        let agent_pos = state.pos(&id).0;
        let mut min_distance = state.map().height();
        for enemy_id in enemy_agent_ids(state, agent_player_id) {
            let enemy_pos = state.pos(&enemy_id).0;
            let distance = map::distance_hex(agent_pos, enemy_pos);
            if distance < min_distance {
                min_distance = distance;
            }
        }
        min_distance
    });
}

pub fn players_agent_types(state: &State, player_id: PlayerId) -> Vec<ObjType> {
    players_agent_ids(state, player_id)
        .into_iter()
        .map(|id| state.parts().meta.get(&id).unwrap().name.clone())
        .collect()
}

pub fn can_agent_use_ability(state: &State, id: &Id, ability: &Ability) -> bool {
    let agent_player_id = state.belongs_to(id).0;
    let agent = state.agent(id);
    let has_actions = agent.attacks > battle::Attacks(0) || agent.jokers > battle::Jokers(0);
    let is_player_agent = agent_player_id == state.player_id();
    let abilities = &state.abilities(id).0;
    let r_ability = abilities.iter().find(|r| &r.ability == ability).unwrap();
    let is_ready = r_ability.status == ability::Status::Ready;
    is_player_agent && is_ready && has_actions
}
