use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::Inspectable;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use heron::{rapier_plugin::rapier2d::prelude::IntegrationParameters, should_run};

pub type PositionCache = HashMap<u32, (Vec3, usize)>;

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::default())
            .insert_resource(PositionCache::default())
            .add_system(changed)
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_run_criteria(should_run)
                    .with_system(draw_trails),
            );
    }
}

#[derive(Component, Inspectable)]
pub struct Trail {
    pub length: f32,
    pub resolution: usize,
}

impl Trail {
    pub fn new(length: f32, resolution: usize) -> Self {
        Self { length, resolution }
    }
}

fn changed(mut cache: ResMut<PositionCache>, removed: RemovedComponents<Trail>) {
    for entity in removed.iter() {
        cache.remove(&entity.id());
    }
}

fn draw_trails(
    integration: Res<IntegrationParameters>,
    mut lines: ResMut<DebugLines>,
    mut cache: ResMut<PositionCache>,
    query: Query<(Entity, &GlobalTransform, &Trail)>,
) {
    for (entity, transform, draw_trail) in query.iter() {
        if let Some((last_position, last_iteration)) = cache.get_mut(&entity.id()) {
            if *last_iteration == draw_trail.resolution {
                lines.line(*last_position, transform.translation(), draw_trail.length);
                *last_position = transform.translation();
                *last_iteration = 0;
            } else {
                lines.line(*last_position, transform.translation(), integration.dt);
                *last_iteration += 1;
            }
        } else {
            cache.insert(entity.id(), (transform.translation(), 0));
        }
    }
}
