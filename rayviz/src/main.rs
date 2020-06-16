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
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0),
        Vector3::new(0.0, 1.8, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        70.0,
    );

    rl.set_camera_mode(&camera, CameraMode::CAMERA_THIRD_PERSON);
    rl.set_target_fps(15);

    let seed = if false { rand::random() } else { 2u64 };
    let mut rnd: SmallRng = SmallRng::seed_from_u64(seed);
    let mut sg = {
        let temp = crustswarm::io::genome_from_file(configfile);
        crustswarm::swarm::grammar::SwarmGrammar::from(temp, &mut rnd)
    };

    let mut render_stats = VizStats::new();
    let mut sim_stats = VizStats::new();
    let mut calc_next = false;
    let mut draw_buoy = true;

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
                draw_buoy = !draw_buoy;
            }

            let old_position = camera.position;
            rl.update_camera(&mut camera);
            let camera_movement = camera.position - old_position;
            if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                camera.position += camera_movement * 20.0;
                camera.target += camera_movement * 20.0;
            } else {
                camera.position += camera_movement * 4.0;
                camera.target += camera_movement * 4.0;
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

                d3d.draw_grid(10, 10.0);

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

                if draw_buoy {
                    let buoys = crustswarm::buoys_to_arr2(&sg);
                    for pos in buoys {
                        d3d.draw_cube(
                            Vector3::new(pos[0], pos[1], pos[2]),
                            0.6,
                            0.6,
                            0.6,
                            get_color(8),
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
/*
        while !window.should_close() {
            //// Handle Input

            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                        calc_next = !calc_next;
                    }
                    glfw::WindowEvent::Key(Key::M, _, Action::Press, _) => {
                        draw_buoy = !draw_buoy;
                    }
                    _ => {}
                }
            }
*/
/*
        //// Recalc delay/time
        println!("{:?}", last_step.elapsed());
        thread::sleep(
            frametime
                .checked_sub(last_step.elapsed().unwrap())
                .unwrap_or(Duration::new(0, 0)),
        );
        last_step = SystemTime::now();

        shader.use_program();

        if calc_next {
            sg.step(&mut rnd);
            agent_vo.update_buffer(crustswarm::agents_to_arr(&sg), Usage::Stream);
        }

        //// Set camera
        uni_view.update(camera.get_view_matrix());
        buoy_shader.use_program();
        ubi_view.update(camera.get_view_matrix());

        //// Render
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }

        shader.use_program();
        uni_mark.update(1);
        floor_vo.draw(DrawType::Points);
        println!("f{}", floor_vo.get_entry_count());

        uni_mark.update(0);
        agent_vo.draw(DrawType::Points);
        println!("a{}", agent_vo.get_entry_count());

        buoy_shader.use_program();
        if draw_buoy {
            buoy_vo.update_buffer(crustswarm::buoys_to_arr(&sg), Usage::Stream);
            buoy_vo.draw(DrawType::Points);
        }
        println!("b{}", buoy_vo.get_entry_count());

        window.swap_buffers();
    }

    //    println!("{:?}", crustswarm::swarm_to_arr(&sg));
}

fn gen_floor_mark(spacing: f32, count: usize) -> Vec<f32> {
    for i_ in 1..=count {
        let i = (i_ as f32) * spacing;
        mark.push(i);
        mark.push(0.0);
        mark.push(0.0);
        mark.push(5.0);

        color 5
        Vector3::new( i,0.0,0.0)
        Vector3::new(-i,0.0,0.0)
        color 4
        Vector3::new(0.0,0.0, i)
        Vector3::new(0.0,0.0,-i)
    }
}
    )*/
