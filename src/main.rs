use hecs::*;
use macroquad::prelude::*;
use rodio::*;

mod components;

// Vars
const DISPLAY_TARGET_WIDTH: u32 = 256; // The width we want.
const DISPLAY_TARGET_HEIGHT: u32 = 144; // The height we want.
const DEBUG_ENABLED: bool = false; // For debug view.
const MAX_VOLUME: f32 = 0.1;

macro_rules! play_audio {
    ($sink:ident, $file:expr $(,)?, $volume:expr $(,)?, $speed:expr $(,)?) => {
        $sink.skip_one();
        $sink.append(
            Decoder::new_wav(std::io::Cursor::new(&include_bytes!($file)))
                .unwrap()
                .amplify($volume)
                .speed($speed),
        );
    };
}

// Helper functions.
pub fn square_distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    (x1 - x2).powf(2.0) + (y1 - y2).powf(2.0)
}

pub fn deg2rad(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}

pub fn rad2deg(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

pub fn normalize_point(point: (f32, f32)) -> (f32, f32) {
    let sum = point.0 + point.1;
    (point.0 / sum, point.1 / sum)
}

pub fn rotate_point(point: (f32, f32), degrees: f32) -> (f32, f32) {
    let sin = deg2rad(degrees).sin();
    let cos = deg2rad(degrees).cos();
    (point.0 * cos - point.1 * sin, point.0 * sin + point.1 * cos)
}

fn create_asteroid(world: &mut World) {
    let time = get_time() as f32 * 128.0;
    create_asteroid_point(
        world,
        (
            DISPLAY_TARGET_WIDTH as f32 * time.cos() + DISPLAY_TARGET_WIDTH as f32 / 2.0,
            DISPLAY_TARGET_HEIGHT as f32 * time.sin() + DISPLAY_TARGET_HEIGHT as f32 / 2.0,
        ),
        (rand::rand() % 400) as f32 / 100.0 + 3.0,
    );
}

fn create_asteroid_point(world: &mut World, point: (f32, f32), size: f32) {
    let mut rock_shape = (0..16)
        .map(|x| rotate_point((0.0, (rand::rand() % 2) as f32 + size), x as f32 * 22.5))
        .collect::<Vec<(f32, f32)>>();
    rock_shape.push(rock_shape[0]);
    world.spawn((
        components::Position(point.0, point.1, 0.0),
        components::Velocity(
            (rand::rand() % 20) as f32 - 10.0,
            (rand::rand() % 20) as f32 - 10.0,
            (rand::rand() % 20) as f32 - 10.0,
        ),
        components::Draw(
            Color {
                r: 0.7,
                g: 0.7,
                b: 0.7,
                a: 1.0,
            },
            rock_shape,
        ),
        components::Collidable(size, components::CollidableType::ASTEROID),
    ));
}

// Particle Sutff for obvi reasons.
//
#[derive(Default, Clone, Copy)]
struct Particle {
    position: (f32, f32),
    velocity: (f32, f32),
    drag: f32,
    size: f32,
    color: Color,
    birthtime: f64,
    deathtime: f64,
}

#[derive(Default, Clone)]
struct ParticleStorage {
    particles_container: Vec<Particle>,
}

impl ParticleStorage {
    fn new() -> Self {
        Self {
            particles_container: Vec::new(),
        }
    }

    fn create_particle(
        &mut self,
        count: i32,
        position: (f32, f32),
        velocity: (f32, f32),
        drag: f32,
        size: f32,
        color: Color,
        age: f64,
        position_variance: (f32, f32),
        velocity_variance: (f32, f32),
        size_variance: f32,
        age_variance: f64,
    ) {
        let curr_time = macroquad::time::get_time();
        for _i in 0..count {
            self.particles_container.push(Particle {
                position: (
                    position.0
                        + rand::RandomRange::gen_range(-position_variance.0, position_variance.0),
                    position.1
                        + rand::RandomRange::gen_range(-position_variance.1, position_variance.1),
                ),
                velocity: (
                    velocity.0
                        + rand::RandomRange::gen_range(-velocity_variance.0, velocity_variance.0),
                    velocity.1
                        + rand::RandomRange::gen_range(-velocity_variance.1, velocity_variance.1),
                ),
                drag: drag,
                size: size + rand::RandomRange::gen_range(-size_variance, size_variance),
                color: color,
                birthtime: curr_time,
                deathtime: curr_time
                    + age
                    + rand::RandomRange::gen_range(-age_variance, age_variance),
            })
        }
    }
}

// This, too, is yuri.
fn world_reset(world: &mut World) {
    world.clear(); // Resetting the world.
                   // Our left paddle.
                   // Getting our new things in.
    world.spawn((
        components::Position(
            DISPLAY_TARGET_WIDTH as f32 / 2.0,
            DISPLAY_TARGET_HEIGHT as f32,
            0.0,
        ),
        components::Velocity(0.0, 20.0, 1125.0),
        components::Draw(
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            vec![
                (-2.0, 3.0),
                (0.0, -3.0),
                (2.0, 3.0),
                (0.0, 1.0),
                (-2.0, 3.0),
            ],
        ),
        components::Collidable(1.0, components::CollidableType::PLAYER),
        components::Controllable(),
    ));
}

// Window Stuff
//

// Setting Window Configurations.
fn config() -> Conf {
    Conf {
        window_title: "Asteroids".to_string(),
        window_width: DISPLAY_TARGET_WIDTH as i32 * 4,
        window_height: DISPLAY_TARGET_HEIGHT as i32 * 4,
        ..Default::default()
    }
}

// And main.
#[macroquad::main(config)]
async fn main() {
    // For rendering.
    let render_target = render_target(DISPLAY_TARGET_WIDTH, DISPLAY_TARGET_HEIGHT); // Setting our internal window size.
    render_target.texture.set_filter(FilterMode::Nearest); // And we love some nearest rendering.

    // For tracking vars and the fun sorta things.
    let mut hitstun: u32 = 0; // Yeah, the funny little hitstun thing returns.
    let mut high_score: u32 = 0; // And a high score?
    let mut score: u32 = 0; // Scooore!
    let mut lives: u32 = 0; // ...And a new one, lives.

    // Item management.
    let mut clear_screen: bool = false;
    let mut asteroid_cooldown: f32 = 0.0;

    // Musics and things.
    let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Creating our sinks.
    let sink_music = Sink::try_new(&stream_handle).unwrap(); // Nothing fancy this time.
    let sink_sfx = Sink::try_new(&stream_handle).unwrap(); // And SFX!
    let mut target_volume_music; // Mhm.
    let mut current_volume_music = 0.0; // Sure.

    // For Hecs
    let mut world = World::new();
    let mut particles = ParticleStorage::new();

    'running: loop {
        // And important vars.
        let current_time = macroquad::time::get_time();
        let delta_time = macroquad::time::get_frame_time();

        // drawing to the texture
        if is_key_pressed(KeyCode::Escape) {
            break 'running;
        }
        if is_key_pressed(KeyCode::R) {
            world_reset(&mut world);
            high_score = high_score.max(score);
            score = 0;
            lives = 3;
            play_audio!(
                sink_sfx,
                "assets/sfx/PlayerSpawn.wav",
                0.8,
                (rand::rand() % 100 / 100) as f32 + 0.9
            );
        }

        // HANDLING OUR DRAWING FUNCTIONS!
        // 0..100, 0..100 camera
        set_camera(&Camera2D {
            zoom: vec2(
                0.0001 * (DISPLAY_TARGET_HEIGHT as f32),
                0.0001 * (DISPLAY_TARGET_WIDTH as f32),
            ),
            target: vec2(0.0, 0.0),
            render_target: Some(render_target.clone()),
            ..Default::default()
        });

        // MUSIC SYSTEM
        //
        {
            // Updating target values.
            target_volume_music = (lives > 0) as u32 as f32;
            // Updating current values.
            current_volume_music = (current_volume_music * 0.99) + (target_volume_music * 0.01);
            // Setting the sinks.
            sink_music.set_volume(current_volume_music.clamp(0.0, MAX_VOLUME));

            // And putting in music.
            if sink_music.empty() {
                sink_music.append(
                    Decoder::new_wav(std::io::Cursor::new(&include_bytes!(
                        "assets/music/ProtoNibbz.wav"
                    )))
                    .unwrap(),
                );
            }
        }

        // PHYSICS SYSTEM
        //
        {
            // Woo, physics.
            // That means mutable bs, but at least we're prepared this time.
            // Once again, the Mario 64 thing, to give a faux sense of concurrence.
            if hitstun <= 0 {
                // The best time for some periodic functions
                if asteroid_cooldown <= 0.0 {
                    create_asteroid(&mut world);
                    asteroid_cooldown = 2.0;
                }
                asteroid_cooldown -= delta_time;

                for _i in 1..4 {
                    // Prepping a destruction system.
                    let mut entities_to_destroy: Vec<Entity> = Vec::new();
                    let mut asteroids_to_create: Vec<(f32, f32, f32)> = Vec::new();
                    let mut bullets_to_create: Vec<(f32, f32, f32, f32, f32)> = Vec::new();

                    // Update velocities.
                    for (_id, (position, velocity)) in
                        world.query_mut::<(&mut components::Position, &components::Velocity)>()
                    {
                        position.0 = position.0 + velocity.0 * delta_time; // Updating X with the current velocity while also wrapping it.
                        position.1 = position.1 + velocity.1 * delta_time; // Same for height.
                        position.2 = (position.2 + velocity.2 * delta_time) % 360.0;
                        // Same for degrees.
                    }
                    // Then get a collection of all possible, collidable objects.
                    let collidable_objects = world
                        .query::<(&components::Position, &components::Collidable)>()
                        .iter()
                        .map(|(e, (&p, &c))| (e, p, c))
                        .collect::<Vec<_>>();

                    for (id, (position, velocity, collidable)) in world.query_mut::<(
                        &mut components::Position,
                        &mut components::Velocity,
                        &components::Collidable,
                    )>() {
                        for other in &collidable_objects {
                            // Doing wrapping here, since we can detect the size of things.
                            if position.0 > 200.0 + collidable.0 {
                                position.0 = 55.0 - collidable.0
                            }
                            if position.0 < 55.0 - collidable.0 {
                                position.0 = 200.0 + collidable.0
                            }
                            if position.1 > 115.0 + collidable.0 {
                                position.1 = 30.0 - collidable.0
                            }
                            if position.1 < 30.0 - collidable.0 {
                                position.1 = 115.0 + collidable.0
                            }

                            // Now performing collision checks.
                            if id != other.0
                                && square_distance(position.0, position.1, other.1 .0, other.1 .1)
                                    < collidable.0.powf(2.0) + other.2 .0.powf(2.0)
                            {
                                // For our current collidable.
                                match collidable.1 {
                                    // If it's the player.
                                    components::CollidableType::PLAYER => {
                                        let mut damaged = false;
                                        match other.2 .1 {
                                            // We were hit by another player?
                                            components::CollidableType::PLAYER => {}
                                            // We were hit by an asteroid.
                                            components::CollidableType::ASTEROID => {
                                                particles.create_particle(
                                                    16,
                                                    (position.0, position.1),
                                                    (0.0, 0.0),
                                                    0.95,
                                                    1.0,
                                                    Color {
                                                        r: 0.9,
                                                        g: 0.9,
                                                        b: 0.9,
                                                        a: 1.0,
                                                    },
                                                    2.0,
                                                    (0.0, 0.0),
                                                    (50.0, 50.0),
                                                    0.5,
                                                    0.2,
                                                );
                                                damaged = true;
                                                play_audio!(
                                                    sink_sfx,
                                                    "assets/sfx/PlayerDeathRock.wav",
                                                    0.7,
                                                    (rand::rand() % 100 / 100) as f32 + 0.9
                                                );
                                            }
                                            // We were hit by a bullet.
                                            components::CollidableType::BULLET => {
                                                particles.create_particle(
                                                    16,
                                                    (position.0, position.1),
                                                    (0.0, 0.0),
                                                    0.95,
                                                    1.0,
                                                    Color {
                                                        r: 0.9,
                                                        g: 0.1,
                                                        b: 0.1,
                                                        a: 1.0,
                                                    },
                                                    2.0,
                                                    (0.0, 0.0),
                                                    (50.0, 50.0),
                                                    0.5,
                                                    0.2,
                                                );
                                                play_audio!(
                                                    sink_sfx,
                                                    "assets/sfx/PlayerDeathLaser.wav",
                                                    0.7,
                                                    (rand::rand() % 100 / 100) as f32 + 0.9
                                                );
                                                damaged = true;
                                            }
                                        }
                                        if damaged {
                                            lives -= 1;
                                            hitstun += 64 / (1 + lives);
                                            clear_screen = true;
                                            if lives <= 0 {
                                                entities_to_destroy.push(id);
                                                // explode.
                                            }
                                        }
                                    }
                                    // If it's an asteroid.
                                    components::CollidableType::ASTEROID => {
                                        match other.2 .1 {
                                            // We were hit by the player.
                                            components::CollidableType::PLAYER => {}
                                            // We were hit by another asteroid.
                                            components::CollidableType::ASTEROID => {
                                                velocity.0 = (position.0 - other.1 .0) / 2.0;
                                                velocity.1 = (position.1 - other.1 .1) / 2.0;
                                            }
                                            // We were hit by a bullet.
                                            components::CollidableType::BULLET => {
                                                if collidable.0 > 3.0 {
                                                    let new_vector = rotate_point(
                                                        (collidable.0, 0.0),
                                                        (rand::rand() % 360) as f32,
                                                    );
                                                    asteroids_to_create.push((
                                                        position.0 + new_vector.0,
                                                        position.1 + new_vector.0,
                                                        collidable.0 - 1.0,
                                                    ));
                                                    asteroids_to_create.push((
                                                        position.0 - new_vector.0,
                                                        position.1 - new_vector.1,
                                                        collidable.0 - 1.0,
                                                    ));
                                                }
                                                particles.create_particle(
                                                    16,
                                                    (position.0, position.1),
                                                    (0.0, 0.0),
                                                    0.95,
                                                    0.5,
                                                    Color {
                                                        r: 0.7,
                                                        g: 0.7,
                                                        b: 0.7,
                                                        a: 1.0,
                                                    },
                                                    0.4,
                                                    (0.0, 0.0),
                                                    (30.0, 30.0),
                                                    0.25,
                                                    0.1,
                                                );
                                                play_audio!(
                                                    sink_sfx,
                                                    "assets/sfx/AsteroidExplode.wav",
                                                    0.2,
                                                    (rand::rand() % 100 / 1000) as f32 + 0.9
                                                );
                                                score += 1;
                                                high_score = high_score.max(score);
                                                entities_to_destroy.push(id);
                                            }
                                        }
                                    }
                                    // If it's a bullet.
                                    components::CollidableType::BULLET => {
                                        // Just explode if we hit anything.
                                        particles.create_particle(
                                            8,
                                            (position.0, position.1),
                                            (0.0, 0.0),
                                            0.95,
                                            0.5,
                                            Color {
                                                r: 0.7,
                                                g: 0.0,
                                                b: 0.0,
                                                a: 1.0,
                                            },
                                            0.4,
                                            (0.0, 0.0),
                                            (10.0, 10.0),
                                            0.25,
                                            0.1,
                                        );
                                        entities_to_destroy.push(id);
                                    }
                                }
                            }
                        }
                    }

                    if clear_screen {
                        for collidable in collidable_objects {
                            match collidable.2 .1 {
                                // If it's the player.
                                components::CollidableType::PLAYER => {}
                                // If it's an asteroid.
                                components::CollidableType::ASTEROID => {
                                    particles.create_particle(
                                        16,
                                        (collidable.1 .0, collidable.1 .1),
                                        (0.0, 0.0),
                                        0.95,
                                        0.5,
                                        Color {
                                            r: 0.7,
                                            g: 0.7,
                                            b: 0.7,
                                            a: 1.0,
                                        },
                                        0.4,
                                        (0.0, 0.0),
                                        (0.5, 0.5),
                                        0.25,
                                        0.1,
                                    );
                                    entities_to_destroy.push(collidable.0);
                                }
                                // If it's a bullet.
                                components::CollidableType::BULLET => {
                                    // Just explode if we hit anything.
                                    particles.create_particle(
                                        8,
                                        (collidable.1 .0, collidable.1 .1),
                                        (0.0, 0.0),
                                        0.95,
                                        0.5,
                                        Color {
                                            r: 0.7,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        },
                                        0.4,
                                        (0.0, 0.0),
                                        (0.5, 0.5),
                                        0.25,
                                        0.1,
                                    );
                                    entities_to_destroy.push(collidable.0);
                                }
                            }
                        }
                        clear_screen = false;
                    }

                    // Updating player controls.
                    for (_id, (position, velocity, _controls)) in world.query_mut::<(
                        &components::Position,
                        &mut components::Velocity,
                        &components::Controllable,
                    )>() {
                        let new_velocity = rotate_point((0.0, -6.0 * delta_time), position.2);
                        velocity.0 =
                            velocity.0 * (1.0 - is_key_down(KeyCode::S) as u32 as f32 * delta_time);
                        velocity.1 =
                            velocity.1 * (1.0 - is_key_down(KeyCode::S) as u32 as f32 * delta_time);
                        velocity.2 =
                            velocity.2 * (1.0 - is_key_down(KeyCode::S) as u32 as f32 * delta_time);

                        velocity.0 = (velocity.0 * 0.995)
                            + (new_velocity.0 * (is_key_down(KeyCode::W) as u32 as f32));
                        velocity.1 = (velocity.1 * 0.995)
                            + (new_velocity.1 * (is_key_down(KeyCode::W) as u32 as f32));
                        velocity.2 = (velocity.2 * 0.975)
                            + 135.0
                                * delta_time
                                * ((is_key_down(KeyCode::D) as u32 as f32)
                                    - (is_key_down(KeyCode::A) as u32 as f32));

                        if is_key_pressed(KeyCode::Space) {
                            bullets_to_create
                                .push((position.0, position.1, position.2, velocity.0, velocity.1));
                            play_audio!(
                                sink_sfx,
                                "assets/sfx/PlayerShoot.wav",
                                0.15,
                                (rand::rand() % 100 / 1000) as f32 + 0.9
                            );
                        }
                        if is_key_down(KeyCode::W) {
                            let backwards = rotate_point((0.0, 4.0), position.2);
                            particles.create_particle(
                                1,
                                (position.0 + backwards.0, position.1 + backwards.1),
                                (backwards.0, backwards.1),
                                1.0,
                                1.0,
                                Color {
                                    r: 0.6,
                                    g: 0.6,
                                    b: 0.6,
                                    a: 1.0,
                                },
                                1.0,
                                (0.0, 0.0),
                                (0.0, 0.0),
                                0.0,
                                0.2,
                            );
                        }
                        if is_key_down(KeyCode::A) {
                            let backwards = rotate_point((2.0, -2.0), position.2);
                            particles.create_particle(
                                1,
                                (position.0 + backwards.0, position.1 + backwards.1),
                                (backwards.0, backwards.1),
                                1.0,
                                1.0,
                                Color {
                                    r: 0.5,
                                    g: 0.5,
                                    b: 0.5,
                                    a: 1.0,
                                },
                                0.7,
                                (0.0, 0.0),
                                (0.0, 0.0),
                                0.0,
                                0.2,
                            );
                        }
                        if is_key_down(KeyCode::D) {
                            let backwards = rotate_point((-2.0, -2.0), position.2);
                            particles.create_particle(
                                1,
                                (position.0 + backwards.0, position.1 + backwards.1),
                                (backwards.0, backwards.1),
                                1.0,
                                1.0,
                                Color {
                                    r: 0.5,
                                    g: 0.5,
                                    b: 0.5,
                                    a: 1.0,
                                },
                                0.7,
                                (0.0, 0.0),
                                (0.0, 0.0),
                                0.0,
                                0.2,
                            );
                        }
                    }

                    // Destroying all things meant to be destroyed.
                    entities_to_destroy.dedup();
                    for entity in entities_to_destroy {
                        // Let's just get its position and type to make an explosion effect.
                        // Finally getting rid of it.
                        match world.despawn(entity) {
                            _ => {}
                        }
                    }

                    // And making new things.
                    asteroids_to_create.dedup();
                    for asteroid in asteroids_to_create {
                        create_asteroid_point(&mut world, (asteroid.0, asteroid.1), asteroid.2);
                    }

                    bullets_to_create.dedup();
                    for bullet in bullets_to_create {
                        let bullet_position = rotate_point((0.0, -2.0), bullet.2);
                        let bulet_velocity = rotate_point((0.0, -20.0), bullet.2);
                        world.spawn((
                            components::Position(
                                bullet.0 + bullet_position.0,
                                bullet.1 + bullet_position.1,
                                bullet.2,
                            ),
                            components::Velocity(
                                bullet.3 + bulet_velocity.0,
                                bullet.4 + bulet_velocity.1,
                                0.0,
                            ),
                            components::Draw(
                                Color {
                                    r: 1.0,
                                    g: 0.2,
                                    b: 0.2,
                                    a: 1.0,
                                },
                                vec![(0.0, -1.0), (0.0, 1.0)],
                            ),
                            components::Collidable(1.0, components::CollidableType::BULLET),
                        ));
                    }
                }
            } else {
                hitstun -= 1;
            }
        }

        // DRAW SYSTEM
        //
        {
            // Clearing the background.
            clear_background(Color {
                r: hitstun as f32 / 256.0,
                g: hitstun as f32 / 256.0,
                b: hitstun as f32 / 256.0,
                a: 1.0,
            });

            // Particles first, to render under everything.
            particles.particles_container.iter_mut().for_each(|part| {
                draw_line(
                    (part.position.0 % crate::DISPLAY_TARGET_WIDTH as f32)
                        - (crate::DISPLAY_TARGET_WIDTH as f32 / 2.0),
                    (part.position.1 % crate::DISPLAY_TARGET_HEIGHT as f32)
                        - (crate::DISPLAY_TARGET_HEIGHT as f32 / 2.0),
                    (part.position.0 % crate::DISPLAY_TARGET_WIDTH as f32)
                        - (crate::DISPLAY_TARGET_WIDTH as f32 / 2.0)
                        + (part.velocity.0 * part.size * 0.125),
                    (part.position.1 % crate::DISPLAY_TARGET_HEIGHT as f32)
                        - (crate::DISPLAY_TARGET_HEIGHT as f32 / 2.0)
                        + (part.velocity.1 * part.size * 0.125),
                    clamp(
                        part.size
                            * ((current_time - part.deathtime) / (part.birthtime - part.deathtime))
                                .clamp(0.0, 1.0) as f32,
                        0.0,
                        f32::MAX,
                    ),
                    Color {
                        r: part.color.r,
                        g: part.color.g,
                        b: part.color.b,
                        a: part.color.a,
                    },
                );

                part.position = (
                    part.position.0 + part.velocity.0 * delta_time,
                    part.position.1 + part.velocity.1 * delta_time,
                );

                part.velocity = (part.velocity.0 * part.drag, part.velocity.1 * part.drag);
            });
            particles
                .particles_container
                .retain(|&part| part.deathtime > current_time);

            // And UI to draw underneath it all.
            draw_text_ex(
                &format!("High Score: {}", high_score),
                -68.0,
                -32.0,
                TextParams {
                    font_size: 340,
                    font_scale: 0.0001 * (DISPLAY_TARGET_WIDTH as f32),
                    rotation: 0.0,
                    color: GRAY,
                    ..Default::default()
                },
            );
            draw_text_ex(
                &format!("Score: {}", score),
                -68.0,
                -26.0,
                TextParams {
                    font_size: 340,
                    font_scale: 0.0001 * (DISPLAY_TARGET_WIDTH as f32),
                    rotation: 0.0,
                    color: GRAY,
                    ..Default::default()
                },
            );
            draw_text_ex(
                &format!("Lives: {}", lives),
                -68.0,
                -20.0,
                TextParams {
                    font_size: 340,
                    font_scale: 0.0001 * (DISPLAY_TARGET_WIDTH as f32),
                    rotation: 0.0,
                    color: GRAY,
                    ..Default::default()
                },
            );
            if lives <= 0 {
                draw_text_ex(
                    "Press R to Start!",
                    -29.0,
                    0.0,
                    TextParams {
                        font_size: 340,
                        font_scale: 0.0001 * (DISPLAY_TARGET_WIDTH as f32),
                        rotation: 0.0,
                        color: GRAY,
                        ..Default::default()
                    },
                );
                draw_text_ex(
                    "Use WASD to move!",
                    -29.0,
                    6.0,
                    TextParams {
                        font_size: 340,
                        font_scale: 0.0001 * (DISPLAY_TARGET_WIDTH as f32),
                        rotation: 0.0,
                        color: GRAY,
                        ..Default::default()
                    },
                );
            }

            // For debugging the game.
            if crate::DEBUG_ENABLED {
                world
                    .query::<(&components::Position, &components::Collidable)>()
                    .iter()
                    .for_each(|collidable| {
                        draw_circle(
                            (collidable.1 .0 .0 % crate::DISPLAY_TARGET_WIDTH as f32)
                                - (crate::DISPLAY_TARGET_WIDTH as f32 / 2.0),
                            (collidable.1 .0 .1 % crate::DISPLAY_TARGET_HEIGHT as f32)
                                - (crate::DISPLAY_TARGET_HEIGHT as f32 / 2.0),
                            collidable.1 .1 .0,
                            match collidable.1 .1 .1 {
                                components::CollidableType::ASTEROID => Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 1.0,
                                    a: 0.5,
                                },
                                components::CollidableType::PLAYER => Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 0.5,
                                },
                                components::CollidableType::BULLET => Color {
                                    r: 1.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.5,
                                },
                            },
                        );
                    });
            }

            // All drawable, transformable objects.
            world
                .query::<(&components::Position, &components::Draw)>() // Querying the world.
                .iter() // Iterating over it.
                .for_each(|drawable| {
                    // For each drawable we find.
                    let drawable_x = (drawable.1 .0 .0 % crate::DISPLAY_TARGET_WIDTH as f32)
                        - (crate::DISPLAY_TARGET_WIDTH as f32 / 2.0);
                    let drawable_y = (drawable.1 .0 .1 % crate::DISPLAY_TARGET_HEIGHT as f32)
                        - (crate::DISPLAY_TARGET_HEIGHT as f32 / 2.0);
                    if drawable.1 .1 .1.len() > 1 {
                        for vector_index in 1..drawable.1 .1 .1.len() {
                            let start_point = rotate_point(
                                (
                                    drawable.1 .1 .1[vector_index - 1].0,
                                    drawable.1 .1 .1[vector_index - 1].1,
                                ),
                                drawable.1 .0 .2,
                            );
                            let end_point = rotate_point(
                                (
                                    drawable.1 .1 .1[vector_index].0,
                                    drawable.1 .1 .1[vector_index].1,
                                ),
                                drawable.1 .0 .2,
                            );
                            draw_line(
                                drawable_x + start_point.0,
                                drawable_y + start_point.1,
                                drawable_x + end_point.0,
                                drawable_y + end_point.1,
                                0.75,
                                drawable.1 .1 .0,
                            );
                        }
                    } else {
                        draw_circle(drawable_x, drawable_y, 1.0, WHITE);
                    }
                });
        }

        // DRAWING OUR TEXTURE TO THE SCREEN
        set_default_camera(); // Setting our camera.
        clear_background(Color {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        }); // Clearing the camera.
        let scaling_factor = (screen_width() / DISPLAY_TARGET_WIDTH as f32)
            .min(screen_height() / DISPLAY_TARGET_HEIGHT as f32);
        draw_texture_ex(
            &render_target.texture,
            (screen_width() - (scaling_factor * DISPLAY_TARGET_WIDTH as f32)) * 0.5,
            (screen_height() - (scaling_factor * DISPLAY_TARGET_HEIGHT as f32)) * 0.5,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    scaling_factor * DISPLAY_TARGET_WIDTH as f32,
                    scaling_factor * DISPLAY_TARGET_HEIGHT as f32,
                )),
                ..Default::default()
            },
        );
        gl_use_default_material(); // Resetting the material.
        next_frame().await; // Next frame time.
    }
}
