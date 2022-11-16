use crustswarm::swarm::evo::{OIDESwarmEvalInfo, OIDESwarmParams};
use crustswarm::swarm::genome::SwarmGenome;
use crustswarm::swarm::{evo::genome::OIDESwarmGenome, grammar::SwarmGrammar, world::World};
use crustswarm_lib as crustswarm;
use r_oide::prelude::OIDERandomize;

use std::io::Write;
use std::ops::Add;
use std::string::String;
use std::time::{Duration, SystemTime};
use std::{cell::RefCell, fs::File};

use circular_queue::CircularQueue as RingBuffer;

use tempfile::TempDir;

use rand::prelude::*;

use raylib::prelude::*;

use clap::{App, Arg};

const TERRAIN_FS_SHADER: &str = include_str!("shaders/terrain.glsl.fs");
const TERRAIN_VS_SHADER: &str = include_str!("shaders/terrain.glsl.vs");

fn main() {
    let matches = App::new("Crustswarm Evolution Visualizer")
        .version("1.0")
        .author("Yasin Raies <yasin.raies@stud-mail.uni-wuerzburg.de>")
        .about("Visualizes a multi species swarm agent simulation.\nPress SPACE to transfer an individuum to the next iteration.\nPress 1 2 and 3 to show the three available variations of a given parent/target indiviuum.")
        .arg(
            Arg::with_name("framerate")
                .long("fps")
                .value_name("FPS")
                .help("Sets the wanted framerate")
                .default_value("30")
                .takes_value(true),
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
                .help("Wether the camera starts fixed"),
        )
        .arg(
            Arg::with_name("camera-height")
                .long("camera-height")
                .short("y")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Y position of the simulation camera."),
        )
        .arg(
            Arg::with_name("camera-x")
                .long("camera-x")
                .short("x")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("X position of the simulation camera."),
        )
        .arg(
            Arg::with_name("camera-z")
                .long("camera-z")
                .short("z")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Z position of the simulation camera."),
        )
        .arg(
            Arg::with_name("camera-target")
                .long("camera-target")
                .short("t")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Position the camera will look at."),
        )
        .arg(
            Arg::with_name("orbit-speed")
                .short("o")
                .long("orbit-speed")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("Set the orbiting speed."),
        )
        .arg(
            Arg::with_name("no-ui")
                .long("no-ui")
                .help("Disables the UI"),
        )
        .arg(
            Arg::with_name("no-buoys")
                .long("no-buoys")
                .help("Shows no buoys initially"),
        )
        .arg(
            Arg::with_name("tweenz")
                .long("tweenz")
                .help("Shows no tweenz initially"),
        )
        .arg(
            Arg::with_name("population")
                .long("population")
                .help("size of evolved population")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("species")
                .long("species")
                .help("species count in evolved vsgs")
                .takes_value(true)
                .conflicts_with("template"),
        )
        .arg(
            Arg::with_name("artifact")
                .long("artifact")
                .help("artifact count in evolved vsgs")
                .takes_value(true)
                .conflicts_with("template"),
        )
        .arg(
            Arg::with_name("rules")
                .long("rules")
                .help("rules count in evolved vsgs")
                .takes_value(true)
                .conflicts_with("template"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .help("time after which a given simulation is aborted")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("template")
                .long("template")
                .help("specify a template to generate the population from")
                .value_name("CONFIG")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("iterations")
                .long("iterations")
                .help("iterations to simulate each vsgs")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("save")
                .long("save-to")
                .help("Specify a dir to save each generation to")
                .value_name("SAVE_DIR")
                .takes_value(true),
        )
        .get_matches();

    let seed = matches
        .value_of("seed")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(3672820499107940204u64);
    println!("Using seed: {}", seed);

    let mut rnd = rand::rngs::StdRng::seed_from_u64(seed);

    let population_size = matches
        .value_of("population")
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(20);

    use r_oide::traits::*;

    let (mut population, _oidegenome) = if matches.is_present("template") {
        let configfile = matches.value_of("template").unwrap();
        println!("Using template: {}", &configfile);
        let mut oidegenome = crustswarm::io::oide_genome_from_file(configfile);

        let mut population = vec![];

        if matches.is_present("rebound") {
            let new_bound_genome = OIDESwarmGenome::new(
                *oidegenome.species_count,
                *oidegenome.artifact_count,
                *oidegenome.rule_count,
            );

            oidegenome = new_bound_genome.apply_bounds(&oidegenome);
        }

        population.push(oidegenome.clone());
        let gens = (population_size - 1) / 2;
        for i in 0..gens {
            let random_genome = oidegenome
                .random(&mut rnd)
                .scale(0.2 * (i as f32 / gens as f32));
            population.push(oidegenome.add(&random_genome));
            population.push(oidegenome.add(&random_genome.opposite(None)));
        }

        (population, oidegenome)
    } else {
        let species_count = matches
            .value_of("species")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(4);
        let artifact_count = matches
            .value_of("artifact")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(4);
        let rules_count = matches
            .value_of("rules")
            .map(|s| s.parse::<usize>().unwrap())
            .unwrap_or(4);

        let base = crustswarm::swarm::evo::genome::OIDESwarmGenome::new(
            species_count,
            artifact_count,
            rules_count,
        );

        let mut population = vec![];

        while population.len() < population_size {
            if let Some(genome) = try_get_improved_random(&base, &mut rnd, seed) {
                population.push(genome);
            }
        }

        (population, base)
    };

    let population_size_target = population_size;

    let _basename = "foo".to_string(); /*
                                       let basename = format!(
                                           "{}_{}_{}_{}_{}_{}_{}",
                                           population_size,
                                           species_count,
                                           artifact_count,
                                           rules_count,
                                           context_count,
                                           replacement_count,
                                           seed
                                       );*/

    let tmp_dir = TempDir::new().expect("TempDir could not be created");
    let fontfile_path = tmp_dir.path().join("font.ttf");
    let mut fontfile = File::create(&fontfile_path).expect("Font-Tempfile could not be created");
    fontfile
        .write_all(include_bytes!("../joystix/joystix monospace.ttf"))
        .unwrap();

    let camera_height = matches
        .value_of("camera-height")
        .map_or(40.0, |h| h.parse().unwrap());
    let camera_x = matches
        .value_of("camera-x")
        .map_or(70.0, |h| h.parse().unwrap());
    let camera_z = matches
        .value_of("camera-z")
        .map_or(70.0, |h| h.parse().unwrap());
    let camera_target = Vector3::new(
        0.0,
        matches
            .value_of("camera-target")
            .map_or(20.0, |h| h.parse().unwrap()),
        0.0,
    );

    let generation = RefCell::new(1usize);
    let next_pop_count = RefCell::new(0usize);

    let save_path = matches.value_of("save");
    save_path.map(|path| std::fs::create_dir(&path).unwrap());

    if let Some(path) = save_path {
        population.iter().enumerate().for_each(|(idx, oide)| {
            match crustswarm_lib::io::oide_genome_to_file(
                &oide,
                format!(
                    "{}/gen{:02}_{:02}_h{:020}_random.oide.json",
                    path,
                    generation.borrow(),
                    idx,
                    oide.my_hash()
                ),
            ) {
                Some(e) => {
                    panic!("{}", e)
                }
                None => {}
            }
        });
    }

    let mut selectfoo = |inp: &[(OIDESwarmGenome, SwarmGrammar, OIDESwarmEvalInfo)]| {
        let mut activations = vec![false; inp.len()];
        let mut curr_sel = 0;
        let mut sg = &inp[0].1;

        raylib::core::logging::set_trace_log(TraceLogType::LOG_NONE);

        let (mut rl, thread) = raylib::init()
            .size(1920, 1080)
            .title(&format!("Hello World this is window speaking",))
            .vsync()
            .msaa_4x()
            .resizable()
            .build();

        let mut camera = Camera3D::perspective(
            Vector3::new(camera_x, camera_height, camera_z),
            camera_target,
            Vector3::new(0.0, 1.0, 0.0),
            60.0,
        );

        rl.set_camera_mode(&camera, CameraMode::CAMERA_THIRD_PERSON);
        rl.update_camera(&mut camera);
        rl.set_target_fps(30); //matches.value_of("framerate").unwrap().parse().unwrap());

        let mut render_stats = VizStats::new();

        let mut orbit = true; //matches.is_present("fixed-camera");
        let mut orbit_speed = matches
            .value_of("orbit-speed")
            .map_or(0.01, |o| o.parse().unwrap());

        let mut conditionals_draws = ConditionalDraw::new();
        conditionals_draws.buoys = true; // !matches.is_present("no-buoys");
        conditionals_draws.tweenz = matches.is_present("tweenz");

        let font = rl
            .load_font(&thread, fontfile_path.to_str().unwrap())
            .expect("Could not load font");

        let mut shader =
            rl.load_shader_code(&thread, Some(TERRAIN_VS_SHADER), Some(TERRAIN_FS_SHADER));

        let loc_draw_height_lines = shader.get_shader_location("drawHeightLines");
        shader.set_shader_value(loc_draw_height_lines, 1);
        let loc_viewpos = shader.get_shader_location("viewPos");
        shader.set_shader_value_v(loc_viewpos, &camera.position.to_array());

        let shader = unsafe {
            let matmod = shader.get_shader_location("matModel");
            let mut unsafe_shader = shader.unwrap();
            unsafe_shader.locs[raylib::consts::ShaderLocationIndex::LOC_MATRIX_MODEL as usize] =
                matmod;
            unsafe_shader
        };

        let mut _iteration = -1;
        let mut cube_size_exp = 0i32;
        let base_size = 1.2f32;

        while !rl.window_should_close() {
            // Handle Inputs
            {
                if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
                    curr_sel = 0;
                    sg = &inp[curr_sel].1;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
                    if inp.len() >= 2 {
                        curr_sel = 1;
                        sg = &inp[curr_sel].1;
                    }
                }
                if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
                    if inp.len() >= 3 {
                        curr_sel = 2;
                        sg = &inp[curr_sel].1;
                    }
                }
                if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
                    if inp.len() >= 4 {
                        curr_sel = 3;
                        sg = &inp[curr_sel].1;
                    }
                }

                if rl.get_mouse_wheel_move() != 0 {
                    cube_size_exp += rl.get_mouse_wheel_move();
                }

                if rl.is_key_released(KeyboardKey::KEY_L) {
                    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                        || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT)
                    {
                        dbg!(&sg.world.get_all_agents().collect::<Vec<_>>());
                        dbg!(&sg.world.get_all_artifacts().collect::<Vec<_>>());
                    } else {
                        dbg!(&sg.genome.species_map[0]);
                    }
                }

                if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    activations[curr_sel] = !activations[curr_sel];
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

                if rl.is_key_pressed(KeyboardKey::KEY_P)
                    && rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                {
                    panic!("This was the easiest way to kill this application ...");
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

            /*mesh.normals().iter().for_each(|n| {
                if !n.eq(&Vector3::up()) {
                    println!("{:?}", n)
                }
            });*/

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

                let screen_height = rl.get_screen_height();
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
                                    base_size.powi(cube_size_exp),
                                    base_size.powi(cube_size_exp),
                                    base_size.powi(cube_size_exp),
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
                                ((0f32.max(1f32.min(art.energy / 10.0))) * 0.7 + 0.3)
                                    * base_color.z,
                            ));
                            d3d.draw_cube(
                                Vector3::new(art.position.x, art.position.y, art.position.z),
                                base_size.powi(cube_size_exp),
                                base_size.powi(cube_size_exp),
                                base_size.powi(cube_size_exp),
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
                {
                    draw_text(
                        &mut d,
                        &font,
                        5,
                        5,
                        &format!(
                            "Generation {}\nSet {} of {}\nViewing: {}\nI={}",
                            generation.borrow(),
                            inp[curr_sel].2.pop_id + 1,
                            inp[curr_sel].2.pop_size,
                            inp[curr_sel].2.trial_type.to_string(),
                            inp[curr_sel].2.iterations
                        ),
                    );

                    if activations[curr_sel] {
                        draw_text(
                            &mut d,
                            &font,
                            5,
                            screen_height - 5 - 18,
                            &format!("Transfering into next generation",),
                        );
                    }
                }
                if false {
                    // !matches.is_present("no-ui") {
                    let stat_info = format!(
                        "FPS: {:02}\n{}\nAgents: {:4}\nArts:   {:4}\nBuoys:  {:4}",
                        d.get_fps(),
                        render_stats.to_string("Render", 6, false),
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
        }

        save_path.map(|path| {
            inp.iter()
                .zip(&activations)
                .for_each(|((oide, grammar, info), active)| {
                    match crustswarm::io::grammar_to_file(
                        grammar,
                        &format!(
                            "{}/gen{:02}_base{:02}_{:02}{}.grammar.json",
                            path,
                            generation.borrow(),
                            info.pop_id,
                            info.trial_type.to_string(),
                            if *active { "_mark" } else { "" }
                        ),
                    ) {
                        Some(e) => {
                            panic!("{}", e)
                        }
                        None => {}
                    }
                    if *active {
                        match crustswarm::io::oide_genome_to_file(
                            oide,
                            format!(
                                "{}/gen{:02}_{:02}_h{:020}_p1-{:020}_p2-{:020}_chosen.oide.json",
                                path,
                                generation.borrow(),
                                next_pop_count.replace_with(|&mut v| v + 1),
                                oide.my_hash(),
                                info.parents.0,
                                info.parents.1,
                            ),
                        ) {
                            Some(e) => {
                                panic!("{}", e)
                            }
                            None => {}
                        };
                    }
                })
        });

        activations
            .iter()
            .enumerate()
            .flat_map(|(idx, active)| {
                if *active {
                    Some(inp[idx].0.clone())
                } else {
                    None
                }
            })
            .collect()
    };

    let iterations = matches
        .value_of("iterations")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(30);

    let timeout = matches
        .value_of("timeout")
        .map(|s| s.parse::<u64>().unwrap())
        .unwrap_or(10);

    for _i in 0..100 {
        let midpoint = population.get_midpoints();
        population = population.step(
            &mut selectfoo,
            &mut rnd,
            OIDESwarmParams {
                seed,
                max_iterations: iterations as usize,
                timeout_hint: Duration::from_secs(timeout),
            },
            Some(&midpoint),
            0.5,
            0.5,
        );
        generation.replace_with(|&mut v| v + 1);
        next_pop_count.replace_with(|_| 0);
        if population.len() > 3 {
            let mut max_add = (population_size_target as f32 / 4f32) as u32;
            while population.len() < population_size_target && max_add > 0 {
                if let Some(genome) = try_get_improved_random(&population[0], &mut rnd, seed) {
                    if let Some(path) = save_path {
                        match crustswarm_lib::io::oide_genome_to_file(
                            &genome,
                            format!(
                                "{}/gen{:02}_{:02}_h{:020}_random.oide.json",
                                path,
                                generation.borrow(),
                                population.len(),
                                genome.my_hash()
                            ),
                        ) {
                            Some(e) => {
                                panic!("{}", e)
                            }
                            None => {}
                        }
                    }
                    population.push(genome);
                    max_add -= 1;
                }
            }
        }
    }

    //dbg!(&sg);
}

