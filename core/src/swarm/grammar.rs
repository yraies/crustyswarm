use std::time::Instant;

use cgmath::prelude::*;
use cgmath::Rad;
use cgmath::Vector3;
use rand::Rng;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use Agent;
use RuleSet;
use Species;

use crate::utils::*;

use super::Val;

#[derive(Debug)]
pub struct SwarmGrammar {
    pub agents: Vec<Agent>,
    pub species: Vec<Species>,
    pub rule_sets: Vec<RuleSet>,
}

impl SwarmGrammar {
    pub fn step(&mut self, rnd: &mut impl Rng) {
        println!("{} Agents", &self.agents.len());

        // 1. Replace by Rules          -------------------------------------
        let mut start = Instant::now();
        let replaced: Vec<Agent> = self.rule_sets.iter().flat_map(|rules| rules.execute(&self.agents, rnd)).collect();
        println!("replacement {:3.1?}", start.elapsed());

        // 2. Recalculate Velocities    -------------------------------------

        start = Instant::now();
        let mut rnd_vec = Vec::new();
        let minus_vec: Vector3<Val> = Vector3::new(-1.0, -1.0, -1.0);
        for _i in 0..replaced.len() {
            rnd_vec.push((rnd.gen::<Vector3<Val>>() * 2.0) - minus_vec);
        }
        println!("random      {:3.1?}", start.elapsed());

        start = Instant::now();
        let recalculated = replaced.par_iter().enumerate().map(|(agent_index, agent)| {
            let agent_species = &self.species[agent.species_index];

            // 2.1. Prepare Vectors

            let mut sep_vec = Vector3::zero();
            let mut ali_vec = Vector3::zero();
            let mut coh_vec = Vector3::zero();

            let mut sep_counter = 0;
            let mut view_counter = 0;

            for other_index in 0..replaced.len() {
                if other_index == agent_index {
                    continue;
                }

                let other = &replaced[other_index];


                let dist = agent.position.distance(other.position);

                if dist < agent_species.view_distance {
                    if dist < agent_species.sep_distance {
                        sep_vec += other.position;
                        sep_counter += 1;
                    }

                    let solid_angle = agent.velocity.angle(other.position - agent.position);

                    if solid_angle > Rad(0.4) {
                        continue;
                    }

                    ali_vec += safe_normalize(other.velocity);
                    coh_vec += other.position;
                    view_counter += 1;
                }
            }

            let sep_temp = safe_devide_mean(sep_vec, sep_counter);

            let sep_norm = - safe_normalize(if sep_temp.is_zero() { sep_temp } else { sep_temp - agent.position });
            let ali_norm = safe_normalize(safe_devide_mean(ali_vec, view_counter));
            let coh_norm = safe_normalize(safe_devide_mean(coh_vec, view_counter) - agent.position);
            let cen_norm = safe_normalize(-agent.position);

            let rnd_norm = safe_normalize(rnd_vec[agent_index]);

            // 2.2. Actually Recalculate    ------------------

            let acceleration = agent_species.separation * sep_norm
                + agent_species.alignment * ali_norm
                + agent_species.cohesion * coh_norm
                + agent_species.center * cen_norm
                + agent_species.randomness * rnd_norm;

            let new_velocity = agent.velocity + acceleration;

            let clipped_new_velocity = if new_velocity.magnitude() > agent_species.max_speed {
                new_velocity.normalize_to(agent_species.max_speed)
            } else {
                new_velocity
            };

            //println!("s{} a{} c{} r{}  - {}", svec(&sep), svec(&ali), svec(&coh), svec(&rnd), svec(&clipped_new_velocity));

            Agent::mk_new(agent.position, clipped_new_velocity, agent.energy, agent.species_index).unwrap()
        }).collect();
        println!("recalc      {:3.1?}", start.elapsed());

        // 3. Move accordingly          -------------------------------------

        start = Instant::now();
        self.agents = recalculated;

        self.agents
            .iter_mut()
            .for_each(|agent| agent.position += agent.velocity);
        println!("moved       {:3.1?}", start.elapsed());
    }

    pub fn get_agents(&self) -> &[Agent] {
        &self.agents
    }
}