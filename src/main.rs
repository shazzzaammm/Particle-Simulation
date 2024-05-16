extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;

// Window constants
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

// Circle constants
const CIRCLE_RADIUS_MAX: f64 = 25.0;
const CIRCLE_RADIUS_MIN: f64 = 5.0;

// Colors
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const BLUE_A: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const BLUE_B: [f32; 4] = [125.0 / 255.0, 249.0 / 255.0, 1.0, 1.0];
// World constants
const GRAVITY: Vec2 = Vec2 { x: 0.0, y: 0.0 };
const FRICTION: f64 = 0.0;
const COLLISION_ENERGY_LOSS: f64 = 1.0;

// Helper function
fn lerp_color(a: &[f32; 4], b: &[f32; 4], t: f64) -> [f32; 4] {
    let mut color: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    color[0] = b[0] + ((a[0] - b[0]) * t as f32);
    color[1] = b[1] + ((a[1] - b[1]) * t as f32);
    color[2] = b[2] + ((a[2] - b[2]) * t as f32);
    color
}

// Structs
#[derive(Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn add(&mut self, other: &Vec2) {
        self.x += other.x;
        self.y += other.y;
    }

    #[allow(dead_code)]
    fn get_magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[allow(dead_code)]
    fn normalize(&mut self) {
        let magnitude = self.get_magnitude();
        self.x /= magnitude;
        self.y /= magnitude;
    }

    fn mult(&mut self, constant: f64) -> Vec2 {
        Vec2 {
            x: self.x * constant,
            y: self.y * constant,
        }
    }
}

#[derive(Clone, Copy)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
    radius: f64,
    mass: f64,
}

fn new_particle(x: f64, y: f64) -> Particle {
    let random_vx: f64 = rand::thread_rng().gen_range(0.01..1.0);
    let random_vy: f64 = rand::thread_rng().gen_range(0.01..1.0);
    let random_mass: f64 = rand::thread_rng().gen_range(CIRCLE_RADIUS_MIN..CIRCLE_RADIUS_MAX);
    Particle {
        pos: Vec2 { x, y },
        vel: Vec2 {
            x: random_vx,
            y: random_vy,
        },
        acc: Vec2 { x: 0.0, y: 0.0 },
        mass: random_mass,
        radius: random_mass,
    }
}

fn new_particle_system() -> Vec<Particle> {
    let mut particles: Vec<Particle> = Vec::new();
    for i in (CIRCLE_RADIUS_MAX as u32..(WIDTH - CIRCLE_RADIUS_MAX as u32))
        .step_by(5 * CIRCLE_RADIUS_MAX as usize)
    {
        for j in (CIRCLE_RADIUS_MAX as u32..(HEIGHT - CIRCLE_RADIUS_MAX as u32))
            .step_by(5 * CIRCLE_RADIUS_MAX as usize)
        {
            particles.push(new_particle(i as f64, j as f64));
        }
    }
    return particles;
}

