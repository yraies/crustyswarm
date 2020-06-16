extern crate fnv;

use std::cmp::Ordering;

use self::fnv::FnvHashMap;
use crate::utils::UidGen;
use serde::{Deserialize, Serialize};
use swarm::actor::*;
use swarm::genome::SurroundingIndex;
use swarm::genome::SwarmGenome;

use cgmath::{MetricSpace, Vector2, Vector3};
use rand::Rng;

type AgentIterBox<'a> = Box<dyn Iterator<Item = &'a Agent> + 'a>;
type ArtifactIterBox<'a> = Box<dyn Iterator<Item = &'a Artifact> + 'a>;
type BuoyIterBox<'a> = Box<dyn Iterator<Item = &'a Buoy> + 'a>;

pub trait World {
    fn replace_by(&mut self, genome: &SwarmGenome, rnd: &mut impl Rng);

    fn get_uid_gen(&mut self) -> &mut UidGen;

    fn get_context_within(&self, range: f32, center_pos: Vector3<f32>) -> Vec<(f32, Actor)>;

    fn get_all_agents(&self) -> AgentIterBox;
    fn get_all_artifacts(&self) -> ArtifactIterBox;
    fn get_all_buoys(&self) -> BuoyIterBox;

    fn insert_agents(&mut self, new_agents: Vec<Agent>);
    fn insert_artifacts(&mut self, new_artifacts: Vec<Artifact>);
    fn insert_buoys(&mut self, new_buoys: Vec<Buoy>);

    fn set_agents(&mut self, new_agents: Vec<Agent>);
    fn set_artifacts(&mut self, new_artifacts: Vec<Artifact>);
    fn set_buoys(&mut self, new_buoys: Vec<Buoy>);

    fn get_agent_count(&self) -> usize;
    fn get_artifact_count(&self) -> usize;
    fn get_buoy_count(&self) -> usize;

    fn update_terrain(&mut self);
    fn get_height(&self, agent: &Agent) -> f32;
}

impl World for ChunkedWorld {
    fn replace_by(&mut self, genome: &SwarmGenome, rnd: &mut impl Rng) {
        if !genome.strategy.should_replace() {
            return;
        }

        let mut new_agents: Vec<Agent> = Vec::with_capacity(self.get_agent_count());
        let mut new_artifacts: Vec<Artifact> = Vec::with_capacity(self.get_artifact_count());

        let mut uid_gen = self.uid_gen.clone();

        self.get_all_agents().for_each(|agent: &Agent| {
            let rules = genome.get_rules(&agent.species_index);

            let max_range = rules
                .iter()
                .max_by(|a, b| a.range.partial_cmp(&b.range).unwrap_or(Ordering::Equal))
                .map(|rule| rule.range);

            let context: Vec<(f32, SurroundingIndex)> = if let Some(range) = max_range {
                self.get_context_within(range, agent.position)
                    .into_iter()
                    .map(|(d, act)| (d, act.into()))
                    .collect()
            } else {
                vec![]
            };

            let applicable_rules: Vec<_> = rules
                .iter()
                .filter(|rule| rule.is_applicable(&context))
                .collect();

            let weight_sum: f32 = applicable_rules.iter().map(|rule| rule.weight).sum();
            let threshold = rnd.gen_range(0.0, weight_sum);

            let mut gauge = 0.0;
            for rule in applicable_rules {
                gauge += rule.weight;
                if gauge < threshold {
                    continue;
                }

                let (mut new_ag, mut new_art) = rule.replace_agent(agent, genome, &mut uid_gen);

                new_agents.append(&mut new_ag);
                new_artifacts.append(&mut new_art);

                break;
            }
        });

        self.uid_gen = uid_gen;

        self.delete_agents();
        self.insert_agents(new_agents);
        self.insert_artifacts(new_artifacts);
    }

    fn get_uid_gen(&mut self) -> &mut UidGen {
        &mut self.uid_gen
    }

    fn get_context_within(&self, range: f32, center_pos: Vector3<f32>) -> Vec<(f32, Actor)> {
        let xz = Vector2::new(center_pos.x, center_pos.z);

        let mut agents: Vec<(f32, Actor)> = self
            .get_agents_at_least_within(range, xz)
            .map(|agent| {
                (
                    MetricSpace::distance(center_pos, agent.position),
                    agent.clone().into(),
                )
            })
            .filter(|(dist, _actor)| dist < &range)
            .collect();

        let mut artifacts: Vec<(f32, Actor)> = self
            .get_artifacts_at_least_within(range, xz)
            .map(|artifact| {
                (
                    MetricSpace::distance(center_pos, artifact.position),
                    artifact.clone().into(),
                )
            })
            .filter(|(dist, _art)| dist < &range)
            .collect();

        agents.append(&mut artifacts);

        agents
    }

    fn get_all_agents(&self) -> AgentIterBox {
        Box::new(self.get_all_agents())
    }

    fn get_all_artifacts(&self) -> ArtifactIterBox {
        Box::new(self.get_all_artifacts())
    }

    fn get_all_buoys(&self) -> BuoyIterBox {
        Box::new(self.get_all_buoys())
    }

    fn insert_agents(&mut self, new_agents: Vec<Agent>) {
        for agent in new_agents {
            self.insert_agent(agent);
        }
    }
    fn insert_artifacts(&mut self, new_artifacts: Vec<Artifact>) {
        for artifact in new_artifacts {
            self.insert_artifact(artifact);
        }
    }
    fn insert_buoys(&mut self, new_buoys: Vec<Buoy>) {
        for buoy in new_buoys {
            self.insert_buoy(buoy);
        }
    }

