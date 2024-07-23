use crate::{
    battle::{
        self,
        ability::{Ability, PassiveAbility},
        check::{check, Error},
        command::{self, Command},
        component::{self, ObjType},
        effect::{self, Effect},
        event::{self, ActiveEvent, Event},
        state::{self, BattleResult, State},
        Id, Phase, PlayerId, PushStrength, Rounds, Strength, Weight,
    },
    game::Level,
    map::{self, Dir, PosHex},
    utils::{self, SimpleRng},
};

#[cfg(feature = "event")]
use crate::battle::movement::Path;
#[cfg(feature = "event")]
use crate::battle::Moves;

use super::state::{
    apply::{
        remove_outdated_lasting_effect, remove_outdated_planned_abilities, reset_status,
        tick_planned_abilities, update_cooldowns, update_reattack, update_reattack_with_stun,
    },
    get_id_by_pos,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ApplyPhase {
    Pre,
    Post,
}

/// A callback for visualization of the events/effects with the correct state.
pub type Cb<'c> = &'c mut dyn FnMut(&State, &Event, ApplyPhase);

pub fn execute(
    state: &mut State,
    command: &Command,
    rng: &mut SimpleRng,
    #[cfg(feature = "event")] cb: Cb,
) -> Result<(), Error> {
    if state.player_id().0 == 0 {
        check(state, command)?
    }

    #[cfg(feature = "debug")]
    add_debug_commands(state, command);

    match command {
        Command::Create(command) => execute_create(
            state,
            #[cfg(feature = "event")]
            cb,
            command,
        ),
        Command::MoveTo(command) => execute_move_to(
            state,
            #[cfg(feature = "event")]
            cb,
            command,
            rng,
        ),
        Command::Attack(command) => execute_attack(
            state,
            #[cfg(feature = "event")]
            cb,
            command,
            rng,
        ),
        Command::EndTurn(command) => execute_end_turn(
            state,
            #[cfg(feature = "event")]
            cb,
            command,
        ),
        Command::UseAbility(command) => execute_use_ability(
            state,
            #[cfg(feature = "event")]
            cb,
            command,
            rng,
        ),
    }

    execute_planned_abilities(
        state,
        rng,
        #[cfg(feature = "event")]
        cb,
    );

    match command {
        Command::Create(_) => {}
        _ => {
            for i in 0..state.players_count() {
                let player_id = PlayerId(i);
                if state::enemy_agent_count(state, player_id) == 0 {
                    let result = BattleResult {
                        winner_id: player_id,
                        survivor_types: state::players_agent_types(state, PlayerId(0)),
                        level: state.level().clone(),
                    };

                    #[cfg(not(feature = "event"))]
                    state.set_battle_result(result);

                    #[cfg(feature = "event")]
                    {
                        let event = Event {
                            active_event: event::EndBattle { result }.into(),
                            actor_ids: Vec::new(),
                            instant_effects: Vec::new(),
                            timed_effects: Vec::new(),
                            scheduled_abilities: Vec::new(),
                        };

                        do_event(
                            state,
                            #[cfg(feature = "event")]
                            cb,
                            &event,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn do_event(state: &mut State, #[cfg(feature = "event")] cb: Cb, event: &Event) {
    #[cfg(feature = "event")]
    cb(state, event, ApplyPhase::Pre);
    state.apply(event);
    #[cfg(feature = "event")]
    cb(state, event, ApplyPhase::Post);
}

#[cfg(feature = "debug")]
fn add_debug_commands(state: &mut State, command: &Command) {
    if state.player_id().0 != 0 {
        return;
    }

    match command.clone() {
        Command::MoveTo(command) => {
            if state.commands.get(state.turn_id).is_none() {
                state.commands.push(vec![]);
            }
            state.commands[state.turn_id].push(command.into())
        }
        Command::Attack(command) => {
            if state.commands.get(state.turn_id).is_none() {
                state.commands.push(vec![]);
            }
            state.commands[state.turn_id].push(command.into())
        }
        Command::EndTurn(_) => {
            if state.commands.get(state.turn_id).is_none() {
                state.commands.push(vec![]);
            }
            state.turn_id += 1;
        }
        Command::UseAbility(command) => {
            if state.commands.get(state.turn_id).is_none() {
                state.commands.push(vec![]);
            }
            state.commands[state.turn_id].push(command.into())
        }
        _ => {}
    }
}

fn execute_move_to(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    command: &command::MoveTo,
    _rng: &mut SimpleRng,
) {
    let id = command.id;

    // let mut cost = Some(Moves(1));
    // for step in command.path.steps() {
    //     let path = Path::new(vec![step.from, step.to]);
    //     do_move(
    //         state,
    //         #[cfg(feature = "event")]
    //         cb,
    //         id,
    //         cost.take(),
    //         path,
    //     );

    //     try_execute_passive_abilities_on_move(
    //         state,
    //         #[cfg(feature = "event")]
    //         cb,
    //         id,
    //     );

    //     let attack_status = try_execute_reaction_attacks(
    //         state,
    //         #[cfg(feature = "event")]
    //         cb,
    //         id,
    //         rng,
    //     );
    //     if attack_status != AttackStatus::Miss {
    //         break;
    //     }
    // }

    #[cfg(feature = "event")]
    {
        use crate::battle::movement::Path;
        use crate::battle::Moves;
        let path = Path::new(vec![command.path.from(), command.path.to()]);
        do_move(
            state,
            #[cfg(feature = "event")]
            cb,
            id,
            Some(Moves(1)),
            path,
        );
    }

    #[cfg(not(feature = "event"))]
    {
        let parts = state.parts_mut();
        let pos = parts.pos.get_mut(&id).unwrap();
        pos.0 = command.path.to();

        let agent = parts.agent.get_mut(&id).unwrap();
        if agent.moves.0 > 0 {
            agent.moves.0 -= 1;
        } else {
            agent.jokers.0 -= 1;
        }
    }

    try_execute_passive_abilities_on_move(
        state,
        #[cfg(feature = "event")]
        cb,
        id,
    );
}

#[cfg(feature = "event")]
fn do_move(state: &mut State, cb: Cb, id: Id, cost: Option<Moves>, path: Path) {
    let cost = cost.unwrap_or(Moves(0));
    let active_event = event::MoveTo { path, cost, id }.into();
    let event = Event {
        active_event,
        actor_ids: vec![id],
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    do_event(state, cb, &event);
}

fn execute_create(state: &mut State, #[cfg(feature = "event")] cb: Cb, command: &command::Create) {
    let mut components = state.prototype_for(&command.prototype);
    if let Some(player_id) = command.owner {
        components.push(component::BelongsTo(player_id).into());
    }
    components.extend_from_slice(&[
        component::Pos(command.pos).into(),
        component::Meta {
            name: command.prototype.clone(),
        }
        .into(),
    ]);

    let id = state.alloc_id();

    #[cfg(feature = "event")]
    {
        let mut instant_effects = Vec::new();
        let effect_create = effect::Create {
            pos: command.pos,
            prototype: command.prototype.clone(),
            components,
            is_teleported: false,
        }
        .into();
        instant_effects.push((id, vec![effect_create]));

        let event = Event {
            active_event: ActiveEvent::Create,
            actor_ids: vec![id],
            instant_effects,
            timed_effects: Vec::new(),
            scheduled_abilities: Vec::new(),
        };
        do_event(state, cb, &event);
    }

    #[cfg(not(feature = "event"))]
    {
        use battle::state::apply::add_components;
        add_components(state, &id, &components);
    }
}

#[derive(PartialEq, Clone, Debug)]
enum AttackStatus {
    Hit,
    Miss,
    Kill,
}

fn execute_attack_internal(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    command: &command::Attack,
    #[cfg(feature = "event")] mode: event::AttackMode,
    rng: &mut SimpleRng,
) -> AttackStatus {
    let attacker_id = command.attacker_id;
    let target_id = command.target_id;

    #[cfg(feature = "event")]
    {
        let weapon_type = state.agent(&attacker_id).weapon_type;
        let event_attack = event::Attack {
            attacker_id,
            target_id,
            mode,
            weapon_type,
        };
        let mut context = ExecuteContext::default();

        let status = if let Some(effect) = try_attack(state, attacker_id, target_id, rng) {
            if let Effect::Kill(_) = effect {
                context.instant_effects.push((target_id, vec![effect]));
                AttackStatus::Kill
            } else {
                context.instant_effects.push((target_id, vec![effect]));

                let c = try_execute_passive_abilities_on_attack(state, attacker_id, target_id);
                context.merge_with(c);

                AttackStatus::Hit
            }
        } else {
            let attacker_pos = state.pos(&attacker_id).0;
            let dodge = effect::Dodge { attacker_pos }.into();
            context.instant_effects.push((target_id, vec![dodge]));

            AttackStatus::Miss
        };

        let event = Event {
            active_event: event_attack.into(),
            actor_ids: vec![attacker_id],
            instant_effects: context.instant_effects,
            timed_effects: context.timed_effects,
            scheduled_abilities: context.scheduled_abilities,
        };
        do_event(state, cb, &event);

        // for id in context.moved_actor_ids {
        //     try_execute_passive_abilities_on_move(
        //         state,
        //         #[cfg(feature = "event")]
        //         cb,
        //         id,
        //     );
        // }

        return status;
    }

    #[cfg(not(feature = "event"))]
    {
        use crate::battle::state::apply::{
            apply_effect_timed, apply_effect_wound, apply_event_attack,
        };
        let attack_effect = try_attack(state, attacker_id, target_id, rng);
        apply_event_attack(state, &attacker_id);

        let mut status = AttackStatus::Miss;

        if let Some(e) = attack_effect {
            match e {
                Effect::Kill(_) => {
                    state.parts_mut().remove(&target_id);
                    status = AttackStatus::Kill;
                }
                Effect::Wound(w) => {
                    apply_effect_wound(state, &target_id, w.damage.0);

                    if let Some(passive_abilities) =
                        state.parts().passive_abilities.get(&attacker_id).cloned()
                    {
                        for ability in &passive_abilities.0 {
                            if let PassiveAbility::PoisonAttack = ability {
                                let owner = state.belongs_to(&target_id).0;
                                let effect = effect::Timed {
                                    duration: effect::Duration::Rounds(2.into()),
                                    phase: Phase::from(owner.0),
                                    effect: effect::Lasting::Poison,
                                };

                                apply_effect_timed(state, &target_id, &effect)
                            }
                        }
                    }

                    status = AttackStatus::Hit;
                }
                _ => unreachable!(),
            }
        }

        return status;
    }
}

fn try_execute_passive_ability_burn(state: &mut State, target_id: Id) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let target_effects = vec![wound_or_kill(state, &target_id, Strength(1))];
    context.instant_effects.push((target_id, target_effects));
    context
}

fn try_execute_passive_ability_spike_trap(state: &mut State, target_id: Id) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let target_effects = vec![wound_or_kill(state, &target_id, Strength(1))];
    context.instant_effects.push((target_id, target_effects));
    context
}

fn try_execute_passive_ability_poison(state: &State, target_id: Id) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    if state.strength(&target_id).strength.0 <= 1 {
        return context;
    }
    let owner = state.belongs_to(&target_id).0;
    let effect = effect::Timed {
        duration: effect::Duration::Rounds(2.into()),
        phase: Phase::from(owner.0),
        effect: effect::Lasting::Poison,
    };
    context.timed_effects.push((target_id, vec![effect]));
    context
}

fn do_passive_ability(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    id: Id,
    target_pos: PosHex,
    ability: PassiveAbility,
    context: ExecuteContext,
) {
    let active_event = event::UsePassiveAbility {
        pos: target_pos,
        id,
        ability,
    }
    .into();
    let event = Event {
        active_event,
        actor_ids: context.actor_ids,
        instant_effects: context.instant_effects,
        timed_effects: context.timed_effects,
        scheduled_abilities: Vec::new(),
    };
    do_event(
        state,
        #[cfg(feature = "event")]
        cb,
        &event,
    );
}

fn try_execute_passive_abilities_on_move(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    target_id: Id,
) {
    try_execute_passive_abilities_tick(
        state,
        #[cfg(feature = "event")]
        cb,
        target_id,
    )
}

fn try_execute_passive_abilities_tick(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    target_id: Id,
) {
    if !state.parts().is_exist(&target_id) {
        return;
    }
    let target_pos = state.pos(&target_id).0;
    for (id, abilities) in state.parts().passive_abilities.clone().into_iter() {
        if state.pos(&id).0 != target_pos {
            continue;
        }

        for ability in abilities.0 {
            match ability {
                PassiveAbility::SpikeTrap => {
                    let context = try_execute_passive_ability_spike_trap(state, target_id);
                    do_passive_ability(
                        state,
                        #[cfg(feature = "event")]
                        cb,
                        id,
                        target_pos,
                        ability,
                        context,
                    );
                }
                PassiveAbility::Burn => {
                    let context = try_execute_passive_ability_burn(state, target_id);
                    do_passive_ability(
                        state,
                        #[cfg(feature = "event")]
                        cb,
                        id,
                        target_pos,
                        ability,
                        context,
                    );
                }
                PassiveAbility::Poison => {
                    let context = try_execute_passive_ability_poison(state, target_id);
                    if !context.timed_effects.is_empty() {
                        do_passive_ability(
                            state,
                            #[cfg(feature = "event")]
                            cb,
                            id,
                            target_pos,
                            ability,
                            context,
                        );
                    }
                }
                PassiveAbility::HeavyImpact
                | PassiveAbility::PoisonAttack
                | PassiveAbility::Regenerate
                | PassiveAbility::SpawnPoisonCloudOnDeath => {}
            }
        }
    }
}

pub fn try_execute_passive_abilities_on_begin_turn(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
) {
    for id in state::players_agent_ids(state, state.player_id()) {
        try_execute_passive_abilities_tick(
            state,
            #[cfg(feature = "event")]
            cb,
            id,
        );
    }

    // TODO: extract to some self-abilities-method?
    {
        for (id, abilities) in state.parts().passive_abilities.clone().into_iter() {
            assert!(state.parts().is_exist(&id));
            let owner = match state.parts().belongs_to.get(&id) {
                Some(owner) => owner.0,
                None => continue,
            };
            if state.player_id() != owner {
                continue;
            }

            for ability in abilities.0 {
                assert!(state.parts().is_exist(&id));
                if let PassiveAbility::Regenerate = ability {
                    let strength = state.strength(&id);
                    if strength.strength >= strength.base_strength {
                        continue;
                    }
                    let pos = state.pos(&id).0;
                    let active_event = event::UsePassiveAbility {
                        id: id,
                        pos,
                        ability,
                    }
                    .into();
                    let mut target_effects = Vec::new();
                    target_effects.push(
                        effect::Heal {
                            strength: Strength(1),
                        }
                        .into(),
                    );
                    let instant_effects = vec![(id, target_effects)];
                    let event = Event {
                        active_event,
                        actor_ids: vec![id],
                        instant_effects,
                        timed_effects: Vec::new(),
                        scheduled_abilities: Vec::new(),
                    };
                    do_event(
                        state,
                        #[cfg(feature = "event")]
                        cb,
                        &event,
                    );
                }
            }
        }
    }
}

#[allow(dead_code)]
fn try_execute_passive_abilities_on_attack(
    state: &mut State,
    attacker_id: Id,
    target_id: Id,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    if let Some(passive_abilities) = state.parts().passive_abilities.get(&attacker_id) {
        for ability in &passive_abilities.0 {
            match ability {
                PassiveAbility::PoisonAttack => {
                    let owner = state.belongs_to(&target_id).0;
                    let effect = effect::Timed {
                        duration: effect::Duration::Rounds(2.into()),
                        phase: Phase::from(owner.0),
                        effect: effect::Lasting::Poison,
                    };
                    context.timed_effects.push((target_id, vec![effect]));
                }
                PassiveAbility::Burn
                | PassiveAbility::SpikeTrap
                | PassiveAbility::Poison
                | PassiveAbility::Regenerate
                | PassiveAbility::HeavyImpact
                | PassiveAbility::SpawnPoisonCloudOnDeath => (),
            }
        }
    }
    context
}

fn try_execute_reaction_attacks(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    target_id: Id,
    rng: &mut SimpleRng,
) -> AttackStatus {
    let mut status = AttackStatus::Miss;
    let target_owner = match state.parts().belongs_to.get(&target_id) {
        Some(belongs_to) => belongs_to.0,
        None => return status,
    };

    for obj_id in state::enemy_agent_ids(state, target_owner) {
        match state.parts().agent.get(&obj_id) {
            Some(a) => {
                if a.attacks.0 == 0 && a.jokers.0 == 0 {
                    continue;
                }

                let from = state.pos(&obj_id).0;
                let to = state.pos(&target_id).0;
                let dist = map::distance_hex(from, to);
                if dist > a.attack_distance {
                    continue;
                }
            }
            None => continue,
        }

        let this_agent_owner = state.belongs_to(&obj_id).0;
        state.set_player_id(this_agent_owner);
        let command_attack = command::Attack {
            attacker_id: obj_id,
            target_id,
        };

        let this_attack_status = execute_attack_internal(
            state,
            #[cfg(feature = "event")]
            cb,
            &command_attack,
            #[cfg(feature = "event")]
            event::AttackMode::Reactive,
            rng,
        );

        if this_attack_status != AttackStatus::Miss {
            status = this_attack_status;
        }

        if status == AttackStatus::Kill {
            break;
        }
    }
    state.set_player_id(target_owner);

    status
}

fn execute_attack(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    command: &command::Attack,
    rng: &mut SimpleRng,
) {
    execute_attack_internal(
        state,
        #[cfg(feature = "event")]
        cb,
        command,
        #[cfg(feature = "event")]
        event::AttackMode::Active,
        rng,
    );

    try_execute_reaction_attacks(
        state,
        #[cfg(feature = "event")]
        cb,
        command.attacker_id,
        rng,
    );
}

pub fn execute_event_end_turn(state: &mut State, #[cfg(feature = "event")] cb: Cb) {
    let player_id_old = state.player_id();
    let active_event = event::EndTurn {
        player_id: player_id_old,
    }
    .into();
    let actor_ids = state::players_agent_ids(state, player_id_old);
    let event = Event {
        active_event,
        actor_ids,
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    do_event(
        state,
        #[cfg(feature = "event")]
        cb,
        &event,
    );
}

pub fn execute_event_begin_turn(state: &mut State, #[cfg(feature = "event")] cb: Cb) {
    let player_id_new = state.next_player_id();
    let active_event = event::BeginTurn {
        player_id: player_id_new,
    }
    .into();
    let mut actor_ids = state::players_agent_ids(state, player_id_new);
    actor_ids.sort();
    let event = Event {
        active_event,
        actor_ids,
        instant_effects: Vec::new(),
        timed_effects: Vec::new(),
        scheduled_abilities: Vec::new(),
    };
    do_event(
        state,
        #[cfg(feature = "event")]
        cb,
        &event,
    );
}

fn execute_planned_abilities(
    state: &mut State,
    rng: &mut SimpleRng,
    #[cfg(feature = "event")] cb: Cb,
) {
    let mut ids: Vec<Id> = state.parts().schedule.keys().cloned().collect();
    ids.sort();
    for obj_id in ids {
        let pos = state.parts().pos.get(&obj_id).unwrap().0;
        let mut activated = Vec::new();
        {
            let schedule = state.parts().schedule.get(&obj_id).unwrap();
            for planned in &schedule.planned {
                if planned.rounds.0 <= 0 {
                    let c = command::UseAbility {
                        ability: planned.ability,
                        id: obj_id,
                        pos,
                    };
                    activated.push(c);
                }
            }
        }
        for command in activated {
            if state.parts().is_exist(&obj_id) {
                execute_use_ability(
                    state,
                    #[cfg(feature = "event")]
                    cb,
                    &command,
                    rng,
                );
            }
        }
    }
}

// TODO: simplify
/// Ticks and kills all the lasting effects.
pub fn execute_effects(state: &mut State, #[cfg(feature = "event")] cb: Cb) {
    let phase = state.player_id().0;

    for (id, effect) in state.parts().effects.clone().iter() {
        for effect in &effect.0 {
            if effect.phase.0 != phase {
                continue;
            }

            {
                let active_event = event::EffectTick {
                    id: *id,
                    effect: effect.effect,
                };
                let mut target_effects = Vec::new();
                match effect.effect {
                    effect::Lasting::Poison => {
                        let strength = state.strength(&id).strength;
                        if strength > battle::Strength(1) {
                            target_effects.push(wound_or_kill(state, id, Strength(1)));
                        }
                    }
                    effect::Lasting::Bloodlust => target_effects.push(Effect::Bloodlust),
                    effect::Lasting::Stun => {}
                }
                let instant_effects = vec![(*id, target_effects)];
                let event = Event {
                    active_event: ActiveEvent::EffectTick(active_event),
                    actor_ids: vec![*id],
                    instant_effects,
                    timed_effects: Vec::new(),
                    scheduled_abilities: Vec::new(),
                };
                do_event(
                    state,
                    #[cfg(feature = "event")]
                    cb,
                    &event,
                );
            }

            if !state.parts().is_exist(id) {
                break;
            }

            #[cfg(feature = "event")]
            if state::is_lasting_effect_over(state, id, &effect) {
                let active_event = event::EffectEnd {
                    id: *id,
                    effect: effect.effect,
                };
                let event = Event {
                    active_event: ActiveEvent::EffectEnd(active_event),
                    actor_ids: vec![*id],
                    instant_effects: Vec::new(),
                    timed_effects: Vec::new(),
                    scheduled_abilities: Vec::new(),
                };
                do_event(
                    state,
                    #[cfg(feature = "event")]
                    cb,
                    &event,
                );
            }
        }
    }
}

fn execute_end_turn(state: &mut State, #[cfg(feature = "event")] cb: Cb, _: &command::EndTurn) {
    let level = state.level();
    match level {
        Level::Level0 | Level::Level1 => {
            let ids = state::players_agent_ids(state, state.player_id());
            update_reattack(state, &ids);

            let player_id_new = state.next_player_id();
            let ids = state::players_agent_ids(state, player_id_new);

            state.set_player_id(player_id_new);
            reset_status(state, &ids);
            update_cooldowns(state, player_id_new);
        }

        Level::Level2 => {
            let ids = state::players_agent_ids(state, state.player_id());
            update_reattack(state, &ids);
            remove_outdated_planned_abilities(state);

            let player_id_new = state.next_player_id();
            let ids = state::players_agent_ids(state, player_id_new);
            state.set_player_id(player_id_new);
            reset_status(state, &ids);
            update_cooldowns(state, player_id_new);
            tick_planned_abilities(state);
        }

        Level::Level3 | Level::Level4 | Level::Level5 => {
            let ids = state::players_agent_ids(state, state.player_id());
            update_reattack_with_stun(state, &ids);
            remove_outdated_planned_abilities(state);
            remove_outdated_lasting_effect(state);

            let player_id_new = state.next_player_id();
            let ids = state::players_agent_ids(state, player_id_new);
            state.set_player_id(player_id_new);
            reset_status(state, &ids);
            update_cooldowns(state, player_id_new);
            tick_planned_abilities(state);

            if player_id_new.0 == 0 {
                for id in state::players_agent_ids(state, state.player_id()) {
                    try_execute_passive_abilities_tick(
                        state,
                        #[cfg(feature = "event")]
                        cb,
                        id,
                    );
                }

                execute_effects(
                    state,
                    #[cfg(feature = "event")]
                    cb,
                );
            }
        }
    }
}

fn start_fire(state: &mut State, pos: PosHex) -> ExecuteContext {
    let vanish = component::PlannedAbility {
        rounds: 2.into(), // TODO: Replace this magic number
        phase: Phase::from(state.player_id().0),
        ability: Ability::Vanish,
    };
    let mut context = ExecuteContext::default();
    if let Some(id) = state::obj_with_passive_ability_at(state, pos, PassiveAbility::Burn) {
        context.scheduled_abilities.push((id, vec![vanish]));
    } else {
        let effect_create = effect_create_object(state, &"fire".into(), pos);
        let id = state.alloc_id();
        context.instant_effects.push((id, vec![effect_create]));
        context.scheduled_abilities.push((id, vec![vanish]));
        for target_id in state::agent_ids_at(state, pos) {
            context.merge_with(try_execute_passive_ability_burn(state, target_id));
        }
    }
    context
}

fn create_poison_cloud(state: &mut State, pos: PosHex) -> ExecuteContext {
    let vanish = component::PlannedAbility {
        rounds: 2.into(), // TODO: Replace this magic number
        phase: Phase::from(state.player_id().0),
        ability: Ability::Vanish,
    };
    let mut context = ExecuteContext::default();
    if let Some(id) = state::obj_with_passive_ability_at(state, pos, PassiveAbility::Poison) {
        context.scheduled_abilities.push((id, vec![vanish]));
    } else {
        let effect_create = effect_create_object(state, &"poison_cloud".into(), pos);

        let id = state.alloc_id();
        context.instant_effects.push((id, vec![effect_create]));
        context.scheduled_abilities.push((id, vec![vanish]));
        for target_id in state::agent_ids_at(state, pos) {
            context.merge_with(try_execute_passive_ability_poison(state, target_id));
        }
    }
    context
}

fn extend_or_crate_sub_vec<T>(vec: &mut Vec<(Id, Vec<T>)>, id: Id, values: Vec<T>) {
    if let Some(i) = vec.iter().position(|(this_id, _)| this_id == &id) {
        vec[i].1.extend(values);
    } else {
        vec.push((id, values));
    }
}

fn any_effect_with_id(effects: &[(Id, Vec<Effect>)], expected_id: Id) -> bool {
    effects.iter().any(|(id, _)| id == &expected_id)
}

#[must_use]
#[derive(Default, Debug, PartialEq, Clone)]
struct ExecuteContext {
    actor_ids: Vec<Id>,
    moved_actor_ids: Vec<Id>,
    instant_effects: Vec<(Id, Vec<Effect>)>,
    timed_effects: Vec<(Id, Vec<effect::Timed>)>,
    scheduled_abilities: Vec<(Id, Vec<component::PlannedAbility>)>,
}

impl ExecuteContext {
    fn merge_with(&mut self, other: Self) {
        type M<T> = Vec<(Id, Vec<T>)>;

        fn merge<T>(m: &mut M<T>, other: M<T>) {
            for (id, values) in other {
                extend_or_crate_sub_vec(m, id, values);
            }
        }

        self.actor_ids.extend(other.actor_ids);
        self.moved_actor_ids.extend(other.moved_actor_ids);
        merge(&mut self.instant_effects, other.instant_effects);
        merge(&mut self.timed_effects, other.timed_effects);
        merge(&mut self.scheduled_abilities, other.scheduled_abilities);
    }
}

fn execute_use_ability_knockback(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let from = command.pos;
    let mut context = ExecuteContext::default();
    let id = get_id_by_pos(state, command.pos);
    let strength = PushStrength(Weight::Normal);
    let blocker_weight = state.blocker(&id).weight;
    let to = if strength.can_push(blocker_weight) {
        let actor_pos = state.pos(&command.id).0;
        let dir = Dir::get_dir_from_to(actor_pos, command.pos);
        Dir::get_neighbor_pos(command.pos, dir)
    } else {
        from
    };

    if to != from || !state::is_tile_blocked(state, to) {
        let effect = effect::Knockback { from, to, strength }.into();
        context.instant_effects.push((id, vec![effect]));
        context.moved_actor_ids.push(id);
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability_club(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = get_id_by_pos(state, command.pos);
    if let Some(x) = state.parts().belongs_to.get(&id) {
        let owner = x.0;
        let phase = Phase::from(owner.0);
        let effect = effect::Timed {
            duration: effect::Duration::Rounds(Rounds(1)),
            phase,
            effect: effect::Lasting::Stun,
        };
        context.timed_effects.push((id, vec![effect]));
        context.instant_effects.push((id, vec![Effect::Stun]));

        //  extend_or_crate_sub_vec(&mut context.instant_effects, id, vec![Effect::Stun]);
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability_explode_fire(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(!any_effect_with_id(&context.instant_effects, command.id));
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context.merge_with(start_fire(state, command.pos));
    for dir in map::dirs() {
        let pos = Dir::get_neighbor_pos(command.pos, dir);
        if state.map().is_inboard(pos) {
            context.merge_with(start_fire(state, pos));
        }
    }
    context
}

fn execute_use_ability_jump(_: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    context.moved_actor_ids.push(command.id);
    context
}

fn execute_use_ability_long_jump(_: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    context.moved_actor_ids.push(command.id);
    context
}

fn execute_use_ability_dash(_: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    context.moved_actor_ids.push(command.id);
    context
}

fn execute_use_ability_rage(_: &mut State, _: &command::UseAbility) -> ExecuteContext {
    ExecuteContext::default()
}

fn execute_use_ability_heal(
    state: &mut State,
    command: &command::UseAbility,
    strength: Strength,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = get_id_by_pos(state, command.pos);
    let effect = effect::Heal { strength }.into();
    context.instant_effects.push((id, vec![effect]));
    context
}

fn execute_use_ability_vanish(command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context
}

fn execute_use_ability_explode_poison(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    assert!(!any_effect_with_id(&context.instant_effects, command.id));
    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context.merge_with(create_poison_cloud(state, command.pos));
    for dir in map::dirs() {
        let pos = Dir::get_neighbor_pos(command.pos, dir);
        if state.map().is_inboard(pos) {
            context.merge_with(create_poison_cloud(state, pos));
        }
    }
    context
}

fn wound_or_kill(state: &State, id: &Id, damage: battle::Strength) -> Effect {
    if state.strength(&id).strength > damage {
        effect::Wound {
            damage,
            attacker_pos: None,
        }
        .into()
    } else {
        effect::Kill { attacker_pos: None }.into()
    }
}

fn try_attack(
    state: &State,
    attacker_id: Id,
    target_id: Id,
    rng: &mut SimpleRng,
) -> Option<Effect> {
    let agent_attacker = state.agent(&attacker_id);
    let attacker_strength = state.strength(&attacker_id);
    let attacker_pos = Some(state.pos(&attacker_id).0);
    let attacker_wounds = utils::clamp_max(
        attacker_strength.base_strength.0 - attacker_strength.strength.0,
        2,
    );

    // let agent_target = state.agent(&target_id);
    let target_strength = state.strength(&target_id).strength;

    let k = agent_attacker.attack_accuracy.0 - attacker_wounds;

    let r = rng.gen_range(0, k + if target_strength.0 > 2 { 1 } else { 2 });

    // println!(
    //     "k:{},attacker_wounds:{},r:{},l:{}",
    //     k,
    //     attacker_wounds,
    //     r,
    //     k + if target_strength.0 > 2 { 1 } else { 2 }
    // );

    let damage_raw = k - r;
    if damage_raw <= 0 {
        // That was a total miss
        return None;
    }

    let damage = Strength(utils::clamp_max(
        damage_raw,
        agent_attacker.attack_strength.0,
    ));
    let effect = if target_strength > damage {
        effect::Wound {
            damage,
            attacker_pos,
        }
        .into()
    } else {
        effect::Kill { attacker_pos }.into()
    };
    Some(effect)
}

fn execute_use_ability_explode_damage(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let from = state.pos(&command.id).0;
    for id in state::players_agent_ids(state, PlayerId(0)) {
        let pos = state.pos(&id).0;
        let distance = map::distance_hex(from, pos);
        if distance > 1 {
            continue;
        }

        let effects = vec![wound_or_kill(state, &id, Strength(1))];
        context.instant_effects.push((id, effects));
    }

    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context
}

fn execute_use_ability_explode_push(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let from = command.pos;
    for (id, blocker) in state.parts().blocker.iter() {
        let pos = state.pos(&id).0;
        let distance = map::distance_hex(from, pos);
        if distance > 1 || command.id.0 == id.0 {
            continue;
        }

        let to = if PushStrength(Weight::Normal).can_push(blocker.weight) {
            Dir::get_dir_pos(from, pos)
        } else {
            continue;
        };

        if state.map().is_inboard(to) && !state::is_tile_blocked(state, to) {
            let effects = vec![effect::Knockback {
                from: pos,
                to,
                strength: PushStrength(Weight::Normal),
            }
            .into()];

            context.instant_effects.push((*id, effects));
            context.moved_actor_ids.push(*id);
        }
    }

    let effects = vec![Effect::Vanish];
    context.instant_effects.push((command.id, effects));
    context
}

fn execute_use_ability_poison(state: &mut State, command: &command::UseAbility) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = get_id_by_pos(state, command.pos);
    let owner = state.belongs_to(&id).0;
    let phase = Phase::from(owner.0);
    let effect = effect::Timed {
        duration: effect::Duration::Rounds(2.into()),
        phase,
        effect: effect::Lasting::Poison,
    };
    context.timed_effects.push((id, vec![effect]));
    context.actor_ids.push(id);
    context
}

fn effect_create_object(state: &State, prototype: &ObjType, pos: PosHex) -> Effect {
    let mut components = state.prototype_for(prototype);
    components.extend_from_slice(&[
        component::Pos(pos).into(),
        component::Meta {
            name: prototype.clone(),
        }
        .into(),
    ]);
    effect::Create {
        pos,
        prototype: prototype.clone(),
        components,
        is_teleported: false,
    }
    .into()
}

fn effect_create_agent(
    state: &State,
    prototype: &ObjType,
    player_id: PlayerId,
    pos: PosHex,
) -> Effect {
    let name = prototype.clone();
    let mut components = state.prototype_for(prototype);
    components.extend_from_slice(&[
        component::Pos(pos).into(),
        component::Meta { name }.into(),
        component::BelongsTo(player_id).into(),
    ]);
    effect::Create {
        pos,
        prototype: prototype.clone(),
        components,
        is_teleported: true,
    }
    .into()
}

fn throw_bomb(
    state: &mut State,
    command: &command::UseAbility,
    prototype: &ObjType,
    rounds: Rounds,
    ability: Ability,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let effect_create = effect_create_object(state, prototype, command.pos);
    let id = state.alloc_id();
    let effects = vec![effect_create];
    context.instant_effects.push((id, effects));

    let planned_ability = component::PlannedAbility {
        rounds,
        phase: Phase::from(state.player_id().0),
        ability,
    };
    context
        .scheduled_abilities
        .push((id, vec![planned_ability]));
    context
}

fn execute_use_ability_bomb_push(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let ability = Ability::ExplodePush;
    throw_bomb(state, command, &"bomb_push".into(), Rounds(0), ability)
}

fn execute_use_ability_bomb_damage(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let ability = Ability::ExplodeDamage;
    throw_bomb(state, command, &"bomb_damage".into(), Rounds(1), ability)
}

fn execute_use_ability_bomb_fire(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let ability = Ability::ExplodeFire;
    throw_bomb(state, command, &"bomb_fire".into(), Rounds(1), ability)
}

fn execute_use_ability_bomb_poison(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let ability = Ability::ExplodePoison;
    throw_bomb(state, command, &"bomb_poison".into(), Rounds(1), ability)
}

fn execute_use_ability_bomb_demonic(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let ability = Ability::ExplodeDamage;
    throw_bomb(state, command, &"bomb_demonic".into(), Rounds(1), ability)
}

fn execute_use_ability_summon(
    state: &mut State,
    command: &command::UseAbility,
    rng: &mut SimpleRng,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let max_summoned_count = state.parts().summoner.get(&command.id).unwrap().count;
    let available_typenames: [ObjType; 3] = ["imp".into(), "toxic_imp".into(), "imp_bomber".into()];

    let mut new_agents = Vec::new();
    for pos in state::free_neighbor_positions(state, command.pos, max_summoned_count as usize) {
        let i = rng.gen_range(0, 3) as usize;
        let prototype = available_typenames[i].clone();

        let effect_create = effect_create_agent(state, &prototype, state.player_id(), pos);
        let id = state.alloc_id();
        let effects = vec![effect_create, Effect::Stun];
        new_agents.push(prototype);
        context.instant_effects.push((id, effects));
        context.moved_actor_ids.push(id);

        // todo if only one, should check max_summoned_count
        // only one
        return context;
    }

    context
}

fn execute_use_ability_bloodlust(
    state: &mut State,
    command: &command::UseAbility,
) -> ExecuteContext {
    let mut context = ExecuteContext::default();
    let id = get_id_by_pos(state, command.pos);
    if let Some(x) = state.parts().belongs_to.get(&id) {
        let phase = Phase::from(x.0 .0);
        let effect = effect::Timed {
            duration: effect::Duration::Rounds(Rounds(3)),
            phase,
            effect: effect::Lasting::Bloodlust,
        };
        context.timed_effects.push((id, vec![effect]));
    }
    context.actor_ids.push(id);
    context
}

fn execute_use_ability(
    state: &mut State,
    #[cfg(feature = "event")] cb: Cb,
    command: &command::UseAbility,
    rng: &mut SimpleRng,
) {
    let mut context = match command.ability {
        Ability::Knockback => execute_use_ability_knockback(state, command),
        Ability::Club => execute_use_ability_club(state, command),
        Ability::Jump => execute_use_ability_jump(state, command),
        Ability::LongJump => execute_use_ability_long_jump(state, command),
        Ability::Dash => execute_use_ability_dash(state, command),
        Ability::Rage => execute_use_ability_rage(state, command),
        Ability::Heal => execute_use_ability_heal(state, command, Strength(2)),
        Ability::GreatHeal => execute_use_ability_heal(state, command, Strength(3)),
        Ability::Vanish => execute_use_ability_vanish(command),
        Ability::ExplodeFire => execute_use_ability_explode_fire(state, command),
        Ability::ExplodePoison => execute_use_ability_explode_poison(state, command),
        Ability::ExplodePush => execute_use_ability_explode_push(state, command),
        Ability::ExplodeDamage => execute_use_ability_explode_damage(state, command),
        Ability::Poison => execute_use_ability_poison(state, command),
        Ability::Bomb => execute_use_ability_bomb_damage(state, command),
        Ability::BombPush => execute_use_ability_bomb_push(state, command),
        Ability::BombFire => execute_use_ability_bomb_fire(state, command),
        Ability::BombPoison => execute_use_ability_bomb_poison(state, command),
        Ability::BombDemonic => execute_use_ability_bomb_demonic(state, command),
        Ability::Summon => execute_use_ability_summon(state, command, rng),
        Ability::Bloodlust => execute_use_ability_bloodlust(state, command),
    };
    context.actor_ids.push(command.id);
    let active_event = event::UseAbility {
        id: command.id,
        pos: command.pos,
        ability: command.ability,
    }
    .into();
    let event = Event {
        active_event,
        actor_ids: context.actor_ids,
        instant_effects: context.instant_effects,
        timed_effects: context.timed_effects,
        scheduled_abilities: context.scheduled_abilities,
    };
    do_event(
        state,
        #[cfg(feature = "event")]
        cb,
        &event,
    );

    for id in context.moved_actor_ids {
        try_execute_passive_abilities_on_move(
            state,
            #[cfg(feature = "event")]
            cb,
            id,
        );
    }

    // if command.ability != Ability::Dash {
    //     try_execute_reaction_attacks(state, cb, command.id);
    // }
}

pub fn hit_chance(state: &State, attacker_id: Id, _target_id: Id) -> (i32, i32) {
    let agent_attacker = state.agent(&attacker_id);
    let attack_strength = state.strength(&attacker_id);

    let attacker_wounds = utils::clamp_max(
        attack_strength.base_strength.0 - attack_strength.strength.0,
        3,
    );

    let k_min = agent_attacker.attack_accuracy.0 - attacker_wounds;
    let k_max = k_min + agent_attacker.attack_strength.0;
    (k_min, k_max)
}

#[cfg(test)]
mod tests {
    use crate::{
        battle::{
            effect::{self, Effect},
            Id,
        },
        map::PosHex,
    };

    use super::ExecuteContext;

    // TODO: Don't create Id's manually? Use a mocked State instead.

    #[test]
    fn test_merge_with_vector() {
        let mut context1 = ExecuteContext {
            actor_ids: vec![Id(0), Id(1)],
            ..Default::default()
        };
        let context2 = ExecuteContext {
            actor_ids: vec![Id(2), Id(3)],
            ..Default::default()
        };
        let context_expected = ExecuteContext {
            actor_ids: vec![Id(0), Id(1), Id(2), Id(3)],
            ..Default::default()
        };
        context1.merge_with(context2);
        assert_eq!(context_expected, context1);
    }

    #[test]
    fn test_merge_with_hashmap() {
        let mut instant_effects1 = Vec::new();
        let attacker_pos = PosHex { q: 0, r: 0 };
        let effect_kill: Effect = effect::Kill {
            attacker_pos: Some(attacker_pos),
        }
        .into();
        instant_effects1.push((Id(0), vec![effect_kill.clone(), Effect::Stun]));
        let mut context1 = ExecuteContext {
            instant_effects: instant_effects1,
            ..Default::default()
        };
        let effect_dodge = effect::Dodge { attacker_pos };
        let instant_effects2 = vec![(Id(0), vec![Effect::Vanish, effect_dodge.clone().into()])];
        let context2 = ExecuteContext {
            instant_effects: instant_effects2,
            ..Default::default()
        };
        let instant_effects_expected = vec![(
            Id(0),
            vec![
                effect_kill,
                Effect::Stun,
                Effect::Vanish,
                effect_dodge.into(),
            ],
        )];
        let context_expected = ExecuteContext {
            instant_effects: instant_effects_expected,
            ..Default::default()
        };
        context1.merge_with(context2);
        assert_eq!(context_expected, context1);
    }
}
