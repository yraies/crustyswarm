use std::fmt;

use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

use Agent;

use super::SpeciesIndex;
use super::Val;
use core::borrow::BorrowMut;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RuleStrategy {
    Every(usize, usize),
}

impl RuleStrategy {
  pub fn shouldReplace(&mut self) -> bool {
    match self {
      RuleStrategy::Every(max, ref mut curr) => {
          if *curr > 1 {
              *curr = *curr - 1;
            false
          } else {
              *curr = *max;
            true
          }
      }
    }
  }
}



#[derive(Serialize, Deserialize)]
pub struct RuleSet {
    pub input: SpeciesIndex,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub p: Val,
    pub out: Vec<SpeciesIndex>,
}


impl Rule {
    pub fn new(out: Vec<SpeciesIndex>, p: Val) -> Rule {
        Rule { out, p }
    }
}

impl fmt::Debug for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rule {}: {:?}", self.input, self.rules)
    }
}

impl RuleSet {
    pub fn execute(&self, agents: &[Agent], rnd: &mut impl Rng) -> Vec<Agent> {
        let mut new_vec = Vec::new();

        for agent in agents {
            if agent.species_index == self.input {
                let thresh = rnd.gen();
                let mut prob_counter: Val = 0.0;

                for r in &self.rules {
                    prob_counter += r.p;
                    if prob_counter > thresh {
                        for s in &r.out {
                            let mut new_agent = agent.clone();
                            new_agent.species_index = *s;
                            new_vec.push(new_agent);
                        }
                        break;
                    }
                }
            }
        }

        new_vec
    }
}