    fn set_agents(&mut self, new_agents: Vec<Agent>) {
        self.delete_agents();
        self.insert_agents(new_agents);
    }
    fn set_artifacts(&mut self, new_artifacts: Vec<Artifact>) {
        self.delete_artifacts();
        self.insert_artifacts(new_artifacts);
    }
    fn set_buoys(&mut self, new_buoys: Vec<Buoy>) {
        self.delete_buoys();
        self.insert_buoys(new_buoys);
    }

    fn get_agent_count(&self) -> usize {
        self.agent_count
    }
    fn get_artifact_count(&self) -> usize {
        self.artifact_count
    }
    fn get_buoy_count(&self) -> usize {
        self.buoy_count
    }

    fn update_terrain(&mut self) {
        let buoys: Vec<&mut Buoy> = self
            .buoy_cells
            .values_mut()
            .flat_map(|cell| cell.iter_mut())
            .collect();
        for b in buoys {
            let mut factors = 0.5;
            let mut d = if b.position.y < 0.0 { 0.1 } else { -0.1 };

            for a in self.agent_cells.values().flat_map(|cell| cell.iter()) {
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
        }
    }
    fn get_height(&self, agent: &Agent) -> f32 {
        agent.position.y - agent.seed_center.y
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkedWorld {
    agent_cells: FnvHashMap<(i16, i16), Vec<Agent>>,
    artifact_cells: FnvHashMap<(i16, i16), Vec<Artifact>>,
    buoy_cells: FnvHashMap<(i16, i16), Vec<Buoy>>,
    agent_count: usize,
    artifact_count: usize,
    buoy_count: usize,
    spacing: f32,
    uid_gen: UidGen,
}

impl ChunkedWorld {
    fn get_all_agents<'a>(&'a self) -> impl Iterator<Item = &'a Agent> + 'a {
        self.agent_cells.iter().flat_map(|(_, cell)| cell.iter())
    }
    fn get_all_artifacts<'a>(&'a self) -> impl Iterator<Item = &'a Artifact> + 'a {
        self.artifact_cells.iter().flat_map(|(_, cell)| cell.iter())
    }
    fn get_all_buoys<'a>(&'a self) -> impl Iterator<Item = &'a Buoy> + 'a {
        self.buoy_cells.iter().flat_map(|(_, cell)| cell.iter())
    }

    pub fn get_agents_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Agent> {
        self.agent_cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.iter())
    }
    fn get_artifacts_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Artifact> {
        self.artifact_cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.iter())
    }

    fn is_cell_included(&self, range: f32, cell_pos: (i16, i16), center_pos: Vector2<f32>) -> bool {
        // improve filter!
        let cell_range = (range / self.spacing).ceil() as i16;

        let x_up = (center_pos.x / self.spacing).floor() as i16 + cell_range;
        let x_low = (center_pos.x / self.spacing).floor() as i16 - cell_range;
        let y_up = (center_pos.y / self.spacing).floor() as i16 + cell_range;
        let y_low = (center_pos.y / self.spacing).floor() as i16 - cell_range;

        let x_bool = x_low <= cell_pos.0 && cell_pos.0 <= x_up;
        let y_bool = y_low <= cell_pos.1 && cell_pos.1 <= y_up;
        x_bool && y_bool
    }

    fn insert_agent(&mut self, agent: Agent) {
        let x_coord = (agent.position.x / self.spacing).floor() as i16;
        let y_coord = (agent.position.y / self.spacing).floor() as i16;

        let cell = self
            .agent_cells
            .entry((x_coord, y_coord))
            .or_insert_with(Vec::new);
        cell.push(agent);
        self.agent_count += 1;
    }

    fn insert_artifact(&mut self, artifact: Artifact) {
        let x_coord = (artifact.position.x / self.spacing).floor() as i16;
        let y_coord = (artifact.position.y / self.spacing).floor() as i16;

        let cell = self
            .artifact_cells
            .entry((x_coord, y_coord))
            .or_insert_with(Vec::new);
        cell.push(artifact);
        self.artifact_count += 1;
    }
    fn insert_buoy(&mut self, buoy: Buoy) {
        let x_coord = (buoy.position.x / self.spacing).floor() as i16;
        let y_coord = (buoy.position.y / self.spacing).floor() as i16;

        let cell = self
            .buoy_cells
            .entry((x_coord, y_coord))
            .or_insert_with(Vec::new);
        cell.push(buoy);
        self.buoy_count += 1;
    }

    fn delete_agents(&mut self) {
        self.agent_cells = FnvHashMap::default();
        self.agent_count = 0;
    }
    fn delete_artifacts(&mut self) {
        self.artifact_cells = FnvHashMap::default();
        self.artifact_count = 0;
    }
    fn delete_buoys(&mut self) {
        self.buoy_cells = FnvHashMap::default();
        self.buoy_count = 0;
    }

    pub fn new(agents: Vec<Agent>, spacing: f32, uid_gen: UidGen) -> ChunkedWorld {
        let mut world = ChunkedWorld {
            spacing,
            agent_cells: FnvHashMap::default(),
            artifact_cells: FnvHashMap::default(),
            buoy_cells: FnvHashMap::default(),
            artifact_count: 0,
            agent_count: 0,
            buoy_count: 0,
            uid_gen,
        };

        agents.into_iter().for_each(|ag| world.insert_agent(ag));

        world
    }
}
