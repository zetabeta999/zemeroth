use battle::{
    battle::heroes::{Hero, HeroObject},
    game::{Input, Level},
};
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ExecutorImpl};

pub fn prove_game(input: Input) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let start = std::time::Instant::now();
    let receipt = prover.prove(env, METHOD_ELF).unwrap().receipt;
    println!("prover time: {:.2?}", start.elapsed());

    assert!(receipt.verify(METHOD_ID).is_ok());
}

pub fn run(input: Input) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let mut exec = ExecutorImpl::from_elf(env, &METHOD_ELF).unwrap();
    let session = exec.run().unwrap();
    println!("cycle:{}", session.user_cycles);
}

fn main() {
    // The player's command is simply `End Turn`.
    let commands = vec![
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    ];

    let heroes = vec![
        HeroObject::new(Hero::Swordsman, 1),
        HeroObject::new(Hero::EliteSpearman, 1),
    ];

    let input = Input {
        commands,
        level: Level::Level0,
        heroes,
    };

    run(input);
}
