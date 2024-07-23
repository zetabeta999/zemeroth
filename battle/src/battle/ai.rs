use crate::{
    battle::{
        ability::Ability,
        command::{self, Command},
        effect,
        movement::Path,
        state, Id, PlayerId, State,
    },
    map::{self, distance_hex},
};

use super::{
    ability::{RechargeableAbility, Status},
    check::{
        check_agent_can_attack, check_agent_can_move, check_command_attack, check_max_distance,
    },
    PosHex,
};

#[allow(dead_code)]
fn does_agent_have_ability(state: &State, id: Id, ability: &Ability) -> bool {
    if let Some(abilities) = state.parts().abilities.get(&id) {
        for current_ability in &abilities.0 {
            if ability == &current_ability.ability {
                return true;
            }
        }
    }
    false
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct DistanceRange {
    min: i32,
    max: i32,
}

#[derive(Debug, Clone)]
pub struct Ai {
    id: PlayerId,

    obj_ids: Vec<Id>,
}

impl Ai {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            obj_ids: vec![],
        }
    }

    pub fn update_obj_ids(&mut self, state: &State) {
        let ids = state::players_agent_ids(state, self.id);
        self.obj_ids = ids;
    }

    fn trim(&mut self, ids: &[usize]) {
        ids.iter().for_each(|i| {
            self.obj_ids.remove(*i);
        });
    }

    fn try_throw_bomb(&self, state: &State, agent_id: Id) -> Option<Command> {
        let agent_pos = state.pos(&agent_id).0;

        for target_id in state::enemy_agent_ids(state, self.id) {
            let target_pos = state.pos(&target_id).0;
            for dir in map::dirs().collect::<Vec<_>>() {
                let pos = map::Dir::get_neighbor_pos(target_pos, dir);
                if !state.map().is_inboard(pos) || state::is_tile_blocked(state, pos) {
                    continue;
                }

                let command = command::UseAbility {
                    id: agent_id,
                    pos,
                    ability: Ability::BombDemonic,
                }
                .into();
                if check_agent_can_attack(state, agent_id).is_ok()
                    && check_max_distance(agent_pos, pos, 3).is_ok()
                {
                    return Some(command);
                }
            }
        }
        None
    }

    fn try_summon_imp(&self, state: &State, agent_id: Id) -> Option<Command> {
        let command = command::UseAbility {
            id: agent_id,
            pos: state.pos(&agent_id).0,
            ability: Ability::Summon,
        }
        .into();
        if check_agent_can_attack(state, agent_id).is_ok() {
            return Some(command);
        }
        None
    }

    fn try_bloodlust_imp(&self, state: &State, agent_id: Id) -> Option<Command> {
        let imps = ["imp", "toxic_imp"];

        'target_loop: for target_id in state::players_agent_ids(state, self.id) {
            let type_name = state.meta(&target_id).name.0.as_str();
            if !imps.contains(&type_name) {
                continue;
            }

            if let Some(effects) = state.parts().effects.get(&target_id) {
                for effect in &effects.0 {
                    if effect.effect == effect::Lasting::Bloodlust {
                        continue 'target_loop;
                    }
                }
            }

            let command = command::UseAbility {
                id: agent_id,
                pos: state.pos(&target_id).0,
                ability: Ability::Bloodlust,
            }
            .into();
            if check_agent_can_attack(state, agent_id).is_ok() {
                return Some(command);
            }
        }
        None
    }

    fn try_to_attack(&self, state: &State, agent_id: Id) -> Option<Command> {
        for &target_id in state::enemy_agent_ids(state, self.id).iter() {
            let attacker_id = agent_id;
            let command = command::Attack {
                attacker_id,
                target_id,
            };
            if check_command_attack(state, &command).is_ok() {
                return Some(command.into());
            }
        }
        None
    }

    fn try_to_move_closer(&mut self, state: &State, id: Id) -> Option<Path> {
        let agent = state.agent(&id);
        let agent_move_point = agent.move_points.0;
        let agent_pos = state.pos(&id).0;

        let enemys = state::enemy_agent_ids(state, self.id);

        let mut to_pos = state.pos(&enemys[0]).0;
        let mut min_distance = distance_hex(to_pos, agent_pos);
        for x in enemys.iter().skip(1) {
            let pos = state.pos(x).0;
            let dist = distance_hex(pos, agent_pos);
            if dist < min_distance {
                min_distance = dist;
                to_pos = pos;
            }
        }

        let radius = state.map().radius();

        let mut target_pos = None;
        let mut nearest = 0;

        match (to_pos.q - agent_pos.q > 0, to_pos.r - agent_pos.r > 0) {
            (true, true) => {
                for i in 0..=agent_move_point {
                    let q = agent_pos.q + i;
                    if q > radius {
                        break;
                    }

                    for j in 0..=agent_move_point {
                        let r = agent_pos.r + j;
                        if r > radius || q + r > radius || q + r < -radius {
                            break;
                        }

                        let pos = PosHex { q, r };

                        if !state::is_tile_blocked_not_check_inboard(state, pos) {
                            if target_pos.is_some() {
                                let d = distance_hex(pos, to_pos);
                                if d < nearest {
                                    target_pos = Some(pos);
                                    nearest = d;
                                }
                            } else {
                                nearest = distance_hex(pos, to_pos);
                                target_pos = Some(pos);
                            }
                        }
                    }
                }
            }
            (true, false) => {
                for i in 0..=agent_move_point {
                    let q = agent_pos.q + i;
                    if q > radius {
                        break;
                    }

                    for j in 0..=agent_move_point {
                        let r = agent_pos.r - j;
                        if r < -radius || q + r > radius || q + r < -radius {
                            break;
                        }
                        let pos = PosHex { q, r };

                        if !state::is_tile_blocked_not_check_inboard(state, pos) {
                            if target_pos.is_some() {
                                let d = distance_hex(pos, to_pos);
                                if d < nearest {
                                    target_pos = Some(pos);
                                    nearest = d;
                                }
                            } else {
                                nearest = distance_hex(pos, to_pos);
                                target_pos = Some(pos);
                            }
                        }
                    }
                }
            }
            (false, true) => {
                for i in 0..=agent_move_point {
                    let q = agent_pos.q - i;
                    if q < -radius {
                        break;
                    }

                    for j in 0..=agent_move_point {
                        let r = agent_pos.r + j;
                        if r > radius || q + r > radius || q + r < -radius {
                            break;
                        }
                        let pos = PosHex { q, r };

                        if !state::is_tile_blocked_not_check_inboard(state, pos) {
                            if target_pos.is_some() {
                                let d = distance_hex(pos, to_pos);
                                if d < nearest {
                                    target_pos = Some(pos);
                                    nearest = d;
                                }
                            } else {
                                nearest = distance_hex(pos, to_pos);
                                target_pos = Some(pos);
                            }
                        }
                    }
                }
            }
            (false, false) => {
                for i in 0..=agent_move_point {
                    let q = agent_pos.q - i;
                    if q < -radius {
                        break;
                    }

                    for j in 0..=agent_move_point {
                        let r = agent_pos.r - j;
                        if r < -radius || q + r > radius || q + r < -radius {
                            break;
                        }
                        let pos = PosHex { q, r };

                        if !state::is_tile_blocked_not_check_inboard(state, pos) {
                            if target_pos.is_some() {
                                let d = distance_hex(pos, to_pos);
                                if d < nearest {
                                    target_pos = Some(pos);
                                    nearest = d;
                                }
                            } else {
                                nearest = distance_hex(pos, to_pos);
                                target_pos = Some(pos);
                            }
                        }
                    }
                }
            }
        };

        if target_pos.is_none() {
            return None;
        }

        let path = Path::new(vec![agent_pos, target_pos.unwrap()]);
        let cost = path.cost_for(state, id);
        if cost.0 > agent_move_point {
            return None;
        }

        Some(path)
    }

    pub fn try_to_move(&mut self, state: &State, agent_id: Id) -> Option<Command> {
        let path_result = self.try_to_move_closer(state, agent_id);
        if let Some(path) = path_result {
            let command = command::MoveTo { id: agent_id, path };

            if check_agent_can_move(state, command.id).is_ok() {
                return Some(command.into());
            }
        }

        None
    }

    pub fn command(&mut self, state: &State) -> Option<Command> {
        if state.battle_result().is_some() {
            return None;
        }

        let mut ids = vec![];

        for (i, agent_id) in self.obj_ids.clone().into_iter().enumerate() {
            ids.push(i);

            if let Some(abilitys) = state.parts().abilities.get(&agent_id) {
                if abilitys.0.contains(&RechargeableAbility {
                    ability: Ability::Summon,
                    status: Status::Ready,
                }) {
                    if let Some(summon_command) = self.try_summon_imp(state, agent_id) {
                        self.trim(&ids);
                        return Some(summon_command);
                    }
                }

                if abilitys.0.contains(&RechargeableAbility {
                    ability: Ability::Bloodlust,
                    status: Status::Ready,
                }) {
                    if let Some(bloodlust_command) = self.try_bloodlust_imp(state, agent_id) {
                        self.trim(&ids);
                        return Some(bloodlust_command);
                    }
                }

                if abilitys.0.contains(&RechargeableAbility {
                    ability: Ability::BombDemonic,
                    status: Status::Ready,
                }) {
                    if let Some(bomb_command) = self.try_throw_bomb(state, agent_id) {
                        self.trim(&ids);
                        return Some(bomb_command);
                    }
                }
            }

            if let Some(attack_command) = self.try_to_attack(state, agent_id) {
                self.trim(&ids);
                return Some(attack_command);
            }

            if let Some(move_command) = self.try_to_move(state, agent_id) {
                self.trim(&ids);
                return Some(move_command);
            }
        }
        Some(command::EndTurn.into())
    }
}
