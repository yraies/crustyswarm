use crustswarm::swarm::world::World;
use crustswarm_lib as crustswarm;

use std::fs::File;
use std::io::Write;
use std::ops::Add;
use std::string::String;
use std::time::{Duration, SystemTime};

use circular_queue::CircularQueue;

use tempfile::TempDir;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use raylib::prelude::*;

fn main() {
    let mut args = std::env::args();
    let configfile = args
        .nth(1)
        .unwrap_or_else(|| String::from("swarm_config.json"));
    println!("Using config: {}", &configfile);

    let tmp_dir = TempDir::new().expect("TempDir could not be created");
    let fontfile_path = tmp_dir.path().join("font.ttf");
    let mut fontfile = File::create(&fontfile_path).expect("Font-Tempfile could not be created");
    fontfile
        .write_all(include_bytes!("../joystix/joystix monospace.ttf"))
        //.write_all(include_bytes!("../monofonto/monofonto.ttf"))
        //.write_all(include_bytes!("../anonymous-pro/AnonymousPro-Regular.ttf"))
        .unwrap();

    let (mut rl, thread) = raylib::init()
        .size(1270, 720)
        .title("Hello World this is window speaking")
        .msaa_4x()
        .resizable()
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(-105.0, 1.0, -105.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    rl.set_camera_mode(&camera, CameraMode::CAMERA_THIRD_PERSON);
    rl.update_camera(&mut camera);
    rl.set_target_fps(30);

    let seed = if false {
        let rnd = rand::random::<u64>();
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .append(true)
            .open("last_seed")
            .unwrap()
            .write_all(
                &format!(
                    "{:?} {}\n",
                    SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    &rnd
                )
                .into_bytes(),
            )
            .unwrap();

        rnd
    } else {
        15770484348065534092u64
    };
    println!("seed: {}", seed);

    let mut rnd: SmallRng = SmallRng::seed_from_u64(seed);
    let mut sg = {
        let temp = crustswarm::io::genome_from_file(configfile);
        crustswarm::swarm::grammar::SwarmGrammar::from(temp, &mut rnd)
    };

    let mut render_stats = VizStats::new();
    let mut sim_stats = VizStats::new();
    let mut calc_next = false;

    let mut orbit = false;
    let orbit_speed = 0.01;

    let mut conditionals_draws = ConditionalDraw::All;

    let font = rl
        .load_font(&thread, fontfile_path.to_str().unwrap())
        .expect("Could not load font");

    while !rl.window_should_close() {
        // Update World State
        {
            if calc_next {
                sim_stats.start();
                sg.step(&mut rnd);
                sim_stats.stop();
            }
        }

        // Handle Inputs
        {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                calc_next = !calc_next;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_B) {
                conditionals_draws = conditionals_draws.next();
            }

            if rl.is_key_pressed(KeyboardKey::KEY_O) {
                if orbit {
                    orbit = false;
                } else {
                    orbit = true;
                }
            }

            if !orbit {
                let old_position = camera.position;
                rl.update_camera(&mut camera);
                let camera_movement = camera.position - old_position;
                let leftshift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                let leftalt = rl.is_key_down(KeyboardKey::KEY_LEFT_ALT);
                if leftshift && leftalt {
                    camera.position += camera_movement * 5.0;
                    camera.target += camera_movement * 5.0;
                } else if leftshift || leftalt {
                    camera.position += camera_movement * 100.0;
                    camera.target += camera_movement * 100.0;
                } else {
                    camera.position += camera_movement * 25.0;
                    camera.target += camera_movement * 20.0;
                }
            } else {
                rl.update_camera(&mut camera);
                camera
                    .position
                    .rotate(dbg!(Quaternion::from_euler(0.0, orbit_speed, 0.0)));
                camera.target = Vector3::zero();
            }
        }

        // Draw the Scene
        {
            render_stats.start();
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::BLACK);

            // Draw 3D Stuff
            {
                let mut d3d = d.begin_mode_3D(camera);
                d3d.draw_cube(Vector3::new(1.0, 0.0, 0.0), 2.5, 0.5, 0.5, Color::RED);
                d3d.draw_cube(Vector3::new(0.0, 1.0, 0.0), 0.5, 2.5, 0.5, Color::GREEN);
                d3d.draw_cube(Vector3::new(0.0, 0.0, 1.0), 0.5, 0.5, 2.5, Color::BLUE);

                if !conditionals_draws.draw_buoys() {
                    d3d.draw_grid(10, 10.0);
                }

                if conditionals_draws.draw_agents() {
                    let agents = crustswarm::agents_to_arr2(&sg);
                    for (pos, color_index) in agents {
                        d3d.draw_cube(
                            Vector3::new(pos[0], pos[1], pos[2]),
                            1.0,
                            1.0,
                            1.0,
                            get_color(color_index),
                        );
                    }
                }

                let artifacts = crustswarm::artifacts_to_arr2(&sg);
                for (pos, color_index) in artifacts {
                    d3d.draw_cube(
                        Vector3::new(pos[0], pos[1], pos[2]),
                        1.0,
                        1.0,
                        1.0,
                        get_color(color_index),
                    );
                }

                if conditionals_draws.draw_buoys() {
                    let buoys = crustswarm::buoys_to_arr2(&sg);
                    for pos in buoys {
                        d3d.draw_cube(
                            Vector3::new(pos[0], pos[1], pos[2]),
                            1.0,
                            0.2,
                            1.0,
                            get_color(99),
                        );
                    }
                }
            }

            // Draw UI Stuff
            {
                let stat_info = format!(
                    "FPS: {:2}\n{}\n{}\nAgents: {:4}\nArts:   {:4}\nBuoys:  {:4}",
                    d.get_fps(),
                    render_stats.to_string("Render", 6, false),
                    sim_stats.to_string("Sim", 6, false),
                    sg.world.get_agent_count(),
                    sg.world.get_artifact_count(),
                    sg.world.get_buoy_count(),
                );
                draw_text(&mut d, &font, 5, 5, &stat_info);

                //d.draw_text("", 20, 30, 10, Color::DARKGRAY);
                d.draw_text(
                    "Controls:
- Mouse to look around
- WASD to move around
- EQ to move up and down
- Shift to increase movement speed",
                    10,
                    d.get_screen_height() - 100,
                    10,
                    Color::GRAY,
                );
                d.draw_text(
                    &format!(
                        "Draw Mode: {}\nOrbiting: {}",
                        conditionals_draws.mode(),
                        orbit
                    ),
                    d.get_screen_width() - 200,
                    10,
                    10,
                    Color::GRAY,
                );
            }
            render_stats.stop();
        }
    }

    fn get_color(index: usize) -> Color {
        match index {
            0 => Color::DARKGRAY,
            1 => Color::RED,
            2 => Color::GREEN,
            3 => Color::BLUE,
            4 => Color::YELLOW,
            5 => Color::SKYBLUE,
            6 => Color::MAGENTA,
            7 => Color::BROWN,
            8 => Color::DARKGREEN,
            9 => Color::DARKBLUE,
            _ => Color::WHITE,
        }
    }

    fn draw_text(d: &mut impl RaylibDraw, font: &Font, x: i32, y: i32, text: &str) {
        d.draw_text_ex(
            font,
            text,
            Vector2::new(x as f32, y as f32),
            16.0,
            2.0,
            Color::WHITE,
        );
    }
}

