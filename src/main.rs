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
use rand::random;

// Window constants
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

// Circle constants
const CIRCLE_RADIUS_FACTOR: f64 = 15.0;
const CIRCLE_RADIUS_MIN: f64 = CIRCLE_RADIUS_FACTOR / 2.0;

// Colors
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// World constants
const GRAVITY: Coordinate = Coordinate { x: 0.0, y: 0.0 };
const FRICTION: f64 = 0.0;
const COLLISION_ENERGY_LOSS: f64 = 1.0;

// Structs
#[derive(Clone, Copy)]
struct Coordinate {
    x: f64,
    y: f64,
}

impl Coordinate {
    fn add(&mut self, other: &Coordinate) {
        self.x += other.x;
        self.y += other.y;
    }

    fn mult(a: &Coordinate, constant: f64) -> Coordinate {
        Coordinate {
            x: a.x * constant,
            y: a.y * constant,
        }
    }
}

#[derive(Clone, Copy)]
struct Particle {
    pos: Coordinate,
    vel: Coordinate,
    acc: Coordinate,
    radius: f64,
    mass: f64,
}

fn main() {
    let mut particles: Vec<Particle> = Vec::new();

    for i in (CIRCLE_RADIUS_FACTOR as u32..(WIDTH - CIRCLE_RADIUS_FACTOR as u32))
        .step_by(5 * CIRCLE_RADIUS_FACTOR as usize)
    {
        for j in (CIRCLE_RADIUS_FACTOR as u32..(HEIGHT - CIRCLE_RADIUS_FACTOR as u32))
            .step_by(5 * CIRCLE_RADIUS_FACTOR as usize)
        {
            let random_x: f64 = random();
            let random_y: f64 = random();
            let random_mass: f64 = random();
            particles.push(Particle {
                pos: Coordinate {
                    x: i as f64,
                    y: j as f64,
                },
                vel: Coordinate {
                    x: random_x,
                    y: random_y,
                },
                acc: Coordinate { x: 0.0, y: 0.0 },
                mass: random_mass + 1.0,
                radius: random_mass * CIRCLE_RADIUS_FACTOR + CIRCLE_RADIUS_MIN,
            })
        }
    }
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

    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            // Render
            gl.draw(r.viewport(), |_c, gl| {
                graphics::clear(BLACK, gl);
                for p in particles.iter() {
                    graphics::ellipse(
                        WHITE,
                        graphics::ellipse::circle(p.pos.x, p.pos.y, p.radius),
                        _c.transform,
                        gl,
                    )
                }
            });
            // Update
            let total_particles: usize = particles.len();
            for i in 0..total_particles {
                let p1 = &mut particles[i];

                // Update Kinematics
                p1.acc = Coordinate::mult(&p1.vel, -FRICTION);
                p1.acc.add(&GRAVITY);
                p1.vel.add(&p1.acc);
                p1.pos.add(&p1.vel);

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

                for j in 0..total_particles {
                    if i == j {
                        continue;
                    }
                    // Update Collisions
                    let pos1 = particles[i].pos;
                    let pos2 = particles[j].pos;
                    let dist = ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt();
                    if dist < particles[i].radius + particles[j].radius {
                        let overlap = (dist - particles[i].radius - particles[j].radius) * 0.05;
                        let corrective_displacement = Coordinate {
                            x: overlap * (particles[i].pos.x - particles[j].pos.x),
                            y: overlap * (particles[i].pos.y - particles[j].pos.y),
                        };

                        particles[j].pos.add(&corrective_displacement);
                        particles[i]
                            .pos
                            .add(&Coordinate::mult(&corrective_displacement, -1.0));

                        let pos1 = particles[i].pos;
                        let pos2 = particles[j].pos;

                        let dist = ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt();

                        let normal = Coordinate {
                            x: (pos2.x - pos1.x) / dist,
                            y: (pos2.y - pos1.y) / dist,
                        };
                        let tangent = Coordinate {
                            x: -normal.y,
                            y: normal.x,
                        };

                        let disp_tan_1 =
                            particles[i].vel.x * tangent.x + particles[i].vel.y * tangent.y;
                        let disp_tan_2 =
                            particles[j].vel.x * tangent.x + particles[j].vel.y * tangent.y;

                        let disp_norm_1 =
                            particles[i].vel.x * normal.x + particles[i].vel.y * normal.y;
                        let disp_norm_2 =
                            particles[j].vel.x * normal.x + particles[j].vel.y * normal.y;

                        let m1 = (disp_norm_1 * (particles[i].mass - particles[j].mass)
                            + 2.0 * particles[j].mass * disp_norm_2)
                            / (particles[i].mass + particles[j].mass);
                        let m2 = (disp_norm_2 * (particles[j].mass - particles[i].mass)
                            + 2.0 * particles[i].mass * disp_norm_1)
                            / (particles[i].mass + particles[j].mass);

                        particles[i].vel = Coordinate {
                            x: tangent.x * disp_tan_1 + normal.x * m1,
                            y: tangent.y * disp_tan_1 + normal.y * m1,
                        };
                        particles[j].vel = Coordinate {
                            x: tangent.x * disp_tan_2 + normal.x * m2,
                            y: tangent.y * disp_tan_2 + normal.y * m2,
                        };

                        Coordinate::mult(&particles[i].vel, COLLISION_ENERGY_LOSS);
                        Coordinate::mult(&particles[j].vel, COLLISION_ENERGY_LOSS);

                        break;
                    }
                }
            }
        }
    }
}
