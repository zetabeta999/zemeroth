use crate::battle::{
    ability::{self, Ability},
    component::{self, Component, Parts, PlannedAbility},
    effect::{self, Duration, Effect},
    event::{self, ActiveEvent, Event},
    state, Id, Phase, PlayerId, State,
};

pub fn apply(state: &mut State, event: &Event) {
    apply_event(state, event);
    for (obj_id, effects) in event.instant_effects.iter() {
        for effect in effects {
            apply_effect_instant(state, obj_id, effect);
        }
    }
    for (obj_id, effects) in event.timed_effects.iter() {
        for effect in effects {
            apply_effect_timed(state, obj_id, effect);
        }
    }
    for (id, abilities) in event.scheduled_abilities.iter() {
        for planned_ability in abilities {
            apply_scheduled_ability(state, id, planned_ability);
        }
    }
}

fn apply_event(state: &mut State, event: &Event) {
    match event.active_event {
        ActiveEvent::Create => {}
        ActiveEvent::MoveTo(ref ev) => apply_event_move_to(state, ev),
        ActiveEvent::Attack(ref ev) => apply_event_attack(state, &ev.attacker_id),
        ActiveEvent::EndTurn(ref ev) => apply_event_end_turn(state, ev.player_id),
        ActiveEvent::EndBattle(ref ev) => apply_event_end_battle(state, ev),
        ActiveEvent::BeginTurn(ref ev) => apply_event_begin_turn(state, ev),
        ActiveEvent::UseAbility(ref ev) => apply_event_use_ability(state, ev),
        ActiveEvent::UsePassiveAbility(_)
        | ActiveEvent::EffectTick(_)
        | ActiveEvent::EffectEnd(_) => {}
    }
}

pub fn add_components(state: &mut State, id: &Id, components: &[Component]) {
    let parts = state.parts_mut();
    for component in components {
        add_component(parts, *id, component);
    }
}

fn apply_event_move_to(state: &mut State, event: &event::MoveTo) {
    let parts = state.parts_mut();
    let pos = parts.pos.get_mut(&event.id).unwrap();
    pos.0 = event.path.to();

    let agent = parts.agent.get_mut(&event.id).unwrap();
    if agent.moves.0 > 0 {
        agent.moves.0 -= event.cost.0;
        assert!(agent.moves.0 >= 0);
    } else {
        agent.jokers.0 -= event.cost.0;
        assert!(agent.jokers.0 >= 0);
    }
}

pub fn apply_event_attack(state: &mut State, attacker_id: &Id) {
    let parts = state.parts_mut();
    let agent = parts.agent.get_mut(&attacker_id).unwrap();
    if agent.attacks.0 > 0 {
        agent.attacks.0 -= 1;
        assert!(agent.attacks.0 >= 0);
    } else {
        agent.jokers.0 -= 1;
        assert!(agent.jokers.0 >= 0);
    }
}

pub fn apply_event_end_turn(state: &mut State, this_player: PlayerId) {
    // Update attacks
    {
        // if the agent does not have stun, agent.attacks = agent.reactive_attacks,
        // otherwise agent.attacks = 0.
        let ids: Vec<Id> = state.parts().agent.keys().cloned().collect();
        for id in ids {
            let player_id = state.belongs_to(&id).0;

            let mut is_stun = false;
            let effects = state.parts().effects.get(&id);
            if let Some(effects) = effects {
                for effect in &effects.0 {
                    if let effect::Lasting::Stun = effect.effect {
                        is_stun = true;
                        break;
                    }
                }
            }

            let agent = state.parts_mut().agent.get_mut(&id).unwrap();
            if player_id == this_player {
                agent.attacks.0 += agent.reactive_attacks.0;
            }

            if is_stun {
                agent.attacks.0 = 0;
            }
        }
    }

    // Remove outdated planned abilities
    for (_, schedule) in state.parts_mut().schedule.iter_mut() {
        schedule.planned.retain(|p| p.rounds.0 > 0);
    }

    // Remove outdated lasting effect
    let ids = state
        .parts_mut()
        .effects
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for id in ids.iter() {
        // let mut effects = state.parts().effects.get(id).unwrap().0.clone();
        // effects.retain(|effect| !state::is_lasting_effect_over(state, id, &effect));
        // state.parts_mut().effects.get_mut(&id).unwrap().0 = effects;

        let strength = state.strength(id).strength;

        let effects = &mut state.parts_mut().effects.get_mut(id).unwrap().0;
        effects.retain(|effect| {
            if let effect::Lasting::Poison = effect.effect {
                if strength.0 <= 0 {
                    false
                } else {
                    true
                }
            } else {
                !effect.duration.is_over()
            }
        });
    }
}

