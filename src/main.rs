mod input;
mod nbody;
mod trails;

use std::f32::consts::{PI, TAU};
use std::time::Duration;

use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::system::EntityCommands;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::scene::SceneInstance;
use bevy::time::FixedTimestep;
use bevy::{prelude::*, window::PresentMode};
use bevy_egui::egui::{ComboBox, Layout, Slider};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_mouse_tracking_plugin::{MousePosPlugin, MousePosWorld};
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Circle;
use heron::{prelude::*, PhysicsSteps};
use rand::{thread_rng, Rng};

use nbody::{ParticularPlugin, PointMass};
use trails::{DrawTrail, TrailsPlugin};

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
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PhysicsSteps::from_steps_per_seconds(60.0))
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin {
            wait_duration: Duration::from_secs_f32(1.0),
            ..default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(MousePosPlugin::SingleCamera)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(TrailsPlugin)
        .add_plugin(ParticularPlugin)
        .init_resource::<SimulationScene>()
        .init_resource::<BodyInfo>()
        .add_state(SimulationState::Running)
        .add_startup_system(spawn_camera)
        .add_startup_system(setup_ui_fps)
        .add_system(update_ui_fps.with_run_criteria(FixedTimestep::step(0.2)))
        .add_system(place_body)
        .add_system(body_info_window)
        .add_system(sim_info_window)
        .add_system(scene_cleanup_and_reinstance.with_run_criteria(scene_changed))
        .add_system_set(SystemSet::on_enter(SimulationState::Paused).with_system(pause_physics))
        .add_system_set(SystemSet::on_exit(SimulationState::Paused).with_system(resume_physics))
        .add_system(pause_resume)
        .run();
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
            text.sections[1].value = format!("{fps:.0}");
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Right, MouseButton::Middle],
            ..default()
        });
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
    trail: bool,
}

impl Default for BodyInfo {
    fn default() -> Self {
        Self {
            position: None,
            mass: 100.0,
            trail: false,
        }
    }
}

