use crate::SimulationScene;
use bevy::ecs::{entity::Entity, system::EntityCommands};

pub struct LoadedScene {
    scene: SimulationScene,
    entity: Option<Entity>,
}

impl LoadedScene {
    pub fn new(scene: SimulationScene) -> Self {
        Self {
            scene,
            entity: None,
        }
    }

    pub fn load(&mut self, scene: SimulationScene) {
        self.scene = scene;
    }

    pub(crate) fn spawned(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub fn entity(&self) -> Entity {
        self.entity.expect("No entity for {self.scene}")
    }

    pub fn get_entity(&self) -> Option<Entity> {
        self.entity
    }

    pub fn instance(&self, scene_commands: EntityCommands) {
        self.scene.instance(scene_commands)
    }

    pub fn max_spawnable_mass(&self) -> f32 {
        self.scene.max_spawnable_mass()
    }
}
