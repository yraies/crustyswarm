use std::collections::HashMap;
use swarm::actor::*;

use cgmath::Vector2;

#[derive(Debug)]
struct World {
    config: WorldConfig,
    cells: HashMap<(i16, i16), WorldCell>,
    spacing: f32,
}

#[derive(Debug)]
struct WorldConfig {
    test: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct WorldCell {
    agents: Vec<Agent>,
    artifacts: Vec<Artifact>,
    buoys: Vec<Buoy>,
}

struct Accessor<T> {
    it: Iterator<Item = T>,
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
            .filter(|(&cell_pos, _)| self.include_cell(range, cell_pos, center_pos))
            .flat_map(|(_, cell)| cell.get_agents().iter());
        iter
    }

    fn include_cell(&self, range: f32, cell_pos: (i16, i16), center_pos: Vector2<f32>) -> bool {
        // improve filter!
        let new_range = range * 2.0;
        let (x_cell, y_cell) = (
            (cell_pos.0 as f32) * self.spacing,
            (cell_pos.1 as f32) * self.spacing,
        );
        let x_bool = (center_pos.x + new_range) > x_cell && (center_pos.x - new_range) < x_cell;
        let y_bool = (center_pos.y + new_range) > y_cell && (center_pos.y - new_range) < y_cell;
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
}

impl WorldCell {
    fn get_agents(&self) -> Vec<Agent> {
        self.agents
    }

    fn new(agent_capacity: usize, artifacts_capacity: usize, buoy_capacity: usize) -> WorldCell {
        WorldCell {
            agents: Vec::with_capacity(agent_capacity),
            artifacts: Vec::with_capacity(artifacts_capacity),
            buoys: Vec::with_capacity(buoy_capacity),
        }
    }
}
