use super::Val;
use super::SpeciesIndex;
use Agent;
use rand::Rng;
use std::fmt;


//#[derive(Debug)]
pub struct RuleSet {
    pub input: SpeciesIndex,
    pub rules: Vec<(Vec<SpeciesIndex>, Val)>,
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

                for (replacements, prob) in &self.rules {
                    prob_counter += prob;
                    if prob_counter > thresh {
                        for s in replacements {
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