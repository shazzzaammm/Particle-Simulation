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
const WIDTH: u32 = 600;
const HEIGHT: u32 = 500;
const CIRCLE_RADIUS: f64 = 10.0;

// Colors
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// World constants
const GRAVITY: Coordinate = Coordinate { x: 0.0, y: 0.00 };
// const COLLISION_CONSTANT: f64 = 0.9;

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
}

#[derive(Clone, Copy)]
struct Particle {
    pos: Coordinate,
    vel: Coordinate,
    radius: f64,
    mass: f64,
}

fn main() {
    let mut particles: Vec<Particle> = Vec::new();

    for i in
        (CIRCLE_RADIUS as u32..(WIDTH - CIRCLE_RADIUS as u32)).step_by(5 * CIRCLE_RADIUS as usize)
    {
        for j in (CIRCLE_RADIUS as u32..(HEIGHT - CIRCLE_RADIUS as u32))
            .step_by(5 * CIRCLE_RADIUS as usize)
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
                mass: random_mass,
                radius: random_mass * CIRCLE_RADIUS + 2.0,
            })
        }
    }
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Particles?", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let e_settings = EventSettings::new();
    let mut gl = GlGraphics::new(opengl);
    let mut events = Events::new(e_settings);
    let mut frame = 0;

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
            frame += 1;
            if frame % 1 == 0 {
                // Update
                let total_particles: usize = particles.len();
                for i in 0..total_particles {
                    let p1 = &mut particles[i];
                    let vel = &p1.vel;
                    p1.pos.add(vel);
                    p1.vel.add(&GRAVITY);

                    if p1.pos.x + p1.radius > WIDTH as f64 || p1.pos.x - p1.radius < 0.0 {
                        p1.vel.x *= -1.0;
                    }

                    if p1.pos.y + p1.radius > HEIGHT as f64 || p1.pos.y - p1.radius < 0.0 {
                        p1.vel.y *= -1.0;
                    }

                    for j in 0..total_particles {
                        if i == j {
                            continue;
                        }
                        let pos1 = particles[i].pos;
                        let pos2 = particles[j].pos;
                        let dist = ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt();
                        if dist < particles[i].radius + particles[j].radius {
                            let vi1 = particles[i].vel.clone();
                            let vi2 = particles[j].vel.clone();
                            particles[i].vel.x = vi2.x;
                            particles[j].vel.x = vi1.x;
                            particles[i].vel.y = vi2.y;
                            particles[j].vel.y = vi1.y;
                            break;
                        }
                    }
                }
            }
        }
    }
}