fn body_info_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut body_info: ResMut<BodyInfo>,
    scene: Res<SimulationScene>,
) {
    egui::Window::new("Body info").show(egui_ctx.ctx_mut(), |ui| {
        let max_mass = scene.max_spawnable_mass();
        ui.add(
            Slider::new(&mut body_info.mass, 1.0..=max_mass)
                .text("Mass")
                .logarithmic(true),
        );

        ui.checkbox(&mut body_info.trail, "Draw trail");
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
    scene_query: Query<Entity, With<SceneInstance>>,
) {
    let mouse_pos = mouse_pos.truncate().extend(0.0);

    for event in click_event.iter() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => body_info.position = Some(mouse_pos),
                ButtonState::Released => {
                    if let Some(place_pos) = body_info.position.take() {
                        let mut scene = commands
                            .entity(scene_query.get_single().expect("There should be one scene"));
                        scene.with_children(|child| {
                            let mut entity = child.spawn_bundle(BodyBundle::new(
                                place_pos,
                                Velocity::from_linear(place_pos - mouse_pos),
                                0.1,
                                body_info.mass,
                                PointMass::HasGravity {
                                    mass: body_info.mass,
                                },
                                Color::WHITE,
                            ));

                            if body_info.trail {
                                entity.insert(DrawTrail::new(20.0, 1));
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum SimulationState {
    Running,
    Paused,
}

#[derive(PartialEq, Debug, Clone)]
enum SimulationScene {
    Empty,
    Orbits(OrbitsInfo),
}

impl Default for SimulationScene {
    fn default() -> Self {
        Self::Orbits(OrbitsInfo::default())
    }
}

impl std::fmt::Display for SimulationScene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationScene::Empty => write!(f, "Empty"),
            SimulationScene::Orbits(_) => write!(f, "Orbits"),
        }
    }
}

#[allow(clippy::single_match)]
impl SimulationScene {
    fn show_ui(&mut self, ui: &mut egui::Ui) {
        let min_pos = self.min_spawnable_position();
        let max_mass = self.max_spawnable_mass();

        match self {
            SimulationScene::Orbits(info) => {
                ui.separator();

                ui.label("Central body:");
                {
                    ui.add(
                        Slider::new(&mut info.main_mass, 1E3..=1E6)
                            .logarithmic(true)
                            .text("Mass"),
                    );
                }

                ui.separator();

                ui.label("Orbiting bodies:");
                {
                    ui.add(
                        Slider::new(&mut info.bodies_count, 1..=5000)
                            .text("Body count")
                            .logarithmic(true),
                    );

                    ui.add(
                        Slider::new(&mut info.bodies_range_pos, min_pos..=10000.0)
                            .text("Position range")
                            .logarithmic(true)
                            .integer(),
                    );

                    ui.add(
                        Slider::new(&mut info.bodies_range_mass, 0.0..=max_mass).text("Mass range"),
                    );
                }
            }
            _ => {}
        }
    }

    fn instance(&self, mut scene_commands: EntityCommands) {
        match self {
            Self::Orbits(info) => {
                let mut rng = thread_rng();

                scene_commands.with_children(|child| {
                    child.spawn_bundle(BodyBundle::new(
                        Vec3::ZERO,
                        Velocity::from_linear(Vec3::ZERO),
                        info.main_density,
                        info.main_mass,
                        PointMass::HasGravity {
                            mass: info.main_mass,
                        },
                        Color::WHITE,
                    ));

                    let min_radius = 2.0 * self.main_radius();
                    let min_p_sqrt =
                        min_radius * min_radius / (info.bodies_range_pos * info.bodies_range_pos);

                    for i in 0..info.bodies_count {
                        let radius = info.bodies_range_pos * rng.gen_range(min_p_sqrt..=1.0).sqrt();
                        let theta = rng.gen_range(0.0..=TAU);

                        let position = Vec3::new(radius * theta.cos(), radius * theta.sin(), 0.0);

                        let mass = rng.gen_range(0.0..=info.bodies_range_mass);

                        let direction = position - Vec3::ZERO;
                        let distance = direction.length_squared();

                        let vel = (G * (info.main_mass + mass)).sqrt() * distance.powf(-0.75);
                        let velvec = Vec3::new(-direction.y * vel, direction.x * vel, 0.0);

                        let mut random_color = || rng.gen_range(0.0..=1.0_f32);
                        let (r, g, b) = (random_color(), random_color(), random_color());

                        child
                            .spawn_bundle(BodyBundle::new(
                                position,
                                Velocity::from_linear(velvec),
                                info.bodies_density,
                                mass.max(1.0),
                                PointMass::HasGravity { mass },
                                Color::rgb(r, g, b),
                            ))
                            .insert(Name::new(format!("Particle {}", i)));
                    }
                });
            }
            _ => {}
        }
    }

    fn main_radius(&self) -> f32 {
        match self {
            SimulationScene::Orbits(info) => (info.main_mass / (info.main_density * PI)).sqrt(),
            _ => 0.0,
        }
    }

    fn max_spawnable_mass(&self) -> f32 {
        match self {
            SimulationScene::Orbits(info) => info.main_mass / 5E3,
            _ => 100.0,
        }
    }

    fn min_spawnable_position(&self) -> f32 {
        match self {
            SimulationScene::Orbits(info) => ((info.bodies_count as f32).sqrt()
                * info.bodies_range_mass)
                .max(self.main_radius() * 4.0),
            _ => 0.0,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
struct OrbitsInfo {
    main_mass: f32,
    main_density: f32,
    bodies_count: usize,
    bodies_density: f32,
    bodies_range_pos: f32,
    bodies_range_mass: f32,
}

impl Default for OrbitsInfo {
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

fn sim_info_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut scene: ResMut<SimulationScene>,
    mut current: Local<Option<SimulationScene>>,
    mut cached: Local<OrbitsInfo>,
) {
    if let Some(current) = current.as_mut() {
        egui::Window::new("Simulation").show(egui_ctx.ctx_mut(), |ui| {
            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                ComboBox::from_label("")
                    .selected_text(current.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(current, SimulationScene::Orbits(*cached), "Orbits");
                        ui.selectable_value(current, SimulationScene::Empty, "Empty");
                    });

                if ui.button("New").clicked() {
                    *scene = current.clone();
                }
            });

            current.show_ui(ui);

            if let SimulationScene::Orbits(info) = *current {
                *cached = info;
            }
        });
    } else {
        *current = Some(scene.clone())
    }
}

fn scene_changed(scene: Res<SimulationScene>) -> ShouldRun {
    if scene.is_changed() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn scene_cleanup_and_reinstance(
    mut commands: Commands,
    scene: Res<SimulationScene>,
    scene_query: Query<Entity, With<SceneInstance>>,
) {
    let mut scene_commands = match scene_query.get_single() {
        Ok(scene) => commands.entity(scene),
        Err(_) => commands.spawn_bundle(SceneBundle { ..default() }),
    };

    scene_commands.despawn_descendants();
    scene.instance(scene_commands);
}

#[derive(Bundle)]
struct BodyBundle {
    #[bundle]
    shape_bundle: ShapeBundle,
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
    ) -> Self {
        let radius = (mass / (density * PI)).sqrt();
        Self {
            shape_bundle: GeometryBuilder::build_as(
                &Circle {
                    radius,
                    center: Vec2::ZERO,
                },
                DrawMode::Fill(FillMode::color(color)),
                Transform::from_translation(position),
            ),
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
