use crate::G;

use bevy::math::Vec3;
use bevy::prelude::*;
use heron::{should_run, Acceleration};
use particular::prelude::*;

#[particle(3)]
pub struct Body {
    position: Vec3,
    mu: f32,
    entity: Entity,
}

impl Body {
    pub fn new(position: Vec3, mu: f32, entity: Entity) -> Self {
        Self {
            position,
            mu,
            entity,
        }
    }
}

#[derive(Component)]
pub enum PointMass {
    HasGravity { mass: f32 },
    AffectedByGravity,
}

pub struct ParticularPlugin;

impl Plugin for ParticularPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ParticleSet::<Body>::new())
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_run_criteria(should_run)
                    .with_system(sync_particle_set),
            )
            .add_system_set_to_stage(
                CoreStage::Update,
                SystemSet::new()
                    .with_run_criteria(should_run)
                    .with_system(accelerate_particles),
            );
    }
}

fn sync_particle_set(
    mut particle_set: ResMut<ParticleSet<Body>>,
    query: Query<(Entity, &GlobalTransform, &PointMass)>,
) {
    *particle_set = ParticleSet::new();
    query.for_each(|(entity, tranform, point_mass)| {
        let position = tranform.translation();
        match point_mass {
            PointMass::HasGravity { mass } => {
                particle_set.add_massive(Body::new(position, mass * G, entity))
            }
            PointMass::AffectedByGravity => {
                particle_set.add_massless(Body::new(position, 0.0, entity))
            }
        };
    })
}

fn accelerate_particles(
    mut particle_set: ResMut<ParticleSet<Body>>,
    mut query: Query<&mut Acceleration, With<PointMass>>,
) {
    for (gravity, body) in particle_set.result() {
        if let Ok(mut acceleration) = query.get_mut(body.entity) {
            acceleration.linear = gravity;
        }
    }
}