pub fn update_reattack(state: &mut State, ids: &[Id]) {
    let parts = state.parts_mut();
    for id in ids {
        let agent = parts.agent.get_mut(&id).unwrap();
        agent.attacks.0 += agent.reactive_attacks.0;
    }
}

pub fn update_reattack_with_stun(state: &mut State, ids: &[Id]) {
    for id in ids {
        let mut is_stun = false;
        let effects = state.parts().effects.get(&id);
        if let Some(effects) = effects {
            for effect in &effects.0 {
                if let effect::Lasting::Stun = effect.effect {
                    is_stun = true;
                    break;
                }
            }
        }

        let agent = state.parts_mut().agent.get_mut(&id).unwrap();
        if is_stun {
            agent.attacks.0 = 0;
        } else {
            agent.attacks.0 += agent.reactive_attacks.0;
        }
    }
}

pub fn remove_outdated_planned_abilities(state: &mut State) {
    for (_, schedule) in state.parts_mut().schedule.iter_mut() {
        schedule.planned.retain(|p| p.rounds.0 > 0);
    }
}

pub fn remove_outdated_lasting_effect(state: &mut State) {
    let phase = state.player_id().0;

    let ids = state
        .parts_mut()
        .effects
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for id in ids.iter() {
        let strength = state.strength(id).strength;

        let effects = &mut state.parts_mut().effects.get_mut(id).unwrap().0;
        effects.retain(|effect| {
            if let effect::Lasting::Poison = effect.effect {
                if strength.0 <= 1 {
                    false
                } else {
                    true
                }
            } else {
                !effect.duration.is_over()
            }
        });

        for e in effects.iter_mut() {
            if e.phase.0 == phase {
                if let Duration::Rounds(ref mut rounds) = e.duration {
                    assert!(rounds.0 > 0);
                    rounds.decrease();
                }
            }
        }
    }
}

fn apply_lasting_effect_stun(state: &mut State, id: &Id) {
    let agent = state.parts_mut().agent.get_mut(id).unwrap();
    agent.moves.0 = 0;
    agent.attacks.0 = 0;
    agent.jokers.0 = 0;
}

fn update_lasting_effects_duration(state: &mut State) {
    let phase = Phase::from(state.player_id().0);
    for (_id, effects) in state.parts_mut().effects.iter_mut() {
        for effect in effects.0.iter_mut() {
            if effect.phase == phase {
                if let Duration::Rounds(ref mut rounds) = effect.duration {
                    assert!(rounds.0 > 0);
                    rounds.decrease();
                }
            }
        }
    }
}

pub fn reset_moves_and_attacks(state: &mut State, player_id: PlayerId) {
    for id in state::players_agent_ids(state, player_id) {
        let agent = state.parts_mut().agent.get_mut(&id).unwrap();
        agent.moves = agent.base_moves;
        agent.attacks = agent.base_attacks;
        agent.jokers = agent.base_jokers;
    }
}

pub fn reset_status(state: &mut State, ids: &[Id]) {
    for id in ids {
        let mut is_stun = false;
        if let Some(effects) = state.parts().effects.get(id).cloned() {
            for effect in effects.0 {
                if let effect::Lasting::Stun = effect.effect {
                    is_stun = true;
                    break;
                }
            }
        }

        let agent = state.parts_mut().agent.get_mut(id).unwrap();
        if is_stun {
            agent.moves.0 = 0;
            agent.attacks.0 = 0;
            agent.jokers.0 = 0;
        } else {
            agent.moves = agent.base_moves;
            agent.attacks = agent.base_attacks;
            agent.jokers = agent.base_jokers;
        }
    }
}

fn apply_event_end_battle(state: &mut State, event: &event::EndBattle) {
    state.set_battle_result(event.result.clone());
}

