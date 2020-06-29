use std::time::Instant;

use cgmath::prelude::*;
use cgmath::Vector3;
use cgmath::{Deg, Rad};
use rand::Rng;
use rayon::prelude::*;

use super::actor::Agent;
use crate::utils::*;
use swarm::genome::SwarmGenome;
use swarm::world::{ChunkedWorld, World};

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct SwarmGrammar {
    pub world: ChunkedWorld,
    pub genome: SwarmGenome,
}

impl SwarmGrammar {
    pub fn step(&mut self, rnd: &mut impl Rng) {
        // Replace Agents
        // Recalc Agents
        // Spawn Artifacts
        // Recalc Buoys

        println!("{} Agents", self.world.get_all_agents().count());

        // 1. Replace by Rules          -------------------------------------
        self.genome.tick();
        let mut start = Instant::now();
        self.world.replace_by(&self.genome, rnd);
        println!("replacement {:3.1?}", start.elapsed());

        // 2. Recalculate Velocities    -------------------------------------
        start = Instant::now();
        self.recalc_agent(rnd);
        println!("recalc      {:3.1?}", start.elapsed());

        // 3. Recalculate Buoys         -------------------------------------
        start = Instant::now();
        self.world.update_terrain((
            &self.genome.terrain_influences.0,
            &self.genome.terrain_influences.1,
        ));
        println!("buoys rec   {:3.1?}", start.elapsed());
    }

    pub fn recalc_agent(&mut self, rnd: &mut impl Rng) {
        let agent_random_pairs: Vec<_> = self
            .world
            .get_all_agents()
            .map(|a| (random_one(rnd), a))
            .collect();
        let mut recalculated: Vec<Agent> = agent_random_pairs
            .iter()
            .map(|a| self.move_agents(*a))
            .collect();
        recalculated.sort_by_key(|agent| agent.id);
        self.world.set_agents(recalculated);
    }

    fn move_agents(&self, (randomness, agent): (Vector3<f32>, &Agent)) -> Agent {
        let agent_species = &self.genome.get_species(agent);

        // 2.1. Prepare Vectors

        let mut sep_vec = Vector3::zero();
        let mut ali_vec = Vector3::zero();
        let mut coh_vec = Vector3::zero();

        let mut sep_counter = 0.0;
        let mut view_counter = 0.0;
        let mut artifact_view_counter = 0.0;
        dbg!("---------");

        for (dist, other) in self
            .world
            .get_context_within(agent_species.view_distance, agent.position)
        {
            if agent.id == other.get_id() {
                continue;
            }

            // Find influence in influence vector
            let inf_opt = agent_species
                .influenced_by
                .get(&other.get_surrounding_index());

            // Default influence = 0
            match inf_opt {
                None => (),
                Some(&influence) => {
                    if dist < agent_species.view_distance {
                        if dist < agent_species.sep_distance {
                            sep_vec += other.get_position() * influence;
                            sep_counter += 1.0 * influence.abs();
                        }

                        let solid_angle =
                            agent.velocity.angle(other.get_position() - agent.position);

                        if solid_angle > Rad::from(Deg(170.0)) {
                            continue;
                        }

                        use super::actor::Actor;

                        match other {
                            Actor::Agent(other_agent) => {
                                ali_vec += other_agent.velocity * influence;
                                coh_vec += other_agent.position * influence;
                                view_counter += 1.0 * influence.abs();
                            }
                            Actor::Artifact(other_artifact) => {
                                coh_vec += other_artifact.position * influence;
                                artifact_view_counter += 1.0 * influence.abs();
                            }
                        }
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
            let cn =
                safe_devide_mean(coh_vec, view_counter + artifact_view_counter) - agent.position;
            (an, cn)
        } else {
            (Vector3::<f32>::zero(), Vector3::<f32>::zero())
        };
        let cen_norm = agent.seed_center - agent.position;

        let rnd_norm = randomness;

        let base_dist = self.world.get_height(agent);
        let gravity =
            -Vector3::<f32>::unit_y() * (base_dist * base_dist / 2000.0 + base_dist / 200.0);
        dbg!(gravity);

        // 2.2. Actually Recalculate    ------------------

        let acceleration = dbg!(agent_species.separation * sep_norm)
            + dbg!(agent_species.alignment * ali_norm)
            + dbg!(agent_species.cohesion * coh_norm)
            + dbg!(agent_species.center * cen_norm)
            + dbg!(agent_species.randomness * rnd_norm);

        let con = agent_species.axis_constraint;

        let mut new_velocity = agent.velocity + acceleration;

        new_velocity = Vector3::new(
            new_velocity.x * con.x,
            new_velocity.y * con.y,
            new_velocity.z * con.z,
        );

        let clipped_new_velocity = if new_velocity.magnitude() > agent_species.max_speed {
            new_velocity.normalize_to(agent_species.max_speed)
        } else {
            new_velocity
        };

        let new_position =
            agent.position + new_velocity + dbg!(agent_species.mass * 0.01 * gravity);

        let new_floor = self.world.get_height_at(new_position.x, new_position.z);

        let clipped_new_position = if !agent_species.noclip && new_floor > new_position.y {
            Vector3::new(new_position.x, new_floor, new_position.z)
        } else {
            new_position
        };

        //println!("s{} a{} c{} r{}  - {}", svec(&sep), svec(&ali), svec(&coh), svec(&rnd), svec(&clipped_new_velocity));

        let mut out_agent = agent.clone();

        out_agent.velocity = clipped_new_velocity;
        out_agent.position = clipped_new_position;
        out_agent.energy -= agent_species
            .depletion_energy
            .get(agent.velocity.magnitude());
        out_agent
    }

    pub fn get_world(&self) -> &ChunkedWorld {
        &self.world
    }

    pub fn from(genome: SwarmGenome, mut rnd: &mut impl rand::Rng) -> SwarmGrammar {
        let mut uid_gen = crate::utils::UidGen::default();
        let (agents, artifacts) = genome.get_start(&mut rnd, &mut uid_gen);
        let mut world = ChunkedWorld::new(agents, genome.terrain_size, 10.0, uid_gen);
        world.insert_artifacts(artifacts);

        SwarmGrammar { genome, world }
    }
}
