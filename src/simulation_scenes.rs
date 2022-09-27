use std::{
    f32::consts::{PI, TAU},
    fmt::Display,
};

use bevy::{
    ecs::system::EntityCommands,
    prelude::{BuildChildren, Color, Name, Vec3},
};
use bevy_egui::egui::{self, Slider, Ui};
use heron::Velocity;
use rand::{thread_rng, Rng};

use crate::{
    nbody::PointMass, simulation_scene::Spawnable, trails::Trail, BodyBundle, SceneData, G,
};

#[derive(Clone)]
pub struct Orbits {
    main_mass: f32,
    main_density: f32,
    bodies_count: usize,
    bodies_density: f32,
    bodies_max_pos: f32,
    bodies_min_mass: f32,
    bodies_max_mass: f32,
    bodies_with_mass: bool,
}

impl Default for Orbits {
    fn default() -> Self {
        Self {
            main_mass: 1E5,
            main_density: 20.0,
            bodies_count: 1000,
            bodies_density: 0.1,
            bodies_max_pos: 1000.0,
            bodies_min_mass: 1.0,
            bodies_max_mass: 10.0,
            bodies_with_mass: true,
        }
    }
}

impl Orbits {
    fn main_radius(&self) -> f32 {
        (self.main_mass / (self.main_density * PI)).sqrt()
    }

    fn min_spawnable_position(&self) -> f32 {
        ((self.bodies_count as f32).sqrt() * self.bodies_max_mass).max(self.main_radius() * 4.0)
    }
}

impl Display for Orbits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Orbits")
    }
}

impl SceneData for Orbits {
    fn instance(&self, mut scene_commands: EntityCommands) {
        let mut rng = thread_rng();

        scene_commands.with_children(|child| {
            child.spawn_bundle(BodyBundle::new(
                Vec3::ZERO,
                Velocity::from_linear(Vec3::ZERO),
                self.main_density,
                self.main_mass,
                PointMass::HasGravity {
                    mass: self.main_mass,
                },
                Color::WHITE,
            ));

            let min_radius = 2.0 * self.main_radius();
            let min_p_sqrt = min_radius * min_radius / (self.bodies_max_pos * self.bodies_max_pos);

            for i in 0..self.bodies_count {
                let radius = self.bodies_max_pos * rng.gen_range(min_p_sqrt..=1.0).sqrt();
                let theta = rng.gen_range(0.0..=TAU);

                let position = Vec3::new(radius * theta.cos(), radius * theta.sin(), 0.0);

                let mass = rng.gen_range(1.0..=self.bodies_max_mass);

                let (density, physics_mass, point_mass) = if self.bodies_with_mass {
                    (self.bodies_density, mass, PointMass::HasGravity { mass })
                } else {
                    (
                        self.bodies_density / 100.0,
                        self.bodies_min_mass / 100.0,
                        PointMass::AffectedByGravity,
                    )
                };

                let direction = position - Vec3::ZERO;
                let distance = direction.length_squared();

                let vel = (G * (self.main_mass + mass)).sqrt() * distance.powf(-0.75);
                let velvec = Vec3::new(-direction.y * vel, direction.x * vel, 0.0);

                let mut random_color = || rng.gen_range(0.0..=1.0_f32);
                let (r, g, b) = (random_color(), random_color(), random_color());

                child
                    .spawn_bundle(BodyBundle::new(
                        position,
                        Velocity::from_linear(velvec),
                        density,
                        physics_mass,
                        point_mass,
                        Color::rgb(r, g, b),
                    ))
                    .insert(Name::new(format!("Particle {}", i)));
            }
        });
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        ui.separator();

        ui.label("Central body:");
        {
            ui.add(
                Slider::new(&mut self.main_mass, 1E3..=1E6)
                    .logarithmic(true)
                    .text("Mass"),
            );
        }

        ui.separator();

        ui.label("Orbiting bodies:");
        {
            ui.add(
                Slider::new(&mut self.bodies_count, 1..=5000)
                    .text(" Body count")
                    .logarithmic(true),
            );

            let min_pos = self.min_spawnable_position();
            ui.add(
                Slider::new(&mut self.bodies_max_pos, min_pos..=10000.0)
                    .text(" Position range")
                    .logarithmic(true)
                    .integer(),
            );

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let (min_mass, max_mass) =
                    (self.bodies_min_mass, self.spawnable().max_mass().unwrap());
                ui.add_enabled(
                    self.bodies_with_mass,
                    Slider::new(&mut self.bodies_max_mass, min_mass..=max_mass),
                );

                ui.toggle_value(&mut self.bodies_with_mass, "Mass range");
            });
        }
    }

    fn spawnable(&self) -> Spawnable {
        Spawnable::Massive {
            min_mass: 1.0,
            max_mass: self.main_mass / 5E3,
            density: 0.1,
        }
    }
}

#[derive(Clone)]
pub struct Figure8 {
    radius: f32,
    mass: f32,
}

impl Default for Figure8 {
    fn default() -> Self {
        Self {
            radius: 30.0,
            mass: 1E5,
        }
    }
}

impl Display for Figure8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Figure8")
    }
}

