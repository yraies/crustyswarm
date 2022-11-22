extern crate fnv;

use self::fnv::FnvHashMap;
use cgmath::{MetricSpace, Vector2, Vector3};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, ops::Deref};

use crate::{
    swarm::{
        actor::*,
        genome::{SurroundingIndex, SwarmGenome},
    },
    utils::UidGen,
};

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

    fn set_agents(&mut self, new_agents: Vec<Agent>);
    fn set_artifacts(&mut self, new_artifacts: Vec<Artifact>);

    fn get_agent_count(&self) -> usize;
    fn get_artifact_count(&self) -> usize;
    fn get_buoy_count(&self) -> usize;

    fn update_terrain(&mut self, influences: (&[(f32, f32)], &[(f32, f32)]));
    fn get_height(&self, agent: &Agent) -> f32;
    fn get_height_at(&self, x: f32, z: f32) -> f32;
    fn get_gradient_and_normal(&self, xpos: f32, zpos: f32) -> (Vector3<f32>, Vector3<f32>);
    fn get_slope(&self, xpos: f32, zpos: f32, gradient: Vector3<f32>) -> Vector3<f32>;

    fn get_size(&self) -> (usize, usize, f32);
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
            let threshold = rnd.gen_range(0.0, weight_sum + f32::EPSILON);

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
        let _xz = Vector2::new(center_pos.x, center_pos.z);

        let mut agents: Vec<(f32, Actor)> = self
            .get_all_agents()
            .map(|agent| {
                (
                    MetricSpace::distance(center_pos, agent.position),
                    agent.clone().into(),
                )
            })
            .filter(|(dist, _actor)| dist < &range)
            .collect();

        let mut artifacts: Vec<(f32, Actor)> = self
            .get_all_artifacts()
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

    fn set_agents(&mut self, new_agents: Vec<Agent>) {
        self.delete_agents();
        self.insert_agents(new_agents);
    }
    fn set_artifacts(&mut self, new_artifacts: Vec<Artifact>) {
        self.delete_artifacts();
        self.insert_artifacts(new_artifacts);
    }

    fn get_agent_count(&self) -> usize {
        self.agent_count
    }
    fn get_artifact_count(&self) -> usize {
        self.artifact_count
    }
    fn get_buoy_count(&self) -> usize {
        self.terrain.sample_points.len() * self.terrain.sample_points[0].len()
    }

    fn update_terrain(&mut self, influences: (&[(f32, f32)], &[(f32, f32)])) {
        use super::actor::*;

        fn update_buoy<'a>(
            actors: &[Actor],
            b: &mut Buoy,
            _spacing: f32,
            influences: (&[(f32, f32)], &[(f32, f32)]),
        ) {
            let mut influecers = 0.0;
            let mut avg_ydist = 0.0;

            let bpos = Vector2::new(b.position.x, b.position.z);

            for other in actors {
                let (otherpos, (influence_factor, influence_weight)) = match other {
                    Actor::Agent(ag) => (ag.position, influences.0[ag.species_index.0]),
                    Actor::Artifact(art) => (art.position, influences.1[art.artifact_index.0]),
                };

                if influence_weight == 0.0 {
                    continue;
                }

                let otherpos2d = Vector2::new(otherpos.x, otherpos.z);
                let xzdist = bpos.distance(otherpos2d);
                let influence = influence_weight / (1.0 + xzdist).powf(influence_factor);
                let ydist = otherpos.y - b.position.y;

                if !(influence.is_nan() || ydist.is_nan()) {
                    influecers += influence;
                    avg_ydist += ydist * influence;
                }
            }

            let vel = if avg_ydist == 0.0 {
                0.0
            } else {
                avg_ydist / influecers
            };

            b.position.y += vel;
        }

        let spacing = self.terrain.spacing;
        let buoys: Vec<&mut Buoy> = self
            .terrain
            .sample_points
            .iter_mut()
            .flat_map(|v| v.iter_mut())
            .collect();
        let agents = self
            .agent_cells
            .values()
            .flat_map(|cell| cell.iter())
            .filter(|ag| influences.0[ag.species_index.0].1 != 0.0)
            .map(|ag| Actor::Agent(ag.clone()));
        let all_actors: Vec<Actor> = self
            .artifact_cells
            .values()
            .flat_map(|cell| cell.iter())
            .filter(|art| influences.1[art.artifact_index.0].1 != 0.0)
            .map(|art| Actor::Artifact(art.clone()))
            .chain(agents)
            .collect();

        buoys
            .into_par_iter()
            .for_each(|b| update_buoy(&all_actors, b, spacing, influences));
    }

    fn get_height(&self, agent: &Agent) -> f32 {
        agent.position.y - self.terrain.get_height(agent.position.x, agent.position.z)
    }

    fn get_height_at(&self, x: f32, z: f32) -> f32 {
        self.terrain.get_height(x, z)
    }

    fn get_gradient_and_normal(&self, xpos: f32, zpos: f32) -> (Vector3<f32>, Vector3<f32>) {
        self.terrain.get_gradient_and_normal(xpos, zpos)
    }
    fn get_slope(&self, xpos: f32, zpos: f32, gradient: Vector3<f32>) -> Vector3<f32> {
        self.terrain.get_slope(xpos, zpos, gradient)
    }

    fn get_size(&self) -> (usize, usize, f32) {
        (
            self.terrain.x_size,
            self.terrain.z_size,
            self.terrain.spacing,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Terrain {
    sample_points: Vec<Vec<Buoy>>,
    spacing: f32,
    x_size: usize,
    z_size: usize,
}

impl Terrain {
    fn translate(pos: i64) -> usize {
        use std::convert::TryInto;
        match pos {
            0 => 0,
            _ if pos.is_negative() => (pos * -2).try_into().unwrap(),
            _ => (pos * 2 - 1).try_into().unwrap(),
        }
    }

    fn translate_back(pos: usize) -> i64 {
        match pos {
            0 => 0,
            _ if (pos % 2 == 0) => -((pos / 2) as i64),
            _ => ((pos + 1) / 2) as i64,
        }
    }

    fn new(size: usize, spacing: f32) -> Self {
        let mut sample_points = Vec::with_capacity(size);
        for x in 0..size {
            sample_points.push(Vec::with_capacity(size));
            for z in 0..size {
                sample_points[x].push(Buoy::new(
                    Vector3::new(
                        Terrain::translate_back(x) as f32 * spacing,
                        0.0,
                        Terrain::translate_back(z) as f32 * spacing,
                    ),
                    0.0,
                    0.0,
                ));
            }
        }
        Terrain {
            sample_points,
            spacing,
            x_size: size,
            z_size: size,
        }
    }

    fn get_slope(&self, xpos: f32, zpos: f32, gradient: Vector3<f32>) -> Vector3<f32> {
        use cgmath::prelude::*;

        if gradient.eq(&Vector3::zero()) {
            return gradient;
        }

        let norm_grad = gradient.normalize() * self.spacing * 0.5;

        let h1 = self.get_height(xpos + norm_grad.x, zpos + norm_grad.z);
        let h2 = self.get_height(xpos - norm_grad.x, zpos - norm_grad.z);

        let diff = h2 - h1;

        if diff >= 0.0 {
            Vector3::new(gradient.x, -diff, gradient.z)
        } else {
            -Vector3::new(gradient.x, -diff, gradient.z)
        }
    }

    fn get_gradient_and_normal(&self, xpos: f32, zpos: f32) -> (Vector3<f32>, Vector3<f32>) {
        let offset = self.spacing * 0.5;

        let diff_x = self.get_height(xpos + offset, zpos) - self.get_height(xpos - offset, zpos);
        let diff_z = self.get_height(xpos, zpos + offset) - self.get_height(xpos, zpos - offset);

        let gradient = Vector3::new(diff_x, 0.0, diff_z);
        let normal = Vector3::new(0.0, diff_z, 1.0).cross(Vector3::new(1.0, diff_x, 0.0));
        (gradient, normal)
    }

    fn get_height(&self, xpos: f32, zpos: f32) -> f32 {
        fn lerp(a: f32, b: f32, fract: f32) -> f32 {
            let ax = a as f64;
            let bx = b as f64;
            let fractx = fract as f64;
            let result = if fract.is_sign_positive() {
                ax * (1.0 - fractx) + bx * fractx
            } else {
                ax * -fractx + bx * (1.0 + fractx)
            };
            result as f32
        }

        //dbg!(xpos);
        let x_grid = xpos / self.spacing;
        let x_low = x_grid.floor() as i64;
        let x_high = x_grid.ceil() as i64;

        let z_grid = zpos / self.spacing;
        let z_low = z_grid.floor() as i64;
        let z_high = z_grid.ceil() as i64;

        let left_height = lerp(
            self.get_height_on_grid(x_low, z_low),
            self.get_height_on_grid(x_high, z_low),
            x_grid.fract(),
        );
        let right_height = lerp(
            self.get_height_on_grid(x_low, z_high),
            self.get_height_on_grid(x_high, z_high),
            x_grid.fract(),
        );

        let fract = z_grid.fract();
        let result = lerp(left_height, right_height, fract);

        if result > left_height.max(right_height) || result < left_height.min(right_height) {
            dbg!(x_grid);
            dbg!(z_grid);
            dbg!(left_height);
            dbg!(right_height);
            dbg!(fract);
            dbg!(result);
            dbg!("--");
        }

        result
    }

    fn get_height_on_grid(&self, xpos: i64, zpos: i64) -> f32 {
        let xindex = Terrain::translate(xpos);
        let zindex = Terrain::translate(zpos);

        let xindex_clamped = if xindex < self.x_size {
            xindex
        } else if xindex % 2 == 0 {
            self.x_size - 1
        } else {
            self.x_size - 2
        };

        let zindex_clamped = if zindex < self.z_size {
            zindex
        } else if zindex % 2 == 0 {
            self.z_size - 1
        } else {
            self.z_size - 2
        };

        self.sample_points[xindex_clamped][zindex_clamped]
            .position
            .y
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkedWorld {
    agent_cells: FnvHashMap<Coord, Vec<Agent>>,
    artifact_cells: FnvHashMap<Coord, Vec<Artifact>>,
    terrain: Terrain,
    agent_count: usize,
    artifact_count: usize,
    buoy_count: usize,
    spacing: f32,
    uid_gen: UidGen,
}

use rayon::prelude::*;
impl ChunkedWorld {
    fn get_all_agents<'a>(&'a self) -> impl Iterator<Item = &'a Agent> + 'a {
        self.agent_cells.iter().flat_map(|(_, cell)| cell.iter())
    }
    fn get_all_artifacts<'a>(&'a self) -> impl Iterator<Item = &'a Artifact> + 'a {
        self.artifact_cells.iter().flat_map(|(_, cell)| cell.iter())
    }
    fn get_all_buoys<'a>(&'a self) -> impl Iterator<Item = &'a Buoy> + 'a {
        self.terrain.sample_points.iter().flat_map(|v| v.iter())
    }

    pub fn get_agents_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Agent> {
        self.agent_cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, *cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.iter())
    }
    #[allow(dead_code)]
    fn get_artifacts_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Artifact> {
        self.artifact_cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, *cell_pos, center_pos))
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
            .entry((x_coord, y_coord).into())
            .or_insert_with(Vec::new);
        cell.push(agent);
        self.agent_count += 1;
    }

    fn insert_artifact(&mut self, artifact: Artifact) {
        let x_coord = (artifact.position.x / self.spacing).floor() as i16;
        let y_coord = (artifact.position.y / self.spacing).floor() as i16;

        let cell = self
            .artifact_cells
            .entry((x_coord, y_coord).into())
            .or_insert_with(Vec::new);
        cell.push(artifact);
        self.artifact_count += 1;
    }
    fn delete_agents(&mut self) {
        self.agent_cells = FnvHashMap::default();
        self.agent_count = 0;
    }
    fn delete_artifacts(&mut self) {
        self.artifact_cells = FnvHashMap::default();
        self.artifact_count = 0;
    }

    pub fn new(agents: Vec<Agent>, size: usize, spacing: f32, uid_gen: UidGen) -> ChunkedWorld {
        let mut world = ChunkedWorld {
            spacing,
            agent_cells: FnvHashMap::default(),
            artifact_cells: FnvHashMap::default(),
            terrain: Terrain::new(size, spacing),
            artifact_count: 0,
            agent_count: 0,
            buoy_count: 0,
            uid_gen,
        };

        agents.into_iter().for_each(|ag| world.insert_agent(ag));

        world
    }
}

#[test]
fn test_terrain_map() {
    for i in -100..100 {
        assert_eq!(i, Terrain::translate_back(Terrain::translate(i)));
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Coord((i16, i16));

impl From<(i16, i16)> for Coord {
    fn from(coord: (i16, i16)) -> Self {
        Coord(coord)
    }
}
impl Deref for Coord {
    type Target = (i16, i16);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Coord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("({},{})", self.0 .0, self.0 .1))
    }
}

impl<'de> Deserialize<'de> for Coord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let foo = deserializer.deserialize_str(super::genome::StrVisitor)?;
        let mut ints = foo
            .trim()
            .strip_prefix("(")
            .unwrap()
            .strip_suffix(")")
            .unwrap()
            .split(",");
        let x = ints.next().unwrap().parse::<i16>().unwrap();
        let y = ints.next().unwrap().parse::<i16>().unwrap();

        Ok(Coord((x, y)))
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = Deserialize::deserialize(deserializer)?;
        Ok(())
    }
}