type Prec = u32;
const WINDOW: usize = 10;
const BASE_INFO_WIDTH: usize = 4;

pub struct VizStats {
    pub start: SystemTime,
    pub queue: CircularQueue<Prec>,
}

impl VizStats {
    fn new() -> VizStats {
        VizStats {
            start: SystemTime::now(),
            queue: CircularQueue::with_capacity(WINDOW),
        }
    }
    fn avg_millis(&self) -> f32 {
        let (c, total) = self.queue.iter().fold((0, 0f32), |(count, acc), next| {
            (count + 1, acc + (*next as f32))
        });
        if c > 0 {
            total / (c as f32 * 1000f32)
        } else {
            total / 1000f32
        }
    }
    fn to_string(&self, title: &str, width: usize, verbose: bool) -> String {
        let text = String::from(title).add(":");
        let p = BASE_INFO_WIDTH + width.saturating_sub(title.len());
        let avg_time = self.avg_millis();
        let smoothed = format!("{0}{1:>2$.1}ms", text, avg_time, p);
        if verbose {
            let unsmoothed = format!("{0:>1$.1}ms", self.get_time(), p + text.len());
            format!("{}\n{}", smoothed, unsmoothed)
        } else {
            smoothed
        }
    }
    fn start(&mut self) {
        self.start = SystemTime::now()
    }
    fn stop(&mut self) {
        self.queue.push(
            self.start
                .elapsed()
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_micros() as Prec,
        );
    }
    fn get_time(&self) -> f32 {
        self.queue.iter().next().unwrap_or(&0).to_owned() as f32 / 1000f32
    }
}

enum ConditionalDraw {
    Buoy,
    Agents,
    All,
    None,
}
impl ConditionalDraw {
    fn next(&self) -> ConditionalDraw {
        match self {
            Self::All => Self::Agents,
            Self::Agents => Self::None,
            Self::None => Self::Buoy,
            Self::Buoy => Self::All,
        }
    }
    fn draw_buoys(&self) -> bool {
        match self {
            Self::Buoy => true,
            Self::Agents => false,
            Self::All => true,
            Self::None => false,
        }
    }
    fn draw_agents(&self) -> bool {
        match self {
            Self::Buoy => false,
            Self::Agents => true,
            Self::All => true,
            Self::None => false,
        }
    }
    fn mode(&self) -> &'static str {
        match self {
            Self::Buoy => "Terrain+A",
            Self::Agents => "Agents+A",
            Self::All => "All",
            Self::None => "Artifacts",
        }
    }
}
