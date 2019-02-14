use super::Val;

#[derive(Debug)]
pub struct Species {
    pub separation: Val,
    pub alignment: Val,
    pub cohesion: Val,
    pub randomness: Val,
    pub center: Val,
    pub max_speed: Val,
    pub view_distance: Val,
    pub sep_distance: Val,
}

impl Species {
    pub fn new(separation: Val, alignment: Val, cohesion: Val, randomness: Val, center: Val, max_speed: Val) -> Species {
        Species { separation, alignment, cohesion, randomness, center, max_speed, view_distance: 80.0, sep_distance: 7.0 }
    }
}