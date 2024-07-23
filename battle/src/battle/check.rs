use crate::{
    battle::{
        self,
        ability::{self, Ability},
        command::{self, Command},
        state, Attacks, Id, Jokers, Moves, PushStrength, State, Weight,
    },
    map::{self, PosHex},
};

use super::BAD_ID;

pub fn check(state: &State, command: &Command) -> Result<(), Error> {
    if state.battle_result().is_some() {
        return Err(Error::BattleEnded);
    }
    match *command {
        Command::Create(ref command) => check_command_create(state, command),
        Command::MoveTo(ref command) => check_command_move_to(state, command),
        Command::Attack(ref command) => check_command_attack(state, command),
        Command::EndTurn(ref command) => check_command_end_turn(state, command),
        Command::UseAbility(ref command) => check_command_use_ability(state, command),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    NotEnoughMovePoints,
    NotEnoughStrength,
    BadActorId,
    BadTargetId,
    BadTargetType,
    TileIsBlocked,
    DistanceIsTooBig,
    DistanceIsTooSmall,
    BadDistance,
    CanNotCommandEnemyAgents,
    NotEnoughMoves,
    NotEnoughAttacks,
    AbilityIsNotReady,
    NoSuchAbility,
    NoTarget,
    BadPos,
    BadActorType,
    BattleEnded,
}

const BOMB_THROW_DISTANCE_MAX: i32 = 3;

fn check_command_move_to(state: &State, command: &command::MoveTo) -> Result<(), Error> {
    let agent = try_get_actor(state, command.id)?;
    let agent_player_id = state.belongs_to(&command.id).0;
    if agent_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyAgents);
    }

    // Check that the agent can move
    if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughMoves);
    }

    // TODO : Add this constraint
    // assert_eq!(command.path.tiles().len(), 2);

    let cost = map::distance_hex(command.path.from(), command.path.to());
    if cost > agent.move_points.0 {
        return Err(Error::NotEnoughMovePoints);
    }

    // for step in command.path.steps() {
    //     check_not_blocked_and_is_inboard(state, step.to)?;
    // }

    // let cost = command.path.cost_for(state, command.id);
    // if cost > agent.move_points {
    //     return Err(Error::NotEnoughMovePoints);
    // }
    Ok(())
}

fn check_command_create(state: &State, command: &command::Create) -> Result<(), Error> {
    check_not_blocked_and_is_inboard(state, command.pos)
}

pub fn check_command_attack(state: &State, command: &command::Attack) -> Result<(), Error> {
    if command.attacker_id == command.target_id {
        return Err(Error::BadTargetId);
    }
    let target_pos = match state.parts().pos.get(&command.target_id) {
        Some(pos) => pos.0,
        None => return Err(Error::BadTargetId),
    };
    let attacker_pos = state.pos(&command.attacker_id).0;
    let attacker_agent = state.agent(&command.attacker_id);
    check_max_distance(attacker_pos, target_pos, attacker_agent.attack_distance)?;

    let attacker_player_id = state.belongs_to(&command.attacker_id).0;
    if attacker_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyAgents);
    }

    // Check that the agent can attack
    if attacker_agent.attacks == Attacks(0) && attacker_agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughAttacks);
    }

    Ok(())
}

fn check_command_end_turn(_: &State, _: &command::EndTurn) -> Result<(), Error> {
    Ok(())
}

fn check_command_use_ability(state: &State, command: &command::UseAbility) -> Result<(), Error> {
    check_agent_belongs_to_correct_player(state, command.id)?;
    check_agent_can_attack(state, command.id)?;
    check_agent_ability_ready(state, command.id, &command.ability)?;
    match command.ability {
        Ability::Knockback => check_ability_knockback(state, command.id, command.pos),
        Ability::Club => check_ability_club(state, command.id, command.pos),
        Ability::Jump => check_ability_jump(state, command.id, command.pos, 2),
        Ability::LongJump => check_ability_jump(state, command.id, command.pos, 3),
        Ability::Poison => check_ability_poison(state, command.id, command.pos),
        Ability::Bomb
        | Ability::BombPush
        | Ability::BombFire
        | Ability::BombPoison
        | Ability::BombDemonic => check_ability_bomb_throw(state, command.id, command.pos),
        Ability::Summon => check_ability_summon(state, command.id, command.pos),
        Ability::Vanish => check_ability_vanish(state, command.id, command.pos),
        Ability::Dash => check_ability_dash(state, command.id, command.pos),
        Ability::Rage => check_ability_rage(state, command.id, command.pos),
        Ability::Heal | Ability::GreatHeal => check_ability_heal(state, command.id, command.pos),
        Ability::Bloodlust => check_ability_bloodlust(state, command.id, command.pos),
        Ability::ExplodePush
        | Ability::ExplodeDamage
        | Ability::ExplodeFire
        | Ability::ExplodePoison => check_ability_explode(state, command.id, command.pos),
    }
}

fn check_ability_knockback(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let strength = PushStrength(Weight::Normal);
    let selected_pos = state.pos(&id).0;
    check_min_distance(selected_pos, pos, 1)?;
    check_max_distance(selected_pos, pos, 1)?;
    let target_id = match state::blocker_id_at_opt(state, pos) {
        Some(id) => id,
        None => return Err(Error::NoTarget),
    };
    let target_weight = state.blocker(&target_id).weight;
    if !strength.can_push(target_weight) {
        return Err(Error::NotEnoughStrength);
    }
    Ok(())
}

