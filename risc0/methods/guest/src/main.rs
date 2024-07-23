use battle::{
    battle::{
        ai::Ai,
        command::{self, Command},
        execute::execute,
        PlayerId,
    },
    game::Input,
    utils::SimpleRng,
};

use risc0_zkvm::guest::env;

pub fn main() {
    let input: Input = env::read();
    assert!(input.commands.len() <= 10);

    let mut rng = SimpleRng::seed_from_u32(0);
    let mut state = input.level.state();
    state.create_heroes(&input.heroes, &mut rng);

    let mut ai = Ai::new(PlayerId(1));

    for round in input.commands.iter() {
        for command in round {
            execute(&mut state, command, &mut rng).unwrap();
        }

        if state.battle_result().is_some() {
            break;
        }

        execute(&mut state, &Command::EndTurn(command::EndTurn), &mut rng).unwrap();

        ai.update_obj_ids(&state);

        while let Some(command) = ai.command(&mut state) {
            execute(&mut state, &command, &mut rng).unwrap();
            if let Command::EndTurn(_) = command {
                break;
            }
        }
    }

    env::commit(&state.battle_result());
}