fn apply_event_begin_turn(state: &mut State, event: &event::BeginTurn) {
    state.set_player_id(event.player_id);
    update_lasting_effects_duration(state);
    reset_moves_and_attacks(state, event.player_id);
    apply_lasting_effects(state);
    update_cooldowns(state, event.player_id);
    tick_planned_abilities(state);
}

pub fn apply_event_use_ability(state: &mut State, event: &event::UseAbility) {
    let id = event.id;
    let parts = state.parts_mut();

    if let Some(abilities) = parts.abilities.get_mut(&id) {
        for r_ability in &mut abilities.0 {
            if r_ability.ability == event.ability {
                assert_eq!(r_ability.status, ability::Status::Ready);
                r_ability.status = ability::Status::Cooldown(r_ability.ability.base_cooldown());
            }
        }
    }

    if let Some(agent) = parts.agent.get_mut(&id) {
        if agent.attacks.0 > 0 {
            agent.attacks.0 -= 1;
        } else if agent.jokers.0 > 0 {
            agent.jokers.0 -= 1;
        } else {
            panic!("internal error: can't use ability if there're not attacks or jokers");
        }
    }
    match event.ability {
        Ability::Jump | Ability::LongJump | Ability::Dash => {
            parts.pos.get_mut(&id).unwrap().0 = event.pos;
        }
        Ability::Rage => {
            let component = parts.agent.get_mut(&id).unwrap();
            component.attacks.0 += 3;
        }
        Ability::Summon => {
            assert!(parts.summoner.get(&id).is_some());
            let summoner = parts.summoner.get_mut(&id).unwrap();
            summoner.count += 1;
        }
        _ => {}
    }
}

fn add_component(parts: &mut Parts, id: Id, component: &Component) {
    match component {
        Component::Pos(c) => {
            parts.pos.insert(id, c.clone());
        }
        Component::Strength(c) => {
            parts.strength.insert(id, c.clone());
        }
        Component::Meta(c) => {
            parts.meta.insert(id, c.clone());
        }
        Component::BelongsTo(c) => {
            parts.belongs_to.insert(id, c.clone());
        }
        Component::Agent(c) => {
            parts.agent.insert(id, c.clone());
        }
        Component::Blocker(c) => {
            parts.blocker.insert(id, c.clone());
        }
        Component::Abilities(c) => {
            parts.abilities.insert(id, c.clone());
        }
        Component::PassiveAbilities(c) => {
            parts.passive_abilities.insert(id, c.clone());
        }
        Component::Effects(c) => {
            parts.effects.insert(id, c.clone());
        }
        Component::Schedule(c) => {
            parts.schedule.insert(id, c.clone());
        }
        Component::Summoner(c) => {
            parts.summoner.insert(id, c.clone());
        }
    }
}

pub fn apply_scheduled_ability(state: &mut State, id: &Id, planned_ability: &PlannedAbility) {
    let planned = &mut state
        .parts_mut()
        .schedule
        .entry(*id)
        .or_insert(component::Schedule::default())
        .planned;

    if let Some(i) = planned
        .iter()
        .position(|e| e.ability == planned_ability.ability)
    {
        planned[i] = planned_ability.clone();
    } else {
        planned.push(planned_ability.clone());
    }
}

pub fn apply_effect_timed(state: &mut State, id: &Id, timed_effect: &effect::Timed) {
    let effects = &mut state
        .parts_mut()
        .effects
        .entry(*id)
        .or_insert(component::Effects(Vec::new()))
        .0;

    if let Some(i) = effects.iter().position(|e| e.effect == timed_effect.effect) {
        effects[i] = timed_effect.clone();
    } else {
        effects.push(timed_effect.clone());
    }
}

fn apply_effect_instant(state: &mut State, id: &Id, effect: &Effect) {
    match effect {
        Effect::Create(effect) => apply_effect_create(state, id, effect),
        Effect::Kill(_) => apply_effect_kill(state, id),
        Effect::Vanish => apply_effect_vanish(state, id),
        Effect::Stun => apply_effect_stun(state, id),
        Effect::Heal(effect) => apply_effect_heal(state, id, effect),
        Effect::Wound(effect) => apply_effect_wound(state, id, effect.damage.0),
        Effect::Knockback(effect) => apply_effect_knockback(state, id, effect),
        Effect::FlyOff(effect) => apply_effect_fly_off(state, id, effect),
        Effect::Throw(effect) => apply_effect_throw(state, id, effect),
        Effect::Bloodlust => apply_effect_bloodlust(state, id),
        Effect::Dodge(_) => {}
    }
}

