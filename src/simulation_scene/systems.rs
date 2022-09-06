use crate::LoadedScene;
use bevy::{
    ecs::{
        change_detection::DetectChanges,
        schedule::ShouldRun,
        system::{Commands, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    scene::SceneBundle,
};
use bevy_prototype_debug_lines::DebugLines;

// Cannot use this as run criteria as changes done to `LoadedScene` in systems with this run criteria are also detected.
pub fn _scene_changed(scene: Res<LoadedScene>) -> ShouldRun {
    if scene.is_changed() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn scene_cleanup_and_reload(
    mut commands: Commands,
    mut scene: ResMut<LoadedScene>,
    mut lines: ResMut<DebugLines>,
) {
    if scene.is_changed() {
        *lines = DebugLines::default();

        let entity_commands = if let Some(entity) = scene.get_entity() {
            let mut commands = commands.entity(entity);
            commands.despawn_descendants();
            commands
        } else {
            let commands = commands.spawn_bundle(SceneBundle::default());
            scene.spawned(commands.id());
            commands
        };

        scene.instance(entity_commands);
    }
}
