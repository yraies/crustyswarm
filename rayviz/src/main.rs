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
use regex;

const TERRAIN_FS_SHADER: &str = include_str!("shaders/terrain.glsl.fs");
const TERRAIN_VS_SHADER: &str = include_str!("shaders/terrain.glsl.vs");

fn main() {
    let matches = App::new("Crustswarm Visualizer")
        .version("0.1")
        .author("Yasin Raies <yasin.raies@gmail.com>")
        .about("Visualize a multi species swarm agent simulation")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG")
                .help("Sets a config file")
                .takes_value(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("framerate")
                .long("fps")
                .value_name("FPS")
                .help("Sets the target framerate")
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
                .value_name("SNUM")
                .default_value("1")
                .takes_value(true)
                .help("Creates screenshots every <SNUM> iterations"),
        )
        .arg(
            Arg::with_name("screenshot-once")
                .long("screenshot-once")
                .takes_value(true)
                .help("Creates a single single screenshot of a given grammar and quites")
                .conflicts_with("screenshot")
                .conflicts_with("instant")
                .conflicts_with("max-iteration"),
        )
        .arg(
            Arg::with_name("instant")
                .long("instant")
                .short("i")
                .help("Instantly starts the simulation"),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .value_name("SEED")
                .help("Sets a seed")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fixed-camera")
                .long("fixed-camera")
                .help("Fixes the camera to orbit the scene"),
        )
        .arg(
            Arg::with_name("camera-height")
                .long("camera-height")
                .short("y")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Y position of the simulation camera"),
        )
        .arg(
            Arg::with_name("camera-x")
                .long("camera-x")
                .short("x")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("X position of the simulation camera"),
        )
        .arg(
            Arg::with_name("camera-z")
                .long("camera-z")
                .short("z")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Z position of the simulation camera"),
        )
        .arg(
            Arg::with_name("camera-target")
                .long("camera-target")
                .short("t")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Target height the camera will look at"),
        )
        .arg(
            Arg::with_name("orbit-speed")
                .short("o")
                .long("orbit-speed")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Sets the orbiting speed."),
        )
        .arg(
            Arg::with_name("max-iteration")
                .short("m")
                .long("max-iteration")
                .takes_value(true)
                .help("Numbers of iterations to run (quits afterwards)"),
        )
        .arg(
            Arg::with_name("no-ui")
                .long("no-ui")
                .help("Disables the UI"),
        )
        .arg(
            Arg::with_name("fullscreen")
                .short("f")
                .long("fullscreen")
                .help("Runs the application fullscreen"),
        )
        .arg(
            Arg::with_name("square")
                .long("square")
                .conflicts_with("fullscreen")
                .help("Runs the simulation with a square aspect ratio"),
        )
        .arg(
            Arg::with_name("no-buoys")
                .long("no-buoys")
                .help("Shows no buoys initially"),
        )
        .arg(
            Arg::with_name("no-terrain")
                .long("no-terrain")
                .help("Shows no terrain initially"),
        )
        .arg(
            Arg::with_name("no-tweenz")
                .long("no-tweenz")
                .help("Shows no tweenz (intermediary boxes indicating predecessors) initially"),
        )
        .arg(
            Arg::with_name("genome")
                .long("genome")
                .help("Interprets the configuration as a SG genome")
                .conflicts_with("oide"),
        )
        .arg(
            Arg::with_name("grammar")
                .long("grammar")
                .help("Interprets the configuration as a raw grammar")
                .conflicts_with("genome"),
        )
        .arg(
            Arg::with_name("restart")
                .long("restart")
                .help("Restarts the interpretation of a raw grammar at iteration zero")
                .requires("grammar"),
        )
        .arg(
            Arg::with_name("oide")
                .long("oide")
                .help("Interprets the configuration as a oide genome")
                .conflicts_with("grammar"),
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

    let (mut rl, thread) = if matches.is_present("fullscreen") {
        raylib::init()
            .title(&format!(
                "Hello World this is fullscreen window speaking - {}",
                &configfile
            ))
            .msaa_4x()
            .resizable()
            .vsync()
            .size(1920, 1080)
            .fullscreen()
            .build()
    } else if matches.is_present("square") {
        raylib::init()
            .title(&format!(
                "Hello World this is square window speaking - {}",
                &configfile
            ))
            .msaa_4x()
            .vsync()
            .size(1024, 1024)
            .build()
    } else {
        raylib::init()
            .size(1270, 720)
            .title(&format!(
                "Hello World this is window speaking - {}",
                &configfile
            ))
            .vsync()
            .msaa_4x()
            .resizable()
            .build()
    };

    let camera_height = matches
        .value_of("camera-height")
        .map_or(40.0, |h| h.parse().unwrap());
    let camera_x = matches
        .value_of("camera-x")
        .map_or(100.0, |h| h.parse().unwrap());
    let camera_z = matches
        .value_of("camera-z")
        .map_or(100.0, |h| h.parse().unwrap());
    let mut camera_target = Vector3::new(
        0.0,
        matches
            .value_of("camera-target")
            .map_or(10.0, |h| h.parse().unwrap()),
        0.0,
    );

    let mut camera = Camera3D::perspective(
        Vector3::new(camera_x, camera_height, camera_z),
        camera_target,
        Vector3::new(0.0, 1.0, 0.0),
        60.0,
    );

    rl.set_camera_mode(&camera, CameraMode::CAMERA_THIRD_PERSON);
    rl.update_camera(&mut camera);
    rl.set_target_fps(matches.value_of("framerate").unwrap().parse().unwrap());

    let mut seed = if matches.is_present("baked_seed") {
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
    let mut sg = if matches.is_present("grammar") && !matches.is_present("restart") {
        crustswarm::io::grammar_from_file(configfile)
    } else {
        let temp = if matches.is_present("genome") {
            crustswarm::io::raw_genome_from_file(configfile)
        } else if matches.is_present("oide") {
            crustswarm::swarm::genome::SwarmGenome::from(&crustswarm::io::oide_genome_from_file(
                configfile,
            ))
        } else if matches.is_present("restart") {
            if matches.is_present("baked_seed") {
                seed = 73151514124691;
            }
            crustswarm::io::grammar_from_file(configfile).genome
        } else {
            crustswarm::io::genome_from_file(configfile)
        };
        crustswarm::swarm::grammar::SwarmGrammar::from(temp, &mut rnd)
    };

    //dbg!(&sg);

    let regex = regex::Regex::new(r"^(.+)\.json$").unwrap();
    let basename = format!(
        "{}_{}",
        regex
            .captures(&configfile)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str(),
        &seed
    );
    let screenshot_path = format!("./{}", &basename);
    if matches.occurrences_of("screenshots") > 0 {
        std::fs::create_dir(&screenshot_path).unwrap();
    }
    let screenshot_modulo = matches
        .value_of("screenshots")
        .map(|s| s.parse::<i32>().unwrap())
        .unwrap();

    let mut render_stats = VizStats::new();
    let mut sim_stats = VizStats::new();
    let mut calc_next = matches.is_present("instant");
    let mut calc_one = false;

    let mut orbit = matches.is_present("fixed-camera");
    let mut orbit_speed = matches
        .value_of("orbit-speed")
        .map_or(0.01, |o| o.parse().unwrap());

    let mut conditionals_draws = ConditionalDraw::new();
    conditionals_draws.buoys = !matches.is_present("no-buoys");
    conditionals_draws.terrain = !matches.is_present("no-terrain");
    conditionals_draws.tweenz = !matches.is_present("no-tweenz");

    let font = rl
        .load_font(&thread, fontfile_path.to_str().unwrap())
        .expect("Could not load font");

    let mut shader = rl.load_shader_code(&thread, Some(TERRAIN_VS_SHADER), Some(TERRAIN_FS_SHADER));

    let loc_draw_height_lines = dbg!(shader.get_shader_location("drawHeightLines"));
    shader.set_shader_value(loc_draw_height_lines, 1);
    let loc_viewpos = dbg!(shader.get_shader_location("viewPos"));
    shader.set_shader_value_v(loc_viewpos, &camera.position.to_array());

    let shader = unsafe {
        let matmod = dbg!(shader.get_shader_location("matModel"));
        let mut unsafe_shader = shader.unwrap();
        unsafe_shader.locs[raylib::consts::ShaderLocationIndex::LOC_MATRIX_MODEL as usize] = matmod;
        unsafe_shader
    };

    let mut iteration = -1;
    let max_iteration = matches
        .value_of("max-iteration")
        .map_or(std::i32::MAX, |i| i.parse::<i32>().unwrap());
    let mut made_screenshot_once = false;

    while !rl.window_should_close() {
        if iteration > max_iteration || made_screenshot_once {
            println!("{:?}", camera);
            break;
        }

        // Handle Inputs
        {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                calc_next = !calc_next;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_X) {
                calc_one = true;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_F) {
                conditionals_draws.artifacts = !conditionals_draws.artifacts;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_I) {
                conditionals_draws.ignores = !conditionals_draws.ignores;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_G) {
                conditionals_draws.grid = !conditionals_draws.grid;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_B) {
                //shader.set_shader_value(loc_draw_height_lines, conditionals_draws.buoys as i32);
                conditionals_draws.buoys = !conditionals_draws.buoys;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_T) {
                conditionals_draws.terrain = !conditionals_draws.terrain;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                conditionals_draws.agents = !conditionals_draws.agents;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_Y) {
                conditionals_draws.tweenz = !conditionals_draws.tweenz;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_L) {
                camera_target = camera.target;
            }

            if rl.is_key_pressed(KeyboardKey::KEY_O) {
                if orbit {
                    orbit = false;
                } else {
                    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        orbit_speed = 0.005;
                    } else if rl.is_key_down(KeyboardKey::KEY_LEFT_ALT) {
                        orbit_speed = 0.0;
                    }
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
                camera.target = camera_target;
            }

            let foo = camera.position;

            unsafe {
                raylib::ffi::SetShaderValue(
                    shader,
                    loc_viewpos,
                    foo.to_array().as_ptr() as *const ::std::os::raw::c_void,
                    (raylib::ffi::ShaderUniformDataType::UNIFORM_VEC3 as u32) as i32,
                );
            }
        }

        // Update World State
        {
            if calc_next || calc_one {
                sim_stats.start();
                sg.step(&mut rnd);
                sim_stats.stop();

                iteration += 1;
            }
        }

        let tsize = sg.world.get_size();
        let theight = 70.0;
        let mut image_data = vec![0u8; tsize.0 * tsize.1];
        let toffset = (((tsize.0 - 1) / 2) as f32, ((tsize.1 - 1) / 2) as f32);
        for x in 0..tsize.0 {
            for z in 0..tsize.1 {
                let height = sg.world.get_height_at(
                    (x as f32 - toffset.0) * tsize.2,
                    (z as f32 - toffset.1) * tsize.2,
                );
                let factor = if height > theight {
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

        let mut mesh = raylib::models::Mesh::gen_mesh_heightmap(
            &thread,
            &image,
            Vector3::new(
                tsize.0 as f32 * tsize.2,
                2.0 * theight,
                tsize.1 as f32 * tsize.2,
            ),
        );

        let terrain_offset = if false {
            mesh = raylib::models::Mesh::gen_mesh_sphere(&thread, 10.0, 32, 32);
            Vector3::new(0.0, 0.0, 0.0)
        } else {
            Vector3::new(-toffset.0 * tsize.2, -theight, -toffset.1 * tsize.2)
        };

        mesh.mesh_tangents();
        mesh.mesh_binormals();

        mesh.normals().iter().for_each(|n| {
            if !n.eq(&Vector3::up()) {
                println!("{:?}", n)
            }
        });

        let model = unsafe { raylib::ffi::LoadModelFromMesh(*mesh.as_ref()) };

        unsafe {
            let materials = std::slice::from_raw_parts_mut(
                model.materials as *mut Material,
                model.materialCount as usize,
            );
            materials[0].shader = shader;
        }

        // Draw the Scene
        {
            render_stats.start();

            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::color_from_hsv(Vector3::new(0.0, 0.0, 0.9)));

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
                        if spec.color_index == 10 && !conditionals_draws.ignores {
                            return;
                        }
                        let base_color = get_color(spec.color_index).color_to_hsv();
                        let new_color = Color::color_from_hsv(Vector3::new(
                            base_color.x,
                            base_color.y,
                            ((0f32.max(1f32.min(art.energy / 10.0))) * 0.7 + 0.3) * base_color.z,
                        ));
                        d3d.draw_cube(
                            Vector3::new(art.position.x, art.position.y, art.position.z),
                            1.0,
                            1.0,
                            1.0,
                            new_color,
                        );

                        if conditionals_draws.tweenz {
                            if let Some(preid) = art.pre {
                                let (pre, _) = artifacts
                                    .iter()
                                    .find(|other| other.0.id.eq(&preid))
                                    .unwrap();

                                for lerp in &[0.25, 0.5, 0.75] {
                                    let mut lerpedpos = pre.position;
                                    lerpedpos += (art.position - lerpedpos) * *lerp;
                                    d3d.draw_cube(
                                        Vector3::new(lerpedpos.x, lerpedpos.y, lerpedpos.z),
                                        0.2,
                                        0.2,
                                        0.2,
                                        new_color,
                                    );
                                }

                                //                            d3d.draw_line_3d(
                                //                                Vector3::new(pre.position.x, pre.position.y, pre.position.z),
                                //                                Vector3::new(art.position.x, art.position.y, art.position.z),
                                //                                get_color(spec.color_index),
                                //                          );
                            }
                        }
                    });
                }

                if conditionals_draws.buoys {
                    let buoys = crustswarm::buoys_to_arr2(&sg);
                    for pos in buoys {
                        d3d.draw_cube(
                            Vector3::new(pos[0], pos[1], pos[2]),
                            0.5,
                            0.5,
                            0.5,
                            Color::GRAY,
                        );
                    }
                }

                if conditionals_draws.terrain {
                    unsafe {
                        raylib::ffi::DrawModelEx(
                            model,
                            terrain_offset.into(),
                            Vector3::up().into(),
                            0.0,
                            Vector3::new(1.0, 1.0, 1.0).into(),
                            Color::DARKGRAY.into(),
                        );
                    }
                }
            }

            // Draw UI Stuff
            if !matches.is_present("no-ui") {
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
- Shift to increase movement speedi
- X to simulate a single step
- Space to run and pause the simulation",
                    10,
                    d.get_screen_height() - 100,
                    10,
                    Color::GRAY,
                );
                d.draw_text(
                    &format!("{}\n(O)rbiting: {}", conditionals_draws, orbit),
                    d.get_screen_width() - 200,
                    10,
                    10,
                    Color::GRAY,
                );
            }
            render_stats.stop();
        }

        // Screenshot
        {
            if calc_next || calc_one {
                calc_one = false;
                if matches.occurrences_of("screenshots") > 0 && iteration % screenshot_modulo == 0 {
                    rl.take_screenshot(
                        &thread,
                        &format!("{}/{}_{:04}.png", &screenshot_path, &basename, &iteration),
                    );
                    image.export_image(&format!(
                        "{}/heightmap_{}_{:04}.png",
                        &screenshot_path, &basename, &iteration
                    ));
                }
            }
            if matches.is_present("screenshot-once") {
                let path = matches.value_of("screenshot-once").unwrap_or(".");
                rl.take_screenshot(&thread, &format!("{}/{}.png", &path, &configfile));
                made_screenshot_once = true;
            }
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
            10 => Color::GRAY,
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
    tweenz: bool,
    ignores: bool,
}
impl ConditionalDraw {
    fn new() -> ConditionalDraw {
        ConditionalDraw {
            agents: true,
            buoys: true,
            terrain: true,
            artifacts: true,
            grid: false,
            tweenz: true,
            ignores: false,
        }
    }
}
impl std::fmt::Display for ConditionalDraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Draw Modes:\nAge(n)ts: {}\nArti(f)acts: {}\n(B)uoys: {}\n(T)errain: {}\nTween(z): {}\n(G)rid: {}\n(I)gnores: {}",
            self.agents, self.artifacts, self.buoys, self.terrain, self.tweenz, self.grid, self.ignores
        )
    }
}
