extern crate fnv;

use self::fnv::FnvHashMap;
use swarm::actor::*;

use cgmath::Vector2;

type AgentIterBox<'a> = Box<dyn Iterator<Item = &'a Agent> + 'a>;
type ArtifactIterBox<'a> = Box<dyn Iterator<Item = &'a Artifact> + 'a>;

pub trait World {
    fn get_all_agents(&self) -> AgentIterBox;
    fn get_agents_at_least_within<'a>(
        &'a self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> AgentIterBox<'a>;
    fn get_all_artifacts(&self) -> ArtifactIterBox;
    fn get_artifacts_at_least_within<'a>(
        &'a self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> ArtifactIterBox<'a>;

    fn insert_agents(&mut self, new_agents: Vec<Agent>);
    fn insert_artifacts(&mut self, new_artifacts: Vec<Artifact>);
}

impl World for ChunkedWorld {
    fn get_all_agents(&self) -> AgentIterBox {
        Box::new(self.get_all_agents())
    }
    fn get_agents_at_least_within<'a>(
        &'a self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> AgentIterBox<'a> {
        Box::new(self.get_agents_at_least_within(range, center_pos))
    }

    fn get_all_artifacts(&self) -> ArtifactIterBox {
        Box::new(self.get_all_artifacts())
    }
    fn get_artifacts_at_least_within<'a>(
        &'a self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> ArtifactIterBox<'a> {
        Box::new(self.get_artifacts_at_least_within(range, center_pos))
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
}

#[derive(Debug)]
pub struct ChunkedWorld {
    config: WorldConfig,
    cells: FnvHashMap<(i16, i16), WorldCell>,
    spacing: f32,
}

#[derive(Debug)]
struct WorldConfig {
    test: String,
}

#[derive(PartialEq, Debug)]
struct WorldCell {
    agents: Vec<Agent>,
    artifacts: Vec<Artifact>,
    buoys: Vec<Buoy>,
}

impl ChunkedWorld {
    fn get_all_agents<'a>(&'a self) -> impl Iterator<Item = &'a Agent> + 'a {
        self.cells
            .iter()
            .flat_map(|(_, cell)| cell.get_agents().iter())
    }
    fn get_all_artifacts<'a>(&'a self) -> impl Iterator<Item = &'a Artifact> + 'a {
        self.cells
            .iter()
            .flat_map(|(_, cell)| cell.get_artifacts().iter())
    }

    fn get_agents_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Agent> {
        self.cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.get_agents().iter())
    }
    fn get_artifacts_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Artifact> {
        self.cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.get_artifacts().iter())
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
            .cells
            .entry((x_coord, y_coord))
            .or_insert_with(|| WorldCell::new(4, 4, 4));
        cell.agents.push(agent);
    }

    fn insert_artifact(&mut self, artifact: Artifact) {
        let x_coord = (artifact.position.x / self.spacing).floor() as i16;
        let y_coord = (artifact.position.y / self.spacing).floor() as i16;

        let cell = self
            .cells
            .entry((x_coord, y_coord))
            .or_insert_with(|| WorldCell::new(4, 4, 4));
        cell.artifacts.push(artifact);
    }

    pub fn new(agents: Vec<Agent>, spacing: f32) -> ChunkedWorld {
        let mut world = ChunkedWorld {
            spacing,
            config: WorldConfig {
                test: String::from("tesa"),
            },
            cells: FnvHashMap::default(),
        };

        agents.into_iter().for_each(|ag| world.insert_agent(ag));

        world
    }
}

impl WorldCell {
    fn get_agents(&self) -> &Vec<Agent> {
        &self.agents
    }
    fn get_artifacts(&self) -> &Vec<Artifact> {
        &self.artifacts
    }

    fn new(agent_capacity: usize, artifacts_capacity: usize, buoy_capacity: usize) -> WorldCell {
        WorldCell {
            agents: Vec::with_capacity(agent_capacity),
            artifacts: Vec::with_capacity(artifacts_capacity),
            buoys: Vec::with_capacity(buoy_capacity),
        }
    }
}