fn handle_collisions(particles: &mut Vec<Particle>) {
    for i in 0..particles.len() {
        for j in 0..particles.len() {
            if i == j {
                continue;
            }
            let pos1 = particles[i].pos;
            let pos2 = particles[j].pos;
            let dist = ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt();
            if dist < particles[i].radius + particles[j].radius {
                let overlap = (dist - particles[i].radius - particles[j].radius) * 0.05;
                let mut corrective_displacement = Vec2 {
                    x: overlap * (particles[i].pos.x - particles[j].pos.x),
                    y: overlap * (particles[i].pos.y - particles[j].pos.y),
                };

                particles[j].pos.add(&corrective_displacement);
                particles[i].pos.add(&corrective_displacement.mult(-1.0));

                let pos1 = particles[i].pos;
                let pos2 = particles[j].pos;

                let dist = ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt();

                let normal = Vec2 {
                    x: (pos2.x - pos1.x) / dist,
                    y: (pos2.y - pos1.y) / dist,
                };
                let tangent = Vec2 {
                    x: -normal.y,
                    y: normal.x,
                };

                let disp_tan_1 = particles[i].vel.x * tangent.x + particles[i].vel.y * tangent.y;
                let disp_tan_2 = particles[j].vel.x * tangent.x + particles[j].vel.y * tangent.y;

                let disp_norm_1 = particles[i].vel.x * normal.x + particles[i].vel.y * normal.y;
                let disp_norm_2 = particles[j].vel.x * normal.x + particles[j].vel.y * normal.y;

                let m1 = (disp_norm_1 * (particles[i].mass - particles[j].mass)
                    + 2.0 * particles[j].mass * disp_norm_2)
                    / (particles[i].mass + particles[j].mass);
                let m2 = (disp_norm_2 * (particles[j].mass - particles[i].mass)
                    + 2.0 * particles[i].mass * disp_norm_1)
                    / (particles[i].mass + particles[j].mass);

                particles[i].vel = Vec2 {
                    x: tangent.x * disp_tan_1 + normal.x * m1,
                    y: tangent.y * disp_tan_1 + normal.y * m1,
                };
                particles[j].vel = Vec2 {
                    x: tangent.x * disp_tan_2 + normal.x * m2,
                    y: tangent.y * disp_tan_2 + normal.y * m2,
                };

                particles[i].vel.mult(COLLISION_ENERGY_LOSS);
                particles[j].vel.mult(COLLISION_ENERGY_LOSS);
                break;
            }
        }
    }
}

fn handle_kinematics(particles: &mut Vec<Particle>) {
    // Update
    let total_particles: usize = particles.len();
    for i in 0..total_particles {
        let p1 = &mut particles[i];

        // Update Kinematics
        p1.acc.mult(-FRICTION);
        p1.acc.add(&GRAVITY);
        p1.vel.add(&p1.acc);
        p1.pos.add(&p1.vel);

        // Wall Collisions
        if p1.pos.x - p1.radius < 0.0 {
            p1.vel.x *= -1.0;
            p1.pos.x = p1.radius;
        } else if p1.pos.x + p1.radius > WIDTH as f64 {
            p1.vel.x *= -1.0;
            p1.pos.x = WIDTH as f64 - p1.radius;
        }

        if p1.pos.y - p1.radius < 0.0 {
            p1.vel.y *= -1.0;
            p1.pos.y = p1.radius;
        } else if p1.pos.y + p1.radius > HEIGHT as f64 {
            p1.vel.y *= -1.0;
            p1.pos.y = HEIGHT as f64 - p1.radius;
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Particles?", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .fullscreen(true)
        .build()
        .unwrap();
    let e_settings = EventSettings::new();
    let mut gl = GlGraphics::new(opengl);
    let mut events = Events::new(e_settings);

    let mut particles: Vec<Particle> = new_particle_system();
    let mut mouse_position = Vec2 {
        x: WIDTH as f64 / 2.0,
        y: HEIGHT as f64 / 2.0,
    };

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            // Render
            gl.draw(r.viewport(), |_c, gl| {
                graphics::clear(BLACK, gl);
                for p in particles.iter() {
                    graphics::ellipse(
                        lerp_color(
                            &BLUE_A,
                            &BLUE_B,
                            (p.mass - CIRCLE_RADIUS_MIN) / CIRCLE_RADIUS_MAX,
                        ),
                        graphics::ellipse::circle(p.pos.x, p.pos.y, p.radius),
                        _c.transform,
                        gl,
                    )
                }
            });

            // Update Physics
            handle_kinematics(&mut particles);
            handle_collisions(&mut particles);
        }

        if let Some(b) = e.button_args() {
            match b.state {
                ButtonState::Press => match b.button {
                    Button::Keyboard(Key::Space) => particles = new_particle_system(),
                    Button::Mouse(MouseButton::Left) => {
                        particles.push(new_particle(mouse_position.x, mouse_position.y))
                    }
                    _ => (),
                },
                ButtonState::Release => (),
            }
        }

        if let Some(m) = e.mouse_cursor_args() {
            mouse_position.x = m[0];
            mouse_position.y = m[1];
        }
    }
}