fn check_ability_jump(state: &State, id: Id, pos: PosHex, max_distance: i32) -> Result<(), Error> {
    let agent_pos = state.pos(&id).0;
    let dist = map::distance_hex(agent_pos, pos);
    if dist < 2 {
        return Err(Error::DistanceIsTooSmall);
    }
    if dist > max_distance {
        return Err(Error::DistanceIsTooBig);
    }
    check_not_blocked_and_is_inboard(state, pos)?;
    Ok(())
}

fn check_ability_club(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let selected_pos = state.pos(&id).0;
    if map::distance_hex(selected_pos, pos) != 1 {
        return Err(Error::BadDistance);
    }
    if state::get_id_by_pos(state, pos).0 == BAD_ID {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn check_ability_poison(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let selected_pos = state.pos(&id).0;
    let dist = map::distance_hex(selected_pos, pos);
    if dist < 1 {
        return Err(Error::DistanceIsTooSmall);
    }
    if dist > 3 {
        return Err(Error::DistanceIsTooBig);
    }
    if state::blocker_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn check_ability_explode(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    check_object_pos(state, id, pos)
}

fn check_ability_bomb_throw(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let agent_pos = state.pos(&id).0;
    check_max_distance(agent_pos, pos, BOMB_THROW_DISTANCE_MAX)?;
    check_not_blocked_and_is_inboard(state, pos)
}

fn check_ability_summon(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    check_object_pos(state, id, pos)
}

fn check_ability_vanish(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    // TODO : Check agent 'is_some' ?
    if state.parts().agent.get(&id).is_some() {
        return Err(Error::BadActorType);
    }
    let actor_pos = match state.parts().pos.get(&id) {
        Some(pos) => pos.0,
        None => return Err(Error::BadActorType),
    };
    if pos != actor_pos {
        return Err(Error::BadPos);
    }
    Ok(())
}

fn check_ability_dash(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let agent_pos = state.pos(&id).0;
    check_max_distance(agent_pos, pos, 1)?;
    check_not_blocked_and_is_inboard(state, pos)
}

fn check_ability_rage(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    check_object_pos(state, id, pos)
}

fn check_ability_heal(state: &State, id: Id, pos: PosHex) -> Result<(), Error> {
    let agent_pos = state.pos(&id).0;
    check_max_distance(agent_pos, pos, 1)?;
    if state::agent_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

fn check_ability_bloodlust(state: &State, _id: Id, pos: PosHex) -> Result<(), Error> {
    // TODO: check that the target belongs to the same player
    if state::agent_id_at_opt(state, pos).is_none() {
        return Err(Error::NoTarget);
    }
    Ok(())
}

pub fn try_get_actor(state: &State, id: Id) -> Result<&battle::component::Agent, Error> {
    match state.parts().agent.get(&id) {
        Some(agent) => Ok(agent),
        None => Err(Error::BadActorId),
    }
}

fn check_agent_ability_ready(
    state: &State,
    id: Id,
    expected_ability: &Ability,
) -> Result<(), Error> {
    let abilities = &state.abilities(&id).0;

    for ability in abilities {
        if ability.ability == *expected_ability {
            if ability.status != ability::Status::Ready {
                return Err(Error::AbilityIsNotReady);
            }

            return Ok(());
        }
    }

    Err(Error::NoSuchAbility)
}

fn check_agent_belongs_to_correct_player(state: &State, id: Id) -> Result<(), Error> {
    let agent_player_id = state.belongs_to(&id).0;
    if agent_player_id != state.player_id() {
        return Err(Error::CanNotCommandEnemyAgents);
    }
    Ok(())
}

pub fn check_agent_can_attack(state: &State, id: Id) -> Result<(), Error> {
    let agent = try_get_actor(state, id)?;
    if agent.attacks == Attacks(0) && agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughAttacks);
    }
    Ok(())
}

pub fn check_agent_can_move(state: &State, id: Id) -> Result<(), Error> {
    let agent = try_get_actor(state, id)?;
    if agent.moves == Moves(0) && agent.jokers == Jokers(0) {
        return Err(Error::NotEnoughMoves);
    }
    Ok(())
}

fn check_min_distance(from: PosHex, to: PosHex, min: i32) -> Result<(), Error> {
    let dist = map::distance_hex(from, to);
    if dist < min {
        return Err(Error::DistanceIsTooSmall);
    }
    Ok(())
}

pub fn check_max_distance(from: PosHex, to: PosHex, max: i32) -> Result<(), Error> {
    let dist = map::distance_hex(from, to);
    if dist > max {
        return Err(Error::DistanceIsTooBig);
    }
    Ok(())
}

fn check_not_blocked_and_is_inboard(state: &State, pos: PosHex) -> Result<(), Error> {
    check_is_inboard(state, pos)?;
    check_is_tile_blocked(state, pos)
}

fn check_is_inboard(state: &State, pos: PosHex) -> Result<(), Error> {
    if !state.map().is_inboard(pos) {
        return Err(Error::BadPos);
    }
    Ok(())
}

fn check_is_tile_blocked(state: &State, pos: PosHex) -> Result<(), Error> {
    for p in state.parts().pos.values() {
        if p.0 == pos {
            return Err(Error::TileIsBlocked);
        }
    }
    Ok(())
}

fn check_object_pos(state: &State, id: Id, expected_pos: PosHex) -> Result<(), Error> {
    let real_pos = state.pos(&id).0;
    if real_pos != expected_pos {
        return Err(Error::BadPos);
    }
    Ok(())
}
