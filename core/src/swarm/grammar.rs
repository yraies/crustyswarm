use std::time::Instant;

use cgmath::prelude::*;
use cgmath::{Deg, Rad};
use cgmath::{Vector2, Vector3};
use rand::Rng;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use crate::utils::*;
use swarm::actor::*;
use swarm::distribution::StartDistribution;
use swarm::ruleset::*;
use swarm::species::*;
use swarm::world::{ChunkedWorld, World};
use swarm::Val;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmGrammar {
    pub world: ChunkedWorld,
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

        println!("{} Agents", self.world.get_all_agents().count());

        // 1. Replace by Rules          -------------------------------------
        let mut start = Instant::now();
        let replaced = self.replace_agents(rnd);
        println!("replacement {:3.1?}", start.elapsed());

        // 2. Recalculate Velocities    -------------------------------------
        start = Instant::now();
        self.recalc_agent(rnd);
        println!("recalc      {:3.1?}", start.elapsed());

        // 3. Recalculate Buoys         -------------------------------------
        start = Instant::now();
        self.world.update_terrain();
        println!("buoys rec   {:3.1?}", start.elapsed());
    }

    fn replace_agents(&mut self, rnd: &mut impl Rng) {
        if self.template.strategy.should_replace() {
            let mut res = self
                .template
                .rule_sets
                .iter()
                .map(|rules| rules.execute(&self.template, self.world.get_all_agents(), rnd))
                .fold(
                    (Vec::<Agent>::new(), Vec::<Buoy>::new()),
                    |mut acc, mut val| {
                        acc.0.append(&mut val.0);
                        acc.1.append(&mut val.1);
                        acc
                    },
                );
            self.world.insert_buoys(res.1);
            self.world.set_agents(res.0);
        }
    }

    pub fn recalc_agent(&mut self, rnd: &mut impl Rng) {
        let mut rnd_vec = Vec::new();
        for _i in 0..self.world.get_all_agents().count() {
            rnd_vec.push(random_one(rnd));
        }

        let recalculated = self
            .world
            .get_all_agents()
            .enumerate()
            .map(|(agent_index, agent)| {
                let agent_species = &self.template.species[agent.species_index];

                // 2.1. Prepare Vectors

                let mut sep_vec = Vector3::zero();
                let mut ali_vec = Vector3::zero();
                let mut coh_vec = Vector3::zero();

                let mut sep_counter = 0.0;
                let mut view_counter = 0.0;

                for (other_index, other) in self
                    .world
                    .get_agents_at_least_within(
                        agent_species.view_distance,
                        Vector2::new(agent.position.x, agent.position.z),
                    )
                    .enumerate()
                {
                    //check for self
                    if other_index == agent_index {
                        continue;
                    }

                    // Find influence in influence vector
                    let inf_opt = agent_species
                        .influence
                        .iter()
                        .find(|&&i| (i.0) == other_index)
                        .map(|v| v.1);

                    // Default influence = 0
                    match inf_opt {
                        None => (),
                        Some(influence) => {
                            let dist = agent.position.distance(other.position);

                            if dist < agent_species.view_distance {
                                if dist < agent_species.sep_distance {
                                    sep_vec += other.position * influence;
                                    sep_counter += 1.0 * influence.abs();
                                }

                                let solid_angle =
                                    agent.velocity.angle(other.position - agent.position);

                                if solid_angle > Rad::from(Deg(90.0)) {
                                    continue;
                                }

                                ali_vec += other.velocity * influence;
                                coh_vec += other.position * influence;
                                view_counter += 1.0 * influence.abs();
                            }
                        }
                    }
                }

                let sep_temp = safe_devide_mean(sep_vec, sep_counter);

                let sep_norm = -(if sep_temp.is_zero() {
                    sep_temp
                } else {
                    sep_temp - agent.position
                });
                let (ali_norm, coh_norm) = if view_counter > 0.0 {
                    let an = safe_devide_mean(ali_vec, view_counter);
                    let cn = safe_devide_mean(coh_vec, view_counter) - agent.position;
                    (an, cn)
                } else {
                    (Vector3::<Val>::zero(), Vector3::<Val>::zero())
                };
                let cen_norm = agent.seed_center - agent.position;

                let rnd_norm = rnd_vec[agent_index];

                let base_dist = agent.position.y - agent.seed_center.y;
                let gravity = -Vector3::<Val>::unit_y()
                    * (base_dist * base_dist / 2000.0 + base_dist / 200.0);

                // 2.2. Actually Recalculate    ------------------

                let acceleration = agent_species.separation * sep_norm * 0.01
                    + agent_species.alignment * ali_norm * 0.1
                    + agent_species.cohesion * coh_norm * 0.01
                    + agent_species.center * cen_norm * 0.01
                    + agent_species.randomness * rnd_norm * 0.1;

                let con = agent_species.axis_constraint;

                let mut new_velocity = agent.velocity + acceleration;

                new_velocity = Vector3::new(
                    new_velocity.x * con[0],
                    new_velocity.y * con[1],
                    new_velocity.z * con[2],
                );

                let clipped_new_velocity = if new_velocity.magnitude() > agent_species.max_speed {
                    new_velocity.normalize_to(agent_species.max_speed)
                } else {
                    new_velocity
                };

                let new_position = new_velocity + agent_species.mass * gravity;

                let clipped_new_position =
                    if agent_species.noclip && new_position.y < agent.seed_center.y {
                        Vector3::new(new_position.x, agent.seed_center.y, new_position.z)
                    } else {
                        new_position
                    };

                //println!("s{} a{} c{} r{}  - {}", svec(&sep), svec(&ali), svec(&coh), svec(&rnd), svec(&clipped_new_velocity));

                let mut out_agent = agent.clone();

                out_agent.velocity = clipped_new_velocity;
                out_agent.position += clipped_new_position;
                out_agent.energy -= match agent_species.depletion_energy {
                    DepletionEnergy::Constant(v) => v,
                    DepletionEnergy::Distance(v) => v * agent.velocity.magnitude(),
                    DepletionEnergy::None => 0.0,
                };
                out_agent
            })
            .collect();
        self.world.set_agents(recalculated);
    }

    pub fn get_world(&self) -> &ChunkedWorld {
        &self.world
    }
}
