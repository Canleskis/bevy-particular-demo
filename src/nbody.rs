use crate::G;
use bevy::prelude::*;
use heron::{should_run, Acceleration};
use particular::prelude::*;

pub struct Body {
    position: Vec2,
    mu: f32,
    entity: Entity,
}

impl Body {
    pub fn new(position: Vec2, mu: f32, entity: Entity) -> Self {
        Self {
            position,
            mu,
            entity,
        }
    }
}

impl Particle for Body {
    type Vector = Vec2;

    fn position(&self) -> Vec2 {
        self.position
    }

    fn mu(&self) -> f32 {
        self.mu
    }
}

#[derive(Component)]
pub enum PointMass {
    HasGravity { mass: f32 },
    _AffectedByGravity,
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
        match point_mass {
            PointMass::HasGravity { mass } => particle_set.add_massive(Body::new(
                tranform.translation().truncate(),
                mass * G,
                entity,
            )),
            PointMass::_AffectedByGravity => {
                particle_set.add_massless(Body::new(tranform.translation().truncate(), 0.0, entity))
            }
        };
    })
}

fn accelerate_particles(
    mut particle_set: ResMut<ParticleSet<Body>>,
    mut query: Query<&mut Acceleration, With<PointMass>>,
) {
    for (body, gravity) in particle_set.result() {
        if let Ok(mut acceleration) = query.get_mut(body.entity) {
            acceleration.linear = gravity.extend(0.0);
        }
    }
}
