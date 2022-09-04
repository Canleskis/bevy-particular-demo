mod loaded_scene;
mod scene_data;
mod systems;

pub use loaded_scene::LoadedScene;
pub use scene_data::{SceneData, SimulationScene};

use bevy::{
    app::{App, Plugin},
    ecs::schedule::ParallelSystemDescriptorCoercion,
};

pub type SceneCollection = Vec<SimulationScene>;

pub struct SimulationScenePlugin {
    scenes: SceneCollection,
}

impl SimulationScenePlugin {
    pub fn new(scenes: SceneCollection) -> Self {
        Self { scenes }
    }
}

impl Plugin for SimulationScenePlugin {
    fn build(&self, app: &mut App) {
        let scenes = self.scenes.clone();
        let loaded = scenes.first().expect("Missing scenes!").clone();

        app.insert_resource(scenes)
            .insert_resource(LoadedScene::new(loaded))
            .add_system(
                systems::scene_cleanup_and_reload.with_run_criteria(systems::scene_changed),
            );
    }
}
