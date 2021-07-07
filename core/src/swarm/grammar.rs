use std::time::Instant;

use cgmath::prelude::*;
use cgmath::Vector3;
use cgmath::{Deg, Rad};
use rand::Rng;
use rayon::prelude::*;

use super::actor::Agent;
use crate::swarm::genome::SwarmGenome;
use crate::{
    swarm::world::{ChunkedWorld, World},
    utils::*,
};

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

        let mut printfoo: Vec<String> = vec![format!(
            "{:5} Agents {:5} Artifacts:",
            self.world.get_all_agents().count(),
            self.world.get_all_artifacts().count()
        )];

        // 1. Replace by Rules          -------------------------------------
        self.genome.tick();
        let mut start = Instant::now();
        self.world.replace_by(&self.genome, rnd);
        printfoo.push(format!(
            "replacement {:>7} ",
            format!("{:.1?}", start.elapsed())
        ));

        // 2. Recalculate Velocities    -------------------------------------
        start = Instant::now();
        self.recalc_agent(rnd);
        printfoo.push(format!("recalc {:>7} ", format!("{:.1?}", start.elapsed())));

        // 3. Recalculate Buoys         -------------------------------------
        start = Instant::now();
        self.world.update_terrain((
            &self.genome.terrain_influences.0,
            &self.genome.terrain_influences.1,
        ));
        printfoo.push(format!(
            "buoys rec {:>7} ",
            format!("{:.1?}", start.elapsed())
        ));
        let pr = printfoo.join(" ");
        println!("{}", pr);
    }

    pub fn recalc_agent(&mut self, rnd: &mut impl Rng) {
        let agent_random_pairs: Vec<_> = self
            .world
            .get_all_agents()
            .map(|a| (random_one(rnd), a))
            .collect();
        let mut recalculated: Vec<Agent> = agent_random_pairs
            .par_iter()
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
                            let d = agent.position - other.get_position();
                            let rev = d * (agent_species.sep_distance - d.magnitude());
                            sep_vec += rev * influence;
                            sep_counter += 1.0 * influence.abs();
                        }

                        let solid_angle =
                            agent.velocity.angle(other.get_position() - agent.position);

                        if solid_angle > Rad::from(Deg(agent_species.view_angle)) {
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

        let sep_norm = safe_devide_mean(sep_vec, sep_counter);

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
        let floor = -Vector3::<f32>::unit_y() * (base_dist * base_dist);

        let (gradient, normal) = self
            .world
            .get_gradient_and_normal(agent.position.x, agent.position.z);

        let slope = self
            .world
            .get_slope(agent.position.x, agent.position.z, gradient);

        // 2.2. Actually Recalculate    ------------------

        let mut acceleration = (agent_species.separation * sep_norm)
            + (agent_species.alignment * ali_norm)
            + (agent_species.cohesion * coh_norm)
            + (agent_species.center * cen_norm)
            + (agent_species.randomness * rnd_norm)
            + (agent_species.floor * floor)
            + (agent_species.gradient * gradient)
            + (agent_species.slope * slope)
            + (agent_species.normal * normal)
            + agent_species.bias;

        acceleration =
            cgmath::ElementWise::mul_element_wise(acceleration, agent_species.axis_constraint);

        acceleration = crate::utils::clip(acceleration, agent_species.max_acceleration);

        let mut new_velocity = agent.velocity + acceleration;
        new_velocity = crate::utils::clip(new_velocity, agent_species.max_speed);

        if agent_species.pacekeeping > 0.0 {
            new_velocity = agent_species.pacekeeping
                * new_velocity.normalize_to(agent_species.normal_speed)
                + (1.0 - agent_species.pacekeeping) * new_velocity;
        }

        let new_position = agent.position + new_velocity;

        let new_floor = self.world.get_height_at(new_position.x, new_position.z);
        let clipped_new_position = if !agent_species.noclip && new_floor > new_position.y {
            Vector3::new(new_position.x, new_floor, new_position.z)
        } else {
            new_position
        };

        let mut out_agent = agent.clone();

        out_agent.velocity = new_velocity;
        out_agent.position = clipped_new_position;
        out_agent.energy -= agent_species
            .energy
            .on_movement
            .get(agent.velocity.magnitude());
        assert!(
            out_agent.energy.is_finite(),
            "Calculated energy not finite! {:?}=>{}",
            out_agent.velocity,
            out_agent.energy
        );
        out_agent
    }

    pub fn get_world(&self) -> &ChunkedWorld {
        &self.world
    }

    pub fn from(genome: SwarmGenome, mut rnd: &mut impl rand::Rng) -> SwarmGrammar {
        let mut uid_gen = crate::utils::UidGen::default();
        let (agents, artifacts) = genome.get_start(&mut rnd, &mut uid_gen);
        let mut world =
            ChunkedWorld::new(agents, genome.terrain_size, genome.terrain_spacing, uid_gen);
        world.insert_artifacts(artifacts);

        SwarmGrammar { genome, world }
    }
}