impl SceneData for Figure8 {
    fn instance(&self, mut scene_commands: EntityCommands) {
        let mass = self.mass;
        let density = 0.5 * mass / (self.radius.powi(2) * PI);
        let distance = (G * mass).cbrt();

        let pos1 = Vec3::new(-0.970_004_4, 0.243_087_53, 0.0) * distance;
        let pos2 = Vec3::ZERO;

        let vel1 = Vec3::new(0.466_203_7, 0.432_365_73, 0.0) * distance;
        let vel2 = -2.0 * vel1;

        scene_commands.with_children(|child| {
            child
                .spawn_bundle(BodyBundle::new(
                    pos1,
                    Velocity::from_linear(vel1),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));

            child
                .spawn_bundle(BodyBundle::new(
                    -pos1,
                    Velocity::from_linear(vel1),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));

            child
                .spawn_bundle(BodyBundle::new(
                    pos2,
                    Velocity::from_linear(vel2),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));
        });
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.radius, 5.0..=100.0)
                .text("Radius")
                .logarithmic(true)
                .integer(),
        );
    }

    fn spawnable(&self) -> Spawnable {
        Spawnable::Massless {
            density: 3E-7 * self.mass / (self.radius * self.radius * PI),
        }
    }
}

#[derive(Clone)]
pub struct TernaryOrbit {
    radius: f32,
    mass: f32,
}

impl Default for TernaryOrbit {
    fn default() -> Self {
        Self {
            radius: 20.0,
            mass: 1E5,
        }
    }
}

impl Display for TernaryOrbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TernaryOrbit")
    }
}

impl SceneData for TernaryOrbit {
    fn instance(&self, mut scene_commands: EntityCommands) {
        let mass: f32 = self.mass;
        let density = mass / (self.radius.powi(2) * PI);
        let distance = (G * mass).cbrt();

        let pos1 = Vec3::new(1.0, 0.0, 0.0) * distance;
        let pos2 = Vec3::new(-0.5, 3.0_f32.sqrt() / 2.0, 0.0) * distance;
        let pos3 = Vec3::new(-0.5, -(3.0_f32.sqrt()) / 2.0, 0.0) * distance;

        let vel1 = Vec3::new(0.0, 1.0, 0.0) * distance * 0.5;
        let vel2 = Vec3::new(-(3.0_f32.sqrt()) / 2.0, -0.5, 0.0) * distance * 0.5;
        let vel3 = Vec3::new(3.0_f32.sqrt() / 2.0, -0.5, 0.0) * distance * 0.5;

        scene_commands.with_children(|child| {
            child
                .spawn_bundle(BodyBundle::new(
                    pos1,
                    Velocity::from_linear(vel1),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));

            child
                .spawn_bundle(BodyBundle::new(
                    pos2,
                    Velocity::from_linear(vel2),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));

            child
                .spawn_bundle(BodyBundle::new(
                    pos3,
                    Velocity::from_linear(vel3),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));
        });
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.radius, 5.0..=100.0)
                .text("Radius")
                .logarithmic(true)
                .integer(),
        );
    }

    fn spawnable(&self) -> Spawnable {
        Spawnable::Massless { density: 1E-4 }
    }
}

#[derive(Clone)]
pub struct DoubleOval {
    radius: f32,
    mass: f32,
}

impl Default for DoubleOval {
    fn default() -> Self {
        Self {
            radius: 20.0,
            mass: 1E5,
        }
    }
}

impl Display for DoubleOval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DoubleOval")
    }
}

impl SceneData for DoubleOval {
    fn instance(&self, mut scene_commands: EntityCommands) {
        let mass: f32 = self.mass;
        let density = mass / (self.radius.powi(2) * PI);
        let distance = (G * mass).cbrt();

        let pos1 = Vec3::new(0.486_657_68, 0.755_041_9, 0.0) * distance;
        let pos2 = Vec3::new(-0.681_738, 0.293_660_22, 0.0) * distance;
        let pos3 = Vec3::new(-0.022_596_328, -0.612_645_6, 0.0) * distance;

        let vel1 = Vec3::new(-0.182_709_86, 0.363_013_3, 0.0) * distance;
        let vel2 = Vec3::new(-0.579_074_9, -0.748_157_5, 0.0) * distance;
        let vel3 = Vec3::new(0.761_784_8, 0.385_144_2, 0.0) * distance;

        scene_commands.with_children(|child| {
            child
                .spawn_bundle(BodyBundle::new(
                    pos1,
                    Velocity::from_linear(vel1),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));

            child
                .spawn_bundle(BodyBundle::new(
                    pos2,
                    Velocity::from_linear(vel2),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));

            child
                .spawn_bundle(BodyBundle::new(
                    pos3,
                    Velocity::from_linear(vel3),
                    density,
                    mass,
                    PointMass::HasGravity { mass },
                    Color::WHITE,
                ))
                .insert(Trail::new(15.0, 1));
        });
    }

    fn show_ui(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.radius, 5.0..=50.0)
                .text("Radius")
                .logarithmic(true)
                .integer(),
        );
    }

    fn spawnable(&self) -> Spawnable {
        Spawnable::Massless { density: 1E-4 }
    }
}
