use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    battle::{component::ObjType, scenario::Scenario, state::BattleResult, PlayerId},
    utils::{self},
};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// Recruiting/upgrading fighters or starting a new battle.
    PreparingForBattle,

    /// Campaign is finished, the player have won.
    Won,

    /// Campaign is finished, the player have lost.
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, derive_more::From)]
#[serde(transparent)]
pub struct Renown(pub i32);

/// An award that is given to the player after the successful battle.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Award {
    #[serde(default)]
    pub recruits: Vec<ObjType>,

    pub renown: Renown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Action {
    Recruit { agent_type: ObjType },
    Upgrade { from: ObjType, to: ObjType },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CampaignNode {
    pub scenario: Scenario,
    pub award: Award,
}

#[allow(dead_code)]
fn casualties(initial_agents: &[ObjType], survivors: &[ObjType]) -> Vec<ObjType> {
    let mut agents = initial_agents.to_vec();
    for typename in survivors {
        assert!(utils::try_remove_item(&mut agents, typename));
    }
    agents
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Plan {
    initial_agents: Vec<ObjType>,
    nodes: Vec<CampaignNode>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentInfo {
    pub cost: Renown,

    #[serde(default)]
    pub upgrades: Vec<ObjType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct State {
    scenarios: Vec<CampaignNode>,
    current_scenario_index: i32,
    mode: Mode,
    agents: Vec<ObjType>,
    last_battle_casualties: Vec<ObjType>,
    agent_info: HashMap<ObjType, AgentInfo>,
    actions: Vec<Action>,
    renown: Renown,
}

impl State {
    pub fn new(plan: Plan, agent_info: HashMap<ObjType, AgentInfo>) -> Self {
        assert!(!plan.nodes.is_empty(), "No scenarios");
        Self {
            current_scenario_index: 0,
            scenarios: plan.nodes,
            mode: Mode::PreparingForBattle,
            agents: plan.initial_agents,
            last_battle_casualties: Vec::new(),
            actions: Vec::new(),
            agent_info,
            renown: Renown(0),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn last_battle_casualties(&self) -> &[ObjType] {
        &self.last_battle_casualties
    }

    pub fn scenario(&self) -> &Scenario {
        assert!(!self.scenarios.is_empty());
        let i = self.current_scenario_index as usize;
        &self.scenarios[i].scenario
    }

    pub fn current_scenario_index(&self) -> i32 {
        self.current_scenario_index
    }

    pub fn scenarios_count(&self) -> i32 {
        self.scenarios.len() as _
    }

    pub fn agents(&self) -> &[ObjType] {
        &self.agents
    }

    pub fn renown(&self) -> Renown {
        self.renown
    }

    pub fn available_actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn execute_action(&mut self, action: Action) {
        assert_eq!(self.mode(), Mode::PreparingForBattle);
        assert!(utils::try_remove_item(&mut self.actions, &action));
        let cost = self.action_cost(&action);
        assert!(self.renown.0 >= cost.0);
        self.renown.0 -= cost.0;
        match action {
            Action::Recruit { agent_type } => {
                self.agents.push(agent_type);
            }
            Action::Upgrade { from, to } => {
                assert!(utils::try_remove_item(&mut self.agents, &from));
                self.agents.push(to);
            }
        }
    }

    pub fn action_cost(&self, action: &Action) -> Renown {
        match action {
            Action::Recruit { agent_type } => {
                let squad_size_penalty = self.agents.len() as i32;
                let agent_cost = self.agent_info[agent_type].cost;
                Renown(agent_cost.0 + squad_size_penalty)
            }
            Action::Upgrade { from, to } => {
                let cost_from = self.agent_info[from].cost;
                let cost_to = self.agent_info[to].cost;
                Renown(cost_to.0 - cost_from.0)
            }
        }
    }

    pub fn report_battle_results(&mut self, result: &BattleResult) -> Result<(), ()> {
        {
            if self.mode != Mode::PreparingForBattle {
                return Err(());
            }

            if result.winner_id == PlayerId(0) && result.survivor_types.is_empty() {
                // You can't win with no survivors.
                return Err(());
            }

            for survivor in &result.survivor_types {
                if !self.agents.contains(survivor) {
                    // This agent isn't a survivor.
                    return Err(());
                }
            }

            if result.winner_id != PlayerId(0) {
                self.mode = Mode::Failed;
                return Ok(());
            }
        }

        self.actions.clear();

        self.last_battle_casualties = casualties(&self.agents, &result.survivor_types);
        self.agents = result.survivor_types.clone();

        if self.current_scenario_index + 1 >= self.scenarios.len() as _ {
            self.mode = Mode::Won;
        } else {
            let i = self.current_scenario_index as usize;
            let award = &self.scenarios[i].award;
            self.renown.0 += award.renown.0;
            for recruit in &award.recruits {
                let action = Action::Recruit {
                    agent_type: recruit.clone(),
                };
                self.actions.push(action);
            }

            if self.current_scenario_index() != 0 {
                let mut set = HashSet::new();
                for agent in &self.agents {
                    if !set.contains(agent) {
                        let agent_info = self.agent_info.get(agent).unwrap();
                        for upgrade in &agent_info.upgrades {
                            let from = agent.clone();
                            let to = upgrade.clone();
                            self.actions.push(Action::Upgrade { from, to });
                        }
                        set.insert(agent.clone());
                    }
                }
            }

            self.current_scenario_index += 1;
            self.mode = Mode::PreparingForBattle;
        }

        Ok(())
    }
}
