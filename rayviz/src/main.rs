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

use clap::{App, Arg};

fn main() {
    let matches = App::new("Crustswarm Visualizer")
        .version("1.0")
        .author("Yasin Raies <yasin.raies@gmail.com")
        .about("Visualizes a multi species swarm agent simulation")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("Setis a config file")
                .takes_value(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("framerate")
                .short("f")
                .long("framerate")
                .value_name("FPS")
                .help("Sets the wanted framerate")
                .default_value("30")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("baked_seed")
                .short("b")
                .long("baked_seed")
                .help("Sets a fixed seed on every startup"),
        )
        .arg(
            Arg::with_name("screenshots")
                .long("screenshots")
                .help("Creates screenshots each iteration"),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("SEED")
                .help("Sets a seed")
                .takes_value(true)
                .conflicts_with("random_seed"),
        )
        .get_matches();

    let configfile = matches.value_of("config").unwrap();
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
    rl.set_target_fps(matches.value_of("framerate").unwrap().parse().unwrap());

    let seed = if matches.is_present("baked_seed") {
        3672820499107940204u64
    } else if let Some(seed_string) = matches.value_of("seed") {
        seed_string.parse::<u64>().unwrap()
    } else {
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
    };
    println!("seed: {}", seed);

    let mut rnd: SmallRng = SmallRng::seed_from_u64(seed);
    let mut sg = {
        let temp = crustswarm::io::genome_from_file(configfile);
        crustswarm::swarm::grammar::SwarmGrammar::from(temp, &mut rnd)
    };

    //dbg!(&sg);

    let mut render_stats = VizStats::new();
    let mut sim_stats = VizStats::new();
    let mut calc_next = false;

    let mut orbit = false;
    let orbit_speed = 0.01;

    let mut conditionals_draws = ConditionalDraw::new();

    let font = rl
        .load_font(&thread, fontfile_path.to_str().unwrap())
        .expect("Could not load font");

    let mut terrain_model = rl
        .load_model(&thread, "floor.obj")
        .expect("Loading Model did not succed");

    let mut iteration = 0;

    while !rl.window_should_close() {
        // Update World State
        {
            if calc_next {
                sim_stats.start();
                sg.step(&mut rnd);
                sim_stats.stop();

                iteration += 1;

                if matches.is_present("screenshots") {
                    rl.take_screenshot(
                        &thread,
                        &format!(
                            "./{}_{}/{}.png",
                            configfile.split('.').take(1).last().unwrap(),
                            &seed,
                            &iteration
                        ),
                    );
                }
            }
        }

        let tsize = sg.world.get_size();
        let theight = 60.0;
        let mut image_data = vec![0u8; tsize.0 * tsize.1];
        let toffset = (((tsize.0 - 1) / 2) as f32, ((tsize.1 - 1) / 2) as f32);
        for x in (0..tsize.0) {
            for z in (0..tsize.1) {
                let height = sg.world.get_height_at(
                    (x as f32 - toffset.0) * tsize.2,
                    (z as f32 - toffset.1) * tsize.2,
                );
                let mut factor = if height > theight {
                    1.0
                } else if height < -theight {
                    0.0
                } else {
                    (height + theight) / (2.0 * theight)
                };

                let colval = (factor * 255.0) as u8;

                image_data[x + z * tsize.0] = colval;
            }
        }

        let image = raylib::texture::Image::load_image_pro(
            &image_data,
            tsize.0 as i32,
            tsize.1 as i32,
            PixelFormat::UNCOMPRESSED_GRAYSCALE,
        )
        .unwrap();

        let mesh = raylib::models::Mesh::gen_mesh_heightmap(
            &thread,
            &image,
            Vector3::new(
                tsize.0 as f32 * tsize.2,
                2.0 * theight,
                tsize.1 as f32 * tsize.2,
            ),
        );

        let model = unsafe { raylib::ffi::LoadModelFromMesh(*mesh.as_ref()) };

        // Handle Inputs
        {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                if iteration == 0 && matches.is_present("screenshots") {
                    std::fs::create_dir(&format!(
                        "./{}_{}",
                        configfile.split('.').take(1).last().unwrap(),
                        &seed
                    ))
                    .unwrap();

                    rl.take_screenshot(
                        &thread,
                        &format!(
                            "./{}_{}/{}.png",
                            configfile.split('.').take(1).last().unwrap(),
                            &seed,
                            &iteration
                        ),
                    );
                }
                calc_next = !calc_next;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_F) {
                conditionals_draws.artifacts = !conditionals_draws.artifacts;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_G) {
                conditionals_draws.grid = !conditionals_draws.grid;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_B) {
                conditionals_draws.buoys = !conditionals_draws.buoys;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_T) {
                conditionals_draws.terrain = !conditionals_draws.terrain;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                conditionals_draws.agents = !conditionals_draws.agents;
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
                    .rotate(Quaternion::from_euler(0.0, orbit_speed, 0.0));
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
                //d3d.draw_cube(Vector3::new(1.0, 0.0, 0.0), 2.5, 0.5, 0.5, Color::RED);
                //d3d.draw_cube(Vector3::new(0.0, 1.0, 0.0), 0.5, 2.5, 0.5, Color::GREEN);
                //d3d.draw_cube(Vector3::new(0.0, 0.0, 1.0), 0.5, 0.5, 2.5, Color::BLUE);

                if conditionals_draws.grid {
                    d3d.draw_grid(10, 10.0);
                }

                if conditionals_draws.agents {
                    crustswarm::get_all_agents(&sg)
                        .iter()
                        .for_each(|(ag, spec)| {
                            d3d.draw_cube(
                                Vector3::new(ag.position.x, ag.position.y, ag.position.z),
                                1.0,
                                1.0,
                                1.0,
                                get_color(spec.color_index),
                            );
                        });
                }

                if conditionals_draws.artifacts {
                    let artifacts = crustswarm::get_all_artifacts(&sg);
                    artifacts.iter().for_each(|(art, spec)| {
                        d3d.draw_cube(
                            Vector3::new(art.position.x, art.position.y, art.position.z),
                            1.0,
                            1.0,
                            1.0,
                            get_color(spec.color_index),
                        );

                        if let Some(preid) = art.pre {
                            let (pre, _) = artifacts
                                .iter()
                                .find(|other| other.0.id.eq(&preid))
                                .unwrap();

                            for lerp in &[0.50] {
                                let mut lerpedpos = pre.position;
                                lerpedpos += (art.position - lerpedpos) * *lerp;
                                d3d.draw_cube(
                                    Vector3::new(lerpedpos.x, lerpedpos.y, lerpedpos.z),
                                    0.2,
                                    0.2,
                                    0.2,
                                    get_color(spec.color_index),
                                );
                            }

                            //                            d3d.draw_line_3d(
                            //                                Vector3::new(pre.position.x, pre.position.y, pre.position.z),
                            //                                Vector3::new(art.position.x, art.position.y, art.position.z),
                            //                                get_color(spec.color_index),
                            //                          );
                        }
                    });
                }

                if conditionals_draws.buoys {
                    let buoys = crustswarm::buoys_to_arr2(&sg);
                    for pos in buoys {
                        d3d.draw_cube(
                            Vector3::new(pos[0], pos[1], pos[2]),
                            1.0,
                            1.0,
                            1.0,
                            Color::GRAY,
                        );
                    }
                }

                if conditionals_draws.terrain {
                    unsafe {
                        raylib::ffi::DrawModelEx(
                            model,
                            Vector3::new(-toffset.0 * tsize.2, -theight, -toffset.1 * tsize.2)
                                .into(),
                            Vector3::up().into(),
                            0.0,
                            Vector3::new(1.0, 1.0, 1.0).into(),
                            Color::DARKGRAY.into(),
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
                    &format!("{}\nOrbiting: {}", conditionals_draws, orbit),
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
            0 => Color::WHITE,
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

struct ConditionalDraw {
    agents: bool,
    buoys: bool,
    terrain: bool,
    artifacts: bool,
    grid: bool,
}
impl ConditionalDraw {
    fn new() -> ConditionalDraw {
        ConditionalDraw {
            agents: true,
            buoys: true,
            terrain: false,
            artifacts: true,
            grid: false,
        }
    }
}
impl std::fmt::Display for ConditionalDraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Draw Modes:\nAge(n)ts: {}\nArti(f)acts: {}\n(B)uoys: {}\n(T)errain: {}\n(G)rid: {})",
            self.agents, self.artifacts, self.buoys, self.terrain, self.grid
        )
    }
}