fn apply_effect_create(state: &mut State, id: &Id, effect: &effect::Create) {
    add_components(state, id, &effect.components);
}

fn apply_effect_kill(state: &mut State, id: &Id) {
    state.parts_mut().remove(id);
}

fn apply_effect_vanish(state: &mut State, id: &Id) {
    let parts = state.parts_mut();
    parts.remove(&id);
}

pub fn apply_effect_stun(state: &mut State, id: &Id) {
    let agent = state.parts_mut().agent.get_mut(id).unwrap();
    agent.moves.0 = 0;
    agent.attacks.0 = 0;
    agent.jokers.0 = 0;
}

// TODO: split `Heal` effect into two? `Heal` + `RemoveLastingEffects`?
fn apply_effect_heal(state: &mut State, id: &Id, effect: &effect::Heal) {
    let parts = state.parts_mut();
    {
        let component = parts.strength.get_mut(id).unwrap();
        component.strength.0 += effect.strength.0;
        if component.strength > component.base_strength {
            component.strength = component.base_strength;
        }
    }
    if let Some(effects) = parts.effects.get_mut(id) {
        effects.0.clear();
    }
}

pub fn apply_effect_wound(state: &mut State, id: &Id, damage: i32) {
    assert!(damage >= 0);

    let parts = state.parts_mut();
    parts.strength.get_mut(id).unwrap().strength.0 -= damage;

    // TODO: Check why attack - 1?
    {
        let agent = parts.agent.get_mut(id).unwrap();
        agent.attacks.0 -= 1;
        if agent.attacks.0 < 0 {
            agent.attacks.0 = 0;
        }
    }
}

fn apply_effect_knockback(state: &mut State, id: &Id, effect: &effect::Knockback) {
    if effect.to == effect.from {
        return;
    }
    state.parts_mut().pos.get_mut(id).unwrap().0 = effect.to;
}

fn apply_effect_fly_off(state: &mut State, id: &Id, effect: &effect::FlyOff) {
    assert!(state.map().is_inboard(effect.from));
    if effect.to == effect.from {
        return;
    }
    assert!(state.map().is_inboard(effect.to));
    assert!(!state::is_tile_blocked(state, effect.to));
    state.parts_mut().pos.get_mut(&id).unwrap().0 = effect.to;
}

fn apply_effect_throw(state: &mut State, id: &Id, effect: &effect::Throw) {
    state.parts_mut().pos.get_mut(&id).unwrap().0 = effect.to;
}

fn apply_effect_bloodlust(state: &mut State, id: &Id) {
    let agent = state.parts_mut().agent.get_mut(&id).unwrap();
    agent.jokers.0 += 3;
}

fn update_cooldowns_for_object(state: &mut State, id: &Id) {
    if let Some(abilities) = state.parts_mut().abilities.get_mut(&id) {
        for ability in &mut abilities.0 {
            ability.status.update();
        }
    }
}

pub fn update_cooldowns(state: &mut State, player_id: PlayerId) {
    for id in state::players_agent_ids(state, player_id) {
        update_cooldowns_for_object(state, &id);
    }
}

fn apply_lasting_effects(state: &mut State) {
    for id in state::players_agent_ids(state, state.player_id()).iter() {
        if let Some(effects) = state.parts().effects.get(id).cloned() {
            for effect in effects.0 {
                if let effect::Lasting::Stun = effect.effect {
                    apply_lasting_effect_stun(state, id);
                }
            }
        }
    }
}

pub fn tick_planned_abilities(state: &mut State) {
    let phase = Phase::from(state.player_id().0);
    for (_, schedule) in state.parts_mut().schedule.iter_mut() {
        for planned in schedule.planned.iter_mut() {
            if planned.phase == phase {
                planned.rounds.decrease();
            }
        }
    }
}
