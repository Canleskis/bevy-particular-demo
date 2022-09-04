use crate::LoadedScene;
use bevy::{
    ecs::{
        schedule::ShouldRun,
        system::{Commands, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    scene::SceneBundle,
};

pub fn scene_changed(scene: Res<LoadedScene>) -> ShouldRun {
    if scene.is_changed() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn scene_cleanup_and_reload(mut commands: Commands, mut scene: ResMut<LoadedScene>) {
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