fn try_get_improved_random(
    base: &OIDESwarmGenome,
    rnd: &mut StdRng,
    seed: u64,
) -> Option<OIDESwarmGenome> {
    let genome = base.random(rnd);
    let mut grammar = SwarmGrammar::from(SwarmGenome::from(&genome), rnd);
    let mut loc_rnd = rand::rngs::StdRng::seed_from_u64(seed);
    for _ in 0..10 {
        grammar.step(&mut loc_rnd);
    }
    if grammar.get_world().get_all_agents().count()
        + grammar.get_world().get_all_artifacts().count()
        > 0
        && grammar.genome.species_map[0]
            .rules
            .iter()
            .any(|r| r.replacement.count_replacements() >= 1)
    {
        Some(genome)
    } else {
        None
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
        _ => Color::ORANGE,
    }
}

fn draw_text(d: &mut impl RaylibDraw, font: &Font, x: i32, y: i32, text: &str) {
    d.draw_text_ex(
        font,
        text,
        Vector2::new(x as f32, y as f32),
        18.0,
        2.0,
        Color::BLACK,
    );
}

type Prec = u32;
const WINDOW: usize = 10;
const BASE_INFO_WIDTH: usize = 4;

pub struct VizStats {
    pub start: SystemTime,
    pub queue: RingBuffer<Prec>,
}

impl VizStats {
    fn new() -> VizStats {
        VizStats {
            start: SystemTime::now(),
            queue: RingBuffer::with_capacity(WINDOW),
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
            ignores: true,
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
