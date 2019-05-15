use std::time::Instant;

use cgmath::prelude::*;
use cgmath::Rad;
use cgmath::{Vector2, Vector3};
use rand::Rng;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use super::distribution::StartDistribution;
use swarm::actor::{Agent, Artifact, Buoy};
use swarm::ruleset::RuleStrategy;
use swarm::species::*;
use RuleSet;

use crate::utils::*;
use serde::Deserialize;
use serde::Serialize;

use super::Val;

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmGrammar {
    pub agents: Vec<Agent>,
    pub buoys: Vec<Buoy>,
    pub artifacts: Vec<Artifact>,
    pub template: SwarmTemplate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmTemplate {
    pub species: Vec<Species>,
    pub rule_sets: Vec<RuleSet>,
    pub start_dist: StartDistribution,
    pub strategy: RuleStrategy,
}

impl SwarmGrammar {
    pub fn step(&mut self, rnd: &mut impl Rng) {
        // Replace Agents
        // Recalc Agents
        // Spawn Artifacts
        // Recalc Buoys

        println!("{} Agents", &self.agents.len());

        // 1. Replace by Rules          -------------------------------------
        let mut start = Instant::now();
        let mut replaced = self.replace_agents(rnd);
        println!("replacement {:3.1?}", start.elapsed());

        // 2. Recalculate Velocities    -------------------------------------
        start = Instant::now();
        self.recalc_agent(rnd, &mut replaced);
        println!("recalc      {:3.1?}", start.elapsed());

        // 3. Spawn Artifacts           -------------------------------------

        // 4. Recalculate Buoys         -------------------------------------
        start = Instant::now();
        self.move_buoys();
        println!("buoys rec   {:3.1?}", start.elapsed());
    }

    fn replace_agents(&mut self, rnd: &mut impl Rng) -> Vec<Agent> {
        if self.template.strategy.should_replace() {
            self.template
                .rule_sets
                .iter()
                .flat_map(|rules| rules.execute(&self.agents, rnd))
                .collect()
        } else {
            self.agents.to_owned()
        }
    }

    pub fn recalc_agent(&mut self, rnd: &mut impl Rng, replaced: &mut Vec<Agent>) {
        let mut rnd_vec = Vec::new();
        for _i in 0..replaced.len() {
            rnd_vec.push(random_one(rnd));
        }

        let recalculated = replaced
            .par_iter()
            .enumerate()
            .map(|(agent_index, agent)| {
                let agent_species = &self.template.species[agent.species_index];

                // 2.1. Prepare Vectors

                let mut sep_vec = Vector3::zero();
                let mut ali_vec = Vector3::zero();
                let mut coh_vec = Vector3::zero();

                let mut sep_counter = 0.0;
                let mut view_counter = 0.0;

                for (other_index, other) in replaced.iter().enumerate() {
                    if other_index == agent_index {
                        continue;
                    }

                    let inf_opt = agent_species
                        .influence
                        .iter()
                        .find(|&&i| (i.0) == other_index)
                        .map(|v| v.1);
                    match inf_opt {
                        None => (),
                        Some(influence) => {
                            let dist = agent.position.distance(other.position);

                            if dist < agent_species.view_distance {
                                if dist < agent_species.sep_distance {
                                    sep_vec += other.position * influence;
                                    sep_counter += 1.0 * influence;
                                }

                                let solid_angle =
                                    agent.velocity.angle(other.position - agent.position);

                                if solid_angle > Rad(0.4) {
                                    continue;
                                }

                                ali_vec += safe_normalize(other.velocity * influence);
                                coh_vec += other.position * influence;
                                view_counter += 1.0 * influence;
                            }
                        }
                    }
                }

                let sep_temp = safe_devide_mean(sep_vec, sep_counter);

                let sep_norm = -safe_normalize(if sep_temp.is_zero() {
                    sep_temp
                } else {
                    sep_temp - agent.position
                });
                let (ali_norm, coh_norm) = if view_counter > 0.0 {
                    let an = safe_normalize(safe_devide_mean(ali_vec, view_counter));
                    let cn =
                        safe_normalize(safe_devide_mean(coh_vec, view_counter) - agent.position);
                    (an, cn)
                } else {
                    (Vector3::<Val>::zero(), Vector3::<Val>::zero())
                };
                let cen_norm = safe_normalize(-agent.position);

                let rnd_norm = safe_normalize(rnd_vec[agent_index]);

                let gravity = if agent.position.y > 20.0 {
                    -Vector3::<Val>::unit_y() * 0.1
                } else if agent.position.y > 0.0 {
                    -Vector3::<Val>::unit_y() * 0.1 * (20.0 - agent.position.y) / 20.0
                } else {
                    Vector3::<Val>::unit_y() * 0.05
                };

                // 2.2. Actually Recalculate    ------------------

                let acceleration = agent_species.separation * sep_norm
                    + agent_species.alignment * ali_norm
                    + agent_species.cohesion * coh_norm
                    + agent_species.center * cen_norm
                    + agent_species.randomness * rnd_norm
                    + agent_species.weight * gravity;

                let con = agent_species.axis_constraint;
                let acc_constrained = Vector3::new(
                    acceleration.x * con[0],
                    acceleration.y * con[1],
                    acceleration.z * con[2],
                );

                let new_velocity = agent.velocity + acc_constrained;

                let clipped_new_velocity = if new_velocity.magnitude() > agent_species.max_speed {
                    new_velocity.normalize_to(agent_species.max_speed)
                } else {
                    new_velocity
                };

                //println!("s{} a{} c{} r{}  - {}", svec(&sep), svec(&ali), svec(&coh), svec(&rnd), svec(&clipped_new_velocity));

                let mut out_agent = agent.clone();

                out_agent.velocity = clipped_new_velocity;
                out_agent.position += out_agent.velocity;
                out_agent.energy -= match agent_species.energy_strategy {
                    EnergyStrategy::Constant(v) => v,
                    EnergyStrategy::Distance(v) => v * agent.velocity.magnitude(),
                    EnergyStrategy::None => 0.0,
                };
                out_agent
            })
            .collect();
        self.agents = recalculated;
    }

    fn move_buoys(&mut self) {
        let bs: &mut Vec<Buoy> = &mut self.buoys;
        let ags: &Vec<Agent> = &self.agents;

        bs.iter_mut().for_each(|b| {
            let mut factors = 0.5;
            let mut d = -0.1;

            // keep above ground
            if b.position.y < 0.0 {
                d = 0.1;
            }

            for a in ags {
                let bpos = Vector2::new(b.position.x, b.position.z);
                let apos = Vector2::new(a.position.x, a.position.z);
                let dist = bpos.distance(apos);
                // let factor = 1.0 / (1.0 + dist).powf(1.5);
                let thresh = 25.5;
                let factor = (if dist > thresh {
                    0.0
                } else {
                    (thresh - dist) / thresh
                })
                .powf(2.0);
                let ydist = a.position.y - b.position.y;

                if !(factor.is_nan() || ydist.is_nan()) {
                    factors += factor;
                    d += ydist * factor;
                }
            }

            let vel = if d == 0.0 { 0.0 } else { d / factors };

            b.position.y += vel * 0.5;
        });
    }

    pub fn get_agents(&self) -> &[Agent] {
        &self.agents
    }
    pub fn get_buoys(&self) -> &[Buoy] {
        &self.buoys
    }
    pub fn get_artifacts(&self) -> &[Artifact] {
        &self.artifacts
    }
}
