use std::{
    f32::consts::{PI, TAU},
    fmt::Display,
};

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_egui::egui::{Slider, Ui};
use heron::prelude::*;
use rand::prelude::*;

use crate::{nbody::PointMass, BodyBundle, SceneData, G};

#[derive(Clone)]
pub struct Orbits {
    main_mass: f32,
    main_density: f32,
    bodies_count: usize,
    bodies_density: f32,
    bodies_range_pos: f32,
    bodies_range_mass: f32,
}

impl Orbits {
    fn main_radius(&self) -> f32 {
        (self.main_mass / (self.main_density * PI)).sqrt()
    }

    fn min_spawnable_position(&self) -> f32 {
        ((self.bodies_count as f32).sqrt() * self.bodies_range_mass).max(self.main_radius() * 4.0)
    }
}

impl Default for Orbits {
    fn default() -> Self {
        Self {
            main_mass: 1E5,
            main_density: 20.0,
            bodies_count: 1000,
            bodies_density: 0.1,
            bodies_range_pos: 1000.0,
            bodies_range_mass: 10.0,
        }
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
            let min_p_sqrt =
                min_radius * min_radius / (self.bodies_range_pos * self.bodies_range_pos);

            for i in 0..self.bodies_count {
                let radius = self.bodies_range_pos * rng.gen_range(min_p_sqrt..=1.0).sqrt();
                let theta = rng.gen_range(0.0..=TAU);

                let position = Vec3::new(radius * theta.cos(), radius * theta.sin(), 0.0);

                let mass = rng.gen_range(0.0..=self.bodies_range_mass);

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
                        self.bodies_density,
                        mass.max(1.0),
                        PointMass::HasGravity { mass },
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
                    .text("Body count")
                    .logarithmic(true),
            );

            let min_pos = self.min_spawnable_position();
            ui.add(
                Slider::new(&mut self.bodies_range_pos, min_pos..=10000.0)
                    .text("Position range")
                    .logarithmic(true)
                    .integer(),
            );

            let max_mass = self.max_spawnable_mass();
            ui.add(Slider::new(&mut self.bodies_range_mass, 0.0..=max_mass).text("Mass range"));
        }
    }

    fn max_spawnable_mass(&self) -> f32 {
        self.main_mass / 5E3
    }
}

#[derive(Clone)]
pub struct Empty {}

impl Display for Empty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Empty")
    }
}

impl SceneData for Empty {
    fn instance(&self, _: EntityCommands) {}

    fn show_ui(&mut self, _: &mut Ui) {}

    fn max_spawnable_mass(&self) -> f32 {
        100.0
    }
}
