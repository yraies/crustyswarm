extern crate fnv;

use self::fnv::FnvHashMap;
use std::collections::HashMap;
use swarm::actor::*;

use cgmath::Vector2;

#[derive(Debug)]
pub struct World {
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

impl World {
    pub fn get_all_agents(&self) -> impl Iterator<Item = &Agent> {
        self.cells
            .iter()
            .flat_map(|(_, cell)| cell.get_agents().iter())
    }

    pub fn get_agents_at_least_within(
        &self,
        range: f32,
        center_pos: Vector2<f32>,
    ) -> impl Iterator<Item = &Agent> {
        let iter = self
            .cells
            .iter()
            .filter(move |(&cell_pos, _)| self.is_cell_included(range, cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.get_agents().iter());
        iter
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
            .or_insert(WorldCell::new(4, 4, 4));
        cell.agents.push(agent);
    }

    pub fn new(agents: Vec<Agent>, spacing: f32) -> World {
        let mut world = World {
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

    fn new(agent_capacity: usize, artifacts_capacity: usize, buoy_capacity: usize) -> WorldCell {
        WorldCell {
            agents: Vec::with_capacity(agent_capacity),
            artifacts: Vec::with_capacity(artifacts_capacity),
            buoys: Vec::with_capacity(buoy_capacity),
        }
    }
}
