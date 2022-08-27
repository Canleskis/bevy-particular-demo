use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::Inspectable;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use heron::{rapier_plugin::rapier2d::prelude::IntegrationParameters, should_run};

pub struct TrailsPlugin;

impl Plugin for TrailsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::default())
            .add_system_set_to_stage(
                CoreStage::Last,
                SystemSet::new()
                    .with_run_criteria(should_run)
                    .with_system(draw_trails),
            );
    }
}

#[derive(Component, Inspectable)]
pub struct DrawTrail {
    pub length: f32,
    pub resolution: usize,
}

impl DrawTrail {
    pub fn new(length: f32, resolution: usize) -> Self {
        Self { length, resolution }
    }
}

fn draw_trails(
    integration: Res<IntegrationParameters>,
    mut lines: ResMut<DebugLines>,
    mut last_positions_and_iterations: Local<HashMap<u32, (Vec3, usize)>>,
    query: Query<(Entity, &GlobalTransform, &DrawTrail)>,
) {
    for (entity, transform, draw_trail) in query.iter() {
        if let Some((last_position, last_iteration)) =
            last_positions_and_iterations.get_mut(&entity.id())
        {
            if *last_iteration == draw_trail.resolution {
                lines.line(*last_position, transform.translation(), draw_trail.length);
                *last_position = transform.translation();
                *last_iteration = 0;
            } else {
                lines.line(*last_position, transform.translation(), integration.dt);
                *last_iteration += 1;
            }
        } else {
            last_positions_and_iterations.insert(entity.id(), (transform.translation(), 0));
        }
    }
}
