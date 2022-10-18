mod nbody;
mod simulation_scene;
mod simulation_scenes;
mod trails;

use std::f32::consts::PI;
use std::time::Duration;

use bevy_egui::egui;
use nbody::{ParticularPlugin, PointMass};
use simulation_scene::*;
use simulation_scenes::{DoubleOval, Figure8, Orbits, TernaryOrbit};
use trails::{Trail, TrailsPlugin};

use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::time::FixedTimestep;
use bevy::{prelude::*, window::PresentMode};
use bevy_egui::{
    egui::{Slider, Window},
    EguiContext, EguiPlugin,
};
use bevy_mouse_tracking_plugin::{prelude::*, MainCamera, MousePosWorld};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::DebugLines;
use heron::{prelude::*, PhysicsSteps};

const G: f32 = 1000.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Particular demo".to_string(),
            #[cfg(not(target_arch = "wasm32"))]
            width: 1500.0,
            #[cfg(not(target_arch = "wasm32"))]
            height: 900.0,
            present_mode: PresentMode::AutoNoVsync,
            fit_canvas_to_parent: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs_f32(1.0),
            ..default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PanCamPlugin)
        .add_plugin(MousePosPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(TrailsPlugin)
        .add_plugin(ParticularPlugin)
        .add_plugin(SimulationScenePlugin)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PhysicsSteps::from_steps_per_seconds(60.0))
        .insert_resource(
            SceneCollection::new()
                .with_scene::<Empty>()
                .with_scene::<Orbits>()
                .with_scene::<Figure8>()
                .with_scene::<DoubleOval>()
                .with_scene::<TernaryOrbit>(),
        )
        .insert_resource(LoadedScene::new(Orbits::default()))
        .init_resource::<BodyInfo>()
        .add_state(SimulationState::Running)
        .add_startup_system(spawn_camera)
        .add_startup_system(setup_ui_fps)
        .add_system(update_ui_fps.with_run_criteria(FixedTimestep::step(0.25)))
        .add_system(place_body)
        .add_system(body_info_window)
        .add_system_set(SystemSet::on_enter(SimulationState::Paused).with_system(pause_physics))
        .add_system_set(SystemSet::on_exit(SimulationState::Paused).with_system(resume_physics))
        .add_system(pause_resume)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .add_world_tracking()
        .insert(MainCamera)
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
            ..default()
        });
}

#[derive(Component)]
struct FpsText;

fn setup_ui_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "FPS: ",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 20.0,
                        color: Color::GRAY,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: Color::GRAY,
                }),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(FpsText);
}

fn update_ui_fps(mut query_text: Query<&mut Text, With<FpsText>>, diagnostic: Res<Diagnostics>) {
    let fps = diagnostic
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average());
    if let Some(fps) = fps {
        for mut text in &mut query_text {
            text.sections[1].value = format!("{fps:.1}");
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum SimulationState {
    Running,
    Paused,
}

fn pause_resume(keys: Res<Input<KeyCode>>, mut state: ResMut<State<SimulationState>>) {
    if keys.just_pressed(KeyCode::Space) {
        match state.current() {
            SimulationState::Running => state.set(SimulationState::Paused).unwrap(),
            SimulationState::Paused => state.set(SimulationState::Running).unwrap(),
        }
    }
}

fn pause_physics(mut physics: ResMut<PhysicsTime>) {
    physics.pause();
}

fn resume_physics(mut physics: ResMut<PhysicsTime>) {
    physics.resume();
}

struct BodyInfo {
    position: Option<Vec3>,
    mass: f32,
    with_mass: bool,
    with_trail: bool,
}

impl Default for BodyInfo {
    fn default() -> Self {
        Self {
            position: None,
            mass: 20.0,
            with_mass: true,
            with_trail: false,
        }
    }
}

fn body_info_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut body_info: ResMut<BodyInfo>,
    scene: Res<LoadedScene>,
) {
    Window::new("Body spawner").show(egui_ctx.ctx_mut(), |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            if let Some((min_mass, max_mass)) = scene.spawnable().mass_range() {
                ui.add_enabled(
                    body_info.with_mass,
                    Slider::new(&mut body_info.mass, min_mass..=max_mass),
                );

                ui.toggle_value(&mut body_info.with_mass, "Mass");
            }
        });

        ui.checkbox(&mut body_info.with_trail, "Draw trail");
    });

    if egui_ctx.ctx_mut().wants_pointer_input() {
        body_info.position = None;
    }
}

fn place_body(
    mut commands: Commands,
    mut click_event: EventReader<MouseButtonInput>,
    mut lines: ResMut<DebugLines>,
    mut body_info: ResMut<BodyInfo>,
    mouse_pos: Res<MousePosWorld>,
    scene: Res<LoadedScene>,
    asset_server: Res<AssetServer>,
) {
    let mouse_pos = mouse_pos.truncate().extend(0.0);

    for event in click_event.iter() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => body_info.position = Some(mouse_pos),
                ButtonState::Released => {
                    if let Some(place_pos) = body_info.position.take() {
                        let spawnable = scene.spawnable();
                        let (density, physics_mass, point_mass) = if spawnable.is_massive() {
                            if body_info.with_mass {
                                (
                                    spawnable.density(),
                                    body_info.mass,
                                    PointMass::HasGravity {
                                        mass: body_info.mass,
                                    },
                                )
                            } else {
                                (0.001, 0.01, PointMass::AffectedByGravity)
                            }
                        } else {
                            (spawnable.density(), 0.01, PointMass::AffectedByGravity)
                        };
                        let mut entity = commands.entity(scene.entity());
                        println!("{}", density);

                        entity.with_children(|child| {
                            let mut entity = child.spawn_bundle(BodyBundle::new(
                                place_pos,
                                Velocity::from_linear(place_pos - mouse_pos),
                                density,
                                physics_mass,
                                point_mass,
                                Color::WHITE,
                                &asset_server,
                            ));

                            if body_info.with_trail {
                                entity.insert(Trail::new(20.0, 1));
                            }
                        });
                    }
                }
            }
        }
    }

    if let Some(place_pos) = body_info.position {
        let scale = (mouse_pos.distance_squared(place_pos).powf(0.04) - 1.0).clamp(0.0, 1.0);
        lines.line_colored(
            place_pos,
            mouse_pos,
            0.0,
            Color::rgb(scale, 1.0 - scale, 0.0),
        )
    }
}

#[derive(Bundle)]
struct BodyBundle {
    #[bundle]
    shape_bundle: SpriteBundle,
    collider: CollisionShape,
    material: PhysicMaterial,
    rigidbody: RigidBody,
    velocity: Velocity,
    acceleration: Acceleration,
    point_mass: PointMass,
}

impl BodyBundle {
    fn new(
        position: Vec3,
        velocity: Velocity,
        density: f32,
        mass: f32,
        point_mass: PointMass,
        color: Color,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        let radius = (mass / (density * PI)).sqrt();
        Self {
            shape_bundle: SpriteBundle {
                transform: Transform::from_translation(position),
                texture: asset_server.load("sprites/circle-sprite-300.png"),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(radius * 2.0)),
                    ..default()
                },
                ..default()
            },
            collider: CollisionShape::Sphere { radius },
            material: PhysicMaterial {
                restitution: 0.0,
                density,
                friction: 0.5,
            },
            rigidbody: RigidBody::Dynamic,
            velocity,
            acceleration: Acceleration::default(),
            point_mass,
        }
    }
}
