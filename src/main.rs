use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowResolution};
use rand::{rngs::StdRng, Rng, SeedableRng};

const STAR_COUNT: usize = 1400;
const STAR_FIELD_RADIUS: f32 = 9000.0;
const SHIP_ACCELERATION: f32 = 380.0;
const SHIP_DRAG: f32 = 0.985;
const SHIP_MAX_SPEED: f32 = 520.0;
const CAMERA_LERP: f32 = 6.5;
const SHIP_SCALE: f32 = 0.18;
const TURN_SPEED: f32 = 4.8;
const WARP_ACCELERATION: f32 = 2400.0;
const WARP_BRAKE_ACCELERATION: f32 = 1800.0;
const WARP_MAX_SPEED: f32 = 7600.0;
const WARP_ENTRY_SPEED: f32 = 1500.0;
const WARP_DEPART_DISTANCE: f32 = 7200.0;
const WARP_ARRIVAL_DISTANCE: f32 = 24000.0;
const WARP_ARRIVAL_RADIUS: f32 = 520.0;
const WARP_FINISH_RADIUS: f32 = 22.0;
const WARP_GUIDANCE: f32 = 1.85;
const WARP_ALIGNMENT_EPSILON: f32 = 0.03;
const ENGINE_OFFSET_X: f32 = 48.5;
const ENGINE_OFFSET_Y: f32 = -136.5;
const FLAME_BASE_LENGTH: f32 = 16.0;
const FLAME_MAX_LENGTH: f32 = 34.0;
const THRUST_RAMP_UP: f32 = 1.8;
const THRUST_RAMP_DOWN: f32 = 2.8;
const MAP_NODE_RADIUS: f32 = 18.0;
const MAP_LEFT_SHIFT: f32 = -150.0;
const MAP_INFO_SHIFT: f32 = 345.0;
const MAP_PREVIEW_SCALE: f32 = 1.7;
const MAP_PREVIEW_Z: f32 = 151.5;
const PLANET_RENDER_Z: f32 = -5.0;
const STATION_RENDER_Z: f32 = -4.0;

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct ThrusterState {
    thrusting: bool,
    intensity: f32,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component, Clone, Copy)]
struct EngineFlame {
    side: f32,
}

#[derive(Component)]
struct StarfieldStar {
    size: f32,
    brightness: f32,
}

#[derive(Component)]
struct CurrentSystemPlanet;

#[derive(Component)]
struct CurrentSystemStation {
    slot: usize,
}

#[derive(Component)]
struct MapElement;

#[derive(Component)]
struct MapBackdrop;

#[derive(Component)]
struct MapInfoBackdrop;

#[derive(Component)]
struct MapSelection;

#[derive(Component)]
struct MapNode {
    index: usize,
}

#[derive(Component)]
struct MapNodeLabel {
    index: usize,
}

#[derive(Component)]
struct MapTitleText;

#[derive(Component)]
struct MapHintText;

#[derive(Component)]
struct MapInfoTitleText;

#[derive(Component)]
struct MapInfoFactionText;

#[derive(Component)]
struct MapInfoStationsText;

#[derive(Component)]
struct MapPlanetPreview;

#[derive(Component)]
struct HudSystemText;

#[derive(Component)]
struct HudHintText;

#[derive(Resource)]
struct GalaxyMap {
    systems: Vec<SystemDefinition>,
    current_system: usize,
    selected_system: usize,
    map_open: bool,
}

#[derive(Resource, Default)]
struct WarpDrive {
    active: bool,
    phase: WarpPhase,
    target_system: usize,
    departure_origin: Vec2,
    travel_direction: Vec2,
    arrival_point: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum WarpPhase {
    #[default]
    Idle,
    Align,
    Depart,
    Cruise,
    Arrive,
}

#[derive(Clone)]
struct SystemDefinition {
    name: &'static str,
    faction: Faction,
    map_position: Vec2,
    star_color: Color,
    sky_color: Color,
    planet_sprite: &'static str,
    planet_position: Vec2,
    planet_scale: f32,
    stations: Vec<StationDefinition>,
}

#[derive(Clone)]
struct StationDefinition {
    name: &'static str,
    owner: Faction,
    sprite: &'static str,
    offset: Vec2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Faction {
    Helios,
    Vanta,
}

impl Faction {
    fn name(self) -> &'static str {
        match self {
            Faction::Helios => "Helios Pact",
            Faction::Vanta => "Vanta Clade",
        }
    }

    fn color(self) -> Color {
        match self {
            Faction::Helios => Color::srgb(0.95, 0.72, 0.32),
            Faction::Vanta => Color::srgb(0.42, 0.70, 0.95),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.01, 0.015, 0.03)))
        .insert_resource(authored_galaxy())
        .insert_resource(WarpDrive::default())
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "DonutSpace".into(),
                        resolution: WindowResolution::new(1600, 900),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                map_input_system,
                warp_travel_system,
                player_input_system,
                engine_flame_system,
                camera_follow_system,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_map_panels_system,
                update_map_nodes_system,
                update_map_title_system,
                update_map_info_title_system,
                update_map_info_faction_system,
                update_map_info_stations_system,
                update_map_planet_preview_system,
                update_local_system_visuals_system,
                update_hud_system,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, galaxy: Res<GalaxyMap>) {
    commands.spawn((Camera2d, MainCamera));

    spawn_starfield(&mut commands, &galaxy);

    commands.spawn((
        Sprite::from_image(asset_server.load("ship1.png")),
        Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(SHIP_SCALE)),
        Player,
        Velocity::default(),
        ThrusterState::default(),
    ));

    for side in [-1.0, 1.0] {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(0.45, 0.82, 1.0, 0.75),
                Vec2::new(8.0, FLAME_BASE_LENGTH),
            ),
            bevy::sprite::Anchor::TOP_CENTER,
            Transform::from_xyz(
                side * ENGINE_OFFSET_X * SHIP_SCALE,
                ENGINE_OFFSET_Y * SHIP_SCALE,
                9.0,
            ),
            Visibility::Hidden,
            EngineFlame { side },
        ));
    }

    let current_system = &galaxy.systems[galaxy.current_system];
    commands.spawn((
        Sprite::from_image(asset_server.load(current_system.planet_sprite)),
        Transform::from_xyz(
            current_system.planet_position.x,
            current_system.planet_position.y,
            PLANET_RENDER_Z,
        )
        .with_scale(Vec3::splat(current_system.planet_scale)),
        CurrentSystemPlanet,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load(
            current_system
                .stations
                .first()
                .map(|station| station.sprite)
                .unwrap_or("stations/SS1.png"),
        )),
        Transform::from_xyz(
            current_system.planet_position.x,
            current_system.planet_position.y,
            STATION_RENDER_Z,
        )
        .with_scale(Vec3::splat(0.22)),
        if current_system.stations.is_empty() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        },
        CurrentSystemStation { slot: 0 },
    ));

    let text_font = TextFont::from_font_size(22.0);
    let small_text_font = TextFont::from_font_size(16.0);

    commands.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextColor(Color::WHITE),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-770.0, 430.0, 200.0),
        HudSystemText,
    ));

    commands.spawn((
        Text2d::new("TAB: star map | LMB: thrust toward cursor | systems and stations are map-only"),
        small_text_font.clone(),
        TextColor(Color::srgba(0.70, 0.76, 0.84, 0.95)),
        bevy::sprite::Anchor::BOTTOM_LEFT,
        Transform::from_xyz(-770.0, -430.0, 200.0),
        HudHintText,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgba(0.03, 0.05, 0.09, 0.92), Vec2::new(1120.0, 620.0)),
        Transform::from_xyz(0.0, 0.0, 150.0),
        Visibility::Hidden,
        MapElement,
        MapBackdrop,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgba(0.06, 0.08, 0.12, 0.96), Vec2::new(320.0, 530.0)),
        Transform::from_xyz(MAP_INFO_SHIFT, 0.0, 151.0),
        Visibility::Hidden,
        MapElement,
        MapInfoBackdrop,
    ));

    commands.spawn((
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.12), Vec2::splat(34.0)),
        Transform::from_xyz(MAP_LEFT_SHIFT, 0.0, 153.0),
        Visibility::Hidden,
        MapElement,
        MapSelection,
    ));

    for (index, system) in galaxy.systems.iter().enumerate() {
        commands.spawn((
            Sprite::from_color(system.faction.color(), Vec2::splat(20.0)),
            Transform::from_xyz(MAP_LEFT_SHIFT + system.map_position.x, system.map_position.y, 152.0),
            Visibility::Hidden,
            MapElement,
            MapNode { index },
        ));

        commands.spawn((
            Text2d::new(system.name),
            small_text_font.clone(),
            TextColor(Color::WHITE),
            bevy::sprite::Anchor::TOP_CENTER,
            Transform::from_xyz(
                MAP_LEFT_SHIFT + system.map_position.x,
                system.map_position.y - 22.0,
                154.0,
            ),
            Visibility::Hidden,
            MapElement,
            MapNodeLabel { index },
        ));
    }

    commands.spawn((
        Text2d::new("STAR MAP"),
        TextFont::from_font_size(30.0),
        TextColor(Color::WHITE),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-510.0, 260.0, 154.0),
        Visibility::Hidden,
        MapElement,
        MapTitleText,
    ));

    commands.spawn((
        Text2d::new("Click system to select. Click again or press Enter to warp."),
        small_text_font.clone(),
        TextColor(Color::srgba(0.72, 0.80, 0.90, 0.95)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(-510.0, 230.0, 154.0),
        Visibility::Hidden,
        MapElement,
        MapHintText,
    ));

    commands.spawn((
        Text2d::new(""),
        TextFont::from_font_size(28.0),
        TextColor(Color::WHITE),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(220.0, 220.0, 154.0),
        Visibility::Hidden,
        MapElement,
        MapInfoTitleText,
    ));

    commands.spawn((
        Text2d::new(""),
        TextFont::from_font_size(20.0),
        TextColor(Color::WHITE),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(220.0, 180.0, 154.0),
        Visibility::Hidden,
        MapElement,
        MapInfoFactionText,
    ));

    commands.spawn((
        Text2d::new(""),
        small_text_font,
        TextColor(Color::srgba(0.82, 0.87, 0.94, 0.96)),
        bevy::sprite::Anchor::TOP_LEFT,
        Transform::from_xyz(220.0, 35.0, 154.0),
        Visibility::Hidden,
        MapElement,
        MapInfoStationsText,
    ));

    let mut preview_sprite =
        Sprite::from_image(asset_server.load(galaxy.systems[galaxy.selected_system].planet_sprite));
    preview_sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.36);

    commands.spawn((
        preview_sprite,
        Transform::from_xyz(410.0, 70.0, MAP_PREVIEW_Z).with_scale(Vec3::splat(MAP_PREVIEW_SCALE)),
        Visibility::Hidden,
        MapElement,
        MapPlanetPreview,
    ));
}

fn spawn_starfield(commands: &mut Commands, galaxy: &GalaxyMap) {
    let mut rng = StdRng::seed_from_u64(42);
    let current_system = &galaxy.systems[galaxy.current_system];

    for _ in 0..STAR_COUNT {
        let x = rng.gen_range(-STAR_FIELD_RADIUS..STAR_FIELD_RADIUS);
        let y = rng.gen_range(-STAR_FIELD_RADIUS..STAR_FIELD_RADIUS);
        let size = rng.gen_range(1.0..3.8);
        let brightness = rng.gen_range(0.45..1.0);
        let color = tint_star(current_system.star_color, brightness);

        commands.spawn((
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_xyz(x, y, -20.0),
            StarfieldStar { size, brightness },
        ));
    }
}

fn map_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<MainCamera>, Without<Player>)>,
    asset_server: Res<AssetServer>,
    mut galaxy: ResMut<GalaxyMap>,
    mut warp: ResMut<WarpDrive>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
    mut preview_query: Query<&mut Sprite, (With<MapPlanetPreview>, Without<StarfieldStar>)>,
) {
    if warp.active {
        galaxy.map_open = false;
        return;
    }

    if keyboard.just_pressed(KeyCode::Tab) {
        galaxy.map_open = !galaxy.map_open;
    }

    if !galaxy.map_open {
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        galaxy.map_open = false;
        return;
    }

    let Ok(window) = primary_window.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor) = window.cursor_position() {
            if let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor) {
                let map_center = camera_transform.translation().truncate();
                for (index, system) in galaxy.systems.iter().enumerate() {
                    let node_position = map_center + Vec2::new(MAP_LEFT_SHIFT, 0.0) + system.map_position;
                    if cursor_world.distance(node_position) <= MAP_NODE_RADIUS {
                        if galaxy.selected_system == index && galaxy.current_system != index {
                            begin_warp(
                                index,
                                &mut galaxy,
                                &mut warp,
                                &mut player_query,
                            );
                        } else {
                            galaxy.selected_system = index;
                            if let Ok(mut sprite) = preview_query.single_mut() {
                                sprite.image = asset_server.load(galaxy.systems[index].planet_sprite);
                            }
                        }
                        return;
                    }
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) && galaxy.current_system != galaxy.selected_system {
        begin_warp(
            galaxy.selected_system,
            &mut galaxy,
            &mut warp,
            &mut player_query,
        );
    }
}

fn begin_warp(
    target_index: usize,
    galaxy: &mut GalaxyMap,
    warp: &mut WarpDrive,
    player_query: &mut Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
) {
    if target_index == galaxy.current_system || warp.active {
        return;
    }

    galaxy.selected_system = target_index;
    galaxy.map_open = false;
    if let Ok((transform, mut velocity, mut thruster_state)) = player_query.single_mut() {
        let current_map_pos = galaxy.systems[galaxy.current_system].map_position;
        let target_map_pos = galaxy.systems[target_index].map_position;
        let travel_direction = (target_map_pos - current_map_pos).normalize_or_zero();

        warp.active = true;
        warp.phase = WarpPhase::Align;
        warp.target_system = target_index;
        warp.departure_origin = transform.translation.truncate();
        warp.travel_direction = if travel_direction.length_squared() > 0.0 {
            travel_direction
        } else {
            Vec2::X
        };
        warp.arrival_point = warp_arrival_point(&galaxy.systems[target_index], target_index);

        velocity.0 *= 0.2;
        thruster_state.thrusting = false;
        thruster_state.intensity = 0.0;
    }
}

fn warp_travel_system(
    time: Res<Time>,
    mut clear_color: ResMut<ClearColor>,
    mut galaxy: ResMut<GalaxyMap>,
    mut warp: ResMut<WarpDrive>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
    mut star_query: Query<(&StarfieldStar, &mut Sprite), Without<MapPlanetPreview>>,
) {
    if !warp.active {
        return;
    }

    let Ok((mut transform, mut velocity, mut thruster_state)) = player_query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let ship_position = transform.translation.truncate();
    let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    let target_direction = match warp.phase {
        WarpPhase::Align | WarpPhase::Depart | WarpPhase::Cruise => warp.travel_direction,
        WarpPhase::Arrive => (warp.arrival_point - ship_position).normalize_or_zero(),
        WarpPhase::Idle => Vec2::ZERO,
    };

    if target_direction.length_squared() > 0.0 {
        let desired_angle = target_direction.y.atan2(target_direction.x);
        let angle_delta = shortest_angle_delta(current_angle, desired_angle);
        let max_step = (TURN_SPEED * 1.85) * dt;
        let new_angle = current_angle + angle_delta.clamp(-max_step, max_step);
        transform.rotation = Quat::from_rotation_z(new_angle - std::f32::consts::FRAC_PI_2);
    }

    match warp.phase {
        WarpPhase::Align => {
            thruster_state.thrusting = false;
            thruster_state.intensity =
                (thruster_state.intensity - THRUST_RAMP_DOWN * dt).max(0.0);
            velocity.0 *= 0.97;

            let desired_angle = warp.travel_direction.y.atan2(warp.travel_direction.x);
            let angle_delta = shortest_angle_delta(current_angle, desired_angle).abs();
            if angle_delta <= WARP_ALIGNMENT_EPSILON {
                warp.phase = WarpPhase::Depart;
            }
        }
        WarpPhase::Depart => {
            thruster_state.thrusting = true;
            thruster_state.intensity =
                (thruster_state.intensity + THRUST_RAMP_UP * dt).clamp(0.0, 1.0);
            velocity.0 += warp.travel_direction * WARP_ACCELERATION * dt;
            velocity.0 = velocity.0.clamp_length_max(WARP_MAX_SPEED);

            let next_position = ship_position + velocity.0 * dt;
            transform.translation.x = next_position.x;
            transform.translation.y = next_position.y;

            let departed_distance = next_position.distance(warp.departure_origin);
            if departed_distance >= WARP_DEPART_DISTANCE && velocity.0.length() >= WARP_ENTRY_SPEED {
                galaxy.current_system = warp.target_system;
                apply_system_palette(
                    &galaxy.systems[warp.target_system],
                    &mut clear_color,
                    &mut star_query,
                );

                let entry_position = warp.arrival_point - warp.travel_direction * WARP_ARRIVAL_DISTANCE;
                transform.translation.x = entry_position.x;
                transform.translation.y = entry_position.y;
                velocity.0 = warp.travel_direction * WARP_MAX_SPEED;
                warp.phase = WarpPhase::Cruise;
            }
        }
        WarpPhase::Cruise => {
            thruster_state.thrusting = true;
            thruster_state.intensity =
                (thruster_state.intensity + THRUST_RAMP_UP * 0.55 * dt).clamp(0.0, 1.0);
            velocity.0 = velocity.0.lerp(warp.travel_direction * WARP_MAX_SPEED, 1.6 * dt);

            let next_position = ship_position + velocity.0 * dt;
            transform.translation.x = next_position.x;
            transform.translation.y = next_position.y;

            let remaining = warp.arrival_point.distance(next_position);
            let brake_distance =
                (velocity.0.length().powi(2) / (2.0 * WARP_BRAKE_ACCELERATION)).max(WARP_ARRIVAL_RADIUS);
            if remaining <= brake_distance {
                warp.phase = WarpPhase::Arrive;
            }
        }
        WarpPhase::Arrive => {
            let to_target = warp.arrival_point - ship_position;
            let distance = to_target.length();
            let direction = to_target.normalize_or_zero();
            let braking_speed = (2.0 * WARP_BRAKE_ACCELERATION * distance).sqrt();
            let approach_factor = (distance / 1800.0).clamp(0.0, 1.0);
            let desired_speed = (braking_speed.min(WARP_MAX_SPEED) * approach_factor)
                .max(if distance > 180.0 { 70.0 } else { 0.0 });
            let desired_velocity = if direction.length_squared() > 0.0 {
                direction * desired_speed
            } else {
                Vec2::ZERO
            };

            velocity.0 = velocity.0.lerp(desired_velocity, (WARP_GUIDANCE * dt).clamp(0.0, 1.0));
            thruster_state.thrusting = velocity.0.length() > 40.0 || distance > WARP_FINISH_RADIUS;
            let desired_intensity = (distance / 2500.0).clamp(0.12, 0.72);
            thruster_state.intensity +=
                (desired_intensity - thruster_state.intensity) * THRUST_RAMP_UP * 0.7 * dt;
            thruster_state.intensity = thruster_state.intensity.clamp(0.0, 1.0);

            let next_position = ship_position + velocity.0 * dt;
            transform.translation.x = next_position.x;
            transform.translation.y = next_position.y;

            if distance <= WARP_FINISH_RADIUS && velocity.0.length() <= 28.0 {
                transform.translation.x = warp.arrival_point.x;
                transform.translation.y = warp.arrival_point.y;
                velocity.0 = Vec2::ZERO;
                thruster_state.thrusting = false;
                thruster_state.intensity = 0.0;
                warp.active = false;
                warp.phase = WarpPhase::Idle;
            }
        }
        WarpPhase::Idle => {}
    }
}

fn player_input_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    time: Res<Time>,
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut thruster_state)) = query.single_mut() else {
        return;
    };

    if warp.active {
        return;
    }

    if galaxy.map_open {
        thruster_state.thrusting = false;
        thruster_state.intensity = (thruster_state.intensity - THRUST_RAMP_DOWN * time.delta_secs()).max(0.0);
        velocity.0 *= SHIP_DRAG;
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
        return;
    }

    let Ok(window) = primary_window.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let ship_position = transform.translation.truncate();
    let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;

    if let Some(cursor) = window.cursor_position() {
        if let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor) {
            let to_cursor = cursor_world - ship_position;
            if to_cursor.length_squared() > 1.0 {
                let desired_angle = to_cursor.y.atan2(to_cursor.x);
                let angle_delta = shortest_angle_delta(current_angle, desired_angle);
                let max_step = TURN_SPEED * time.delta_secs();
                let new_angle = current_angle + angle_delta.clamp(-max_step, max_step);
                transform.rotation = Quat::from_rotation_z(new_angle - std::f32::consts::FRAC_PI_2);
            }
        }
    }

    let wants_thrust = mouse_buttons.pressed(MouseButton::Left);
    let ramp_speed = if wants_thrust { THRUST_RAMP_UP } else { THRUST_RAMP_DOWN };
    let target_intensity = if wants_thrust { 1.0 } else { 0.0 };
    thruster_state.intensity +=
        (target_intensity - thruster_state.intensity) * ramp_speed * time.delta_secs();
    thruster_state.intensity = thruster_state.intensity.clamp(0.0, 1.0);
    thruster_state.thrusting = thruster_state.intensity > 0.03;

    if thruster_state.thrusting {
        let facing = Vec2::from_angle(
            transform.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2,
        );
        velocity.0 += facing * SHIP_ACCELERATION * thruster_state.intensity * time.delta_secs();
    }

    velocity.0 *= SHIP_DRAG;
    if velocity.0.length() > SHIP_MAX_SPEED {
        velocity.0 = velocity.0.normalize() * SHIP_MAX_SPEED;
    }

    transform.translation.x += velocity.0.x * time.delta_secs();
    transform.translation.y += velocity.0.y * time.delta_secs();
}

fn engine_flame_system(
    player_query: Query<(&Transform, &ThrusterState), With<Player>>,
    mut flame_query: Query<(&EngineFlame, &mut Transform, &mut Sprite, &mut Visibility), Without<Player>>,
) {
    let Ok((player_transform, thruster_state)) = player_query.single() else {
        return;
    };

    let rotation = player_transform.rotation.to_euler(EulerRot::XYZ).2;
    let rotation_matrix = Mat2::from_angle(rotation);
    let flame_length = FLAME_BASE_LENGTH
        + (FLAME_MAX_LENGTH - FLAME_BASE_LENGTH) * thruster_state.intensity;

    for (flame, mut transform, mut sprite, mut visibility) in &mut flame_query {
        if thruster_state.thrusting {
            *visibility = Visibility::Inherited;

            let local_offset = Vec2::new(
                flame.side * ENGINE_OFFSET_X * SHIP_SCALE,
                ENGINE_OFFSET_Y * SHIP_SCALE,
            );
            let world_offset = rotation_matrix * local_offset;

            transform.translation.x = player_transform.translation.x + world_offset.x;
            transform.translation.y = player_transform.translation.y + world_offset.y;
            transform.translation.z = player_transform.translation.z - 1.0;
            transform.rotation = player_transform.rotation;

            sprite.custom_size = Some(Vec2::new(8.0, flame_length));
            sprite.color = Color::srgba(
                0.45 + 0.25 * thruster_state.intensity,
                0.82 + 0.10 * thruster_state.intensity,
                1.0,
                0.72,
            );
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn camera_follow_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let target = player_transform.translation;
    let current = camera_transform.translation;
    let next = current.lerp(
        Vec3::new(target.x, target.y, current.z),
        CAMERA_LERP * time.delta_secs(),
    );
    camera_transform.translation = next;
}

fn update_map_panels_system(
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut backdrop_query: Query<(&mut Transform, &mut Visibility), (With<MapBackdrop>, Without<MapInfoBackdrop>)>,
    mut info_backdrop_query: Query<(&mut Transform, &mut Visibility), (With<MapInfoBackdrop>, Without<MapBackdrop>)>,
    mut selection_query: Query<(&mut Transform, &mut Visibility), (With<MapSelection>, Without<MapBackdrop>, Without<MapInfoBackdrop>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    let selected = &galaxy.systems[galaxy.selected_system];

    if let Ok((mut transform, mut map_visibility)) = backdrop_query.single_mut() {
        transform.translation = camera_center.extend(150.0);
        *map_visibility = visible;
    }

    if let Ok((mut transform, mut map_visibility)) = info_backdrop_query.single_mut() {
        transform.translation = (camera_center + Vec2::new(MAP_INFO_SHIFT, 0.0)).extend(151.0);
        *map_visibility = visible;
    }

    if let Ok((mut transform, mut map_visibility)) = selection_query.single_mut() {
        let pos = camera_center + Vec2::new(MAP_LEFT_SHIFT, 0.0) + selected.map_position;
        transform.translation = pos.extend(153.0);
        *map_visibility = visible;
    }
}

fn update_map_nodes_system(
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut node_query: Query<(&MapNode, &mut Transform, &mut Sprite, &mut Visibility), Without<MapNodeLabel>>,
    mut label_query: Query<(&MapNodeLabel, &mut Transform, &mut Visibility), Without<MapNode>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };

    for (node, mut transform, mut sprite, mut map_visibility) in &mut node_query {
        let system = &galaxy.systems[node.index];
        let pos = camera_center + Vec2::new(MAP_LEFT_SHIFT, 0.0) + system.map_position;
        transform.translation = pos.extend(152.0);
        sprite.color = if node.index == galaxy.current_system {
            Color::WHITE
        } else {
            system.faction.color()
        };
        sprite.custom_size = Some(if node.index == galaxy.current_system {
            Vec2::splat(24.0)
        } else {
            Vec2::splat(20.0)
        });
        *map_visibility = visible;
    }

    for (label, mut transform, mut map_visibility) in &mut label_query {
        let system = &galaxy.systems[label.index];
        let pos = camera_center + Vec2::new(MAP_LEFT_SHIFT, 0.0) + system.map_position + Vec2::new(0.0, -22.0);
        transform.translation = pos.extend(154.0);
        *map_visibility = visible;
    }
}

fn update_map_title_system(
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut title_query: Query<(&mut Transform, &mut Visibility), (With<MapTitleText>, Without<MapHintText>)>,
    mut hint_query: Query<(&mut Transform, &mut Visibility), (With<MapHintText>, Without<MapTitleText>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    if let Ok((mut transform, mut map_visibility)) = title_query.single_mut() {
        transform.translation = (camera_center + Vec2::new(-510.0, 260.0)).extend(154.0);
        *map_visibility = visible;
    }

    if let Ok((mut transform, mut map_visibility)) = hint_query.single_mut() {
        transform.translation = (camera_center + Vec2::new(-510.0, 230.0)).extend(154.0);
        *map_visibility = visible;
    }
}

fn update_map_info_title_system(
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut info_title_query: Query<(&mut Text2d, &mut Transform, &mut Visibility), With<MapInfoTitleText>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    let selected = &galaxy.systems[galaxy.selected_system];

    if let Ok((mut text, mut transform, mut map_visibility)) = info_title_query.single_mut() {
        *text = Text2d::new(selected.name);
        transform.translation = (camera_center + Vec2::new(220.0, 220.0)).extend(154.0);
        *map_visibility = visible;
    }
}

fn update_map_info_faction_system(
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut info_faction_query: Query<(&mut Text2d, &mut TextColor, &mut Transform, &mut Visibility), With<MapInfoFactionText>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    let selected = &galaxy.systems[galaxy.selected_system];

    if let Ok((mut text, mut color, mut transform, mut map_visibility)) = info_faction_query.single_mut() {
        *text = Text2d::new(format!("Faction: {}", selected.faction.name()));
        *color = TextColor(selected.faction.color());
        transform.translation = (camera_center + Vec2::new(220.0, 180.0)).extend(154.0);
        *map_visibility = visible;
    }
}

fn update_map_info_stations_system(
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut info_stations_query: Query<(&mut Text2d, &mut Transform, &mut Visibility), With<MapInfoStationsText>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    let selected = &galaxy.systems[galaxy.selected_system];

    if let Ok((mut text, mut transform, mut map_visibility)) = info_stations_query.single_mut() {
        let mut lines = vec!["Stations:".to_string()];
        for station in &selected.stations {
            lines.push(format!("- {} [{}]", station.name, station.owner.name()));
        }
        *text = Text2d::new(lines.join("\n"));
        transform.translation = (camera_center + Vec2::new(220.0, 35.0)).extend(154.0);
        *map_visibility = visible;
    }
}

fn update_map_planet_preview_system(
    asset_server: Res<AssetServer>,
    galaxy: Res<GalaxyMap>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut planet_query: Query<(&mut Sprite, &mut Transform, &mut Visibility), With<MapPlanetPreview>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let visible = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    let selected = &galaxy.systems[galaxy.selected_system];

    if let Ok((mut sprite, mut transform, mut map_visibility)) = planet_query.single_mut() {
        sprite.image = asset_server.load(selected.planet_sprite);
        sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.36);
        transform.translation = (camera_center + Vec2::new(410.0, 70.0)).extend(MAP_PREVIEW_Z);
        transform.scale = Vec3::splat(MAP_PREVIEW_SCALE);
        *map_visibility = visible;
    }
}

fn update_local_system_visuals_system(
    asset_server: Res<AssetServer>,
    galaxy: Res<GalaxyMap>,
    mut planet_query: Query<(&mut Sprite, &mut Transform), With<CurrentSystemPlanet>>,
    mut station_query: Query<(&CurrentSystemStation, &mut Sprite, &mut Transform, &mut Visibility), Without<CurrentSystemPlanet>>,
) {
    let system = &galaxy.systems[galaxy.current_system];

    if let Ok((mut sprite, mut transform)) = planet_query.single_mut() {
        sprite.image = asset_server.load(system.planet_sprite);
        transform.translation.x = system.planet_position.x;
        transform.translation.y = system.planet_position.y;
        transform.translation.z = PLANET_RENDER_Z;
        transform.scale = Vec3::splat(system.planet_scale);
    }

    for (slot, mut sprite, mut transform, mut visibility) in &mut station_query {
        if slot.slot == 0 {
            if let Some(station) = system.stations.first() {
                sprite.image = asset_server.load(station.sprite);
                transform.scale = Vec3::splat(0.22);
                transform.translation.x = system.planet_position.x + station.offset.x;
                transform.translation.y = system.planet_position.y + station.offset.y;
                transform.translation.z = STATION_RENDER_Z;
                transform.rotation = Quat::IDENTITY;
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_hud_system(
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut system_text_query: Query<(&mut Text2d, &mut TextColor, &mut Transform), (With<HudSystemText>, Without<HudHintText>)>,
    mut hint_text_query: Query<(&mut Text2d, &mut Transform), (With<HudHintText>, Without<HudSystemText>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let current_system = &galaxy.systems[galaxy.current_system];

    if let Ok((mut text, mut color, mut transform)) = system_text_query.single_mut() {
        let warp_status = if warp.active {
            let destination = &galaxy.systems[warp.target_system];
            let phase = match warp.phase {
                WarpPhase::Align => "Aligning",
                WarpPhase::Depart => "Departure burn",
                WarpPhase::Cruise => "Warp cruise",
                WarpPhase::Arrive => "Arrival burn",
                WarpPhase::Idle => "Idle",
            };
            format!("\nWarp: {} -> {}", phase, destination.name)
        } else {
            "\nWarp: Offline".to_string()
        };

        *text = Text2d::new(format!(
            "Current system: {}\nHolding: {}\nVisible stations: {}{}",
            current_system.name,
            current_system.faction.name(),
            current_system.stations.len(),
            warp_status,
        ));
        *color = TextColor(current_system.faction.color());
        transform.translation = (camera_center + Vec2::new(-770.0, 430.0)).extend(200.0);
    }

    if let Ok((mut text, mut transform)) = hint_text_query.single_mut() {
        let hint = if warp.active {
            "Warp in progress..."
        } else if galaxy.map_open {
            "TAB/Esc: close map | click system twice or press Enter to warp"
        } else {
            "TAB: star map | LMB: thrust toward cursor | systems and stations are map-only"
        };
        *text = Text2d::new(hint);
        transform.translation = (camera_center + Vec2::new(-770.0, -430.0)).extend(200.0);
    }
}

fn tint_star(star_color: Color, brightness: f32) -> Color {
    let srgba = star_color.to_srgba();
    Color::srgba(
        (srgba.red * brightness).clamp(0.0, 1.0),
        (srgba.green * brightness).clamp(0.0, 1.0),
        (srgba.blue * brightness).clamp(0.0, 1.0),
        0.45 + brightness * 0.5,
    )
}

fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    let mut delta = target - current;
    while delta > std::f32::consts::PI {
        delta -= std::f32::consts::TAU;
    }
    while delta < -std::f32::consts::PI {
        delta += std::f32::consts::TAU;
    }
    delta
}

fn station_anchor(system: &SystemDefinition) -> Vec2 {
    system
        .stations
        .first()
        .map(|station| system.planet_position + station.offset)
        .unwrap_or(system.planet_position + Vec2::new(220.0, 0.0))
}

fn warp_arrival_point(system: &SystemDefinition, system_index: usize) -> Vec2 {
    let station_position = station_anchor(system);
    let angle = 0.85 + system_index as f32 * 0.9;
    let offset = Vec2::from_angle(angle) * WARP_ARRIVAL_RADIUS;
    station_position + offset
}

fn apply_system_palette(
    system: &SystemDefinition,
    clear_color: &mut ClearColor,
    star_query: &mut Query<(&StarfieldStar, &mut Sprite), Without<MapPlanetPreview>>,
) {
    clear_color.0 = system.sky_color;
    for (star, mut sprite) in star_query.iter_mut() {
        sprite.color = tint_star(system.star_color, star.brightness);
        sprite.custom_size = Some(Vec2::splat(star.size));
    }
}

fn authored_galaxy() -> GalaxyMap {
    GalaxyMap {
        systems: vec![
            SystemDefinition {
                name: "Hearthlight",
                faction: Faction::Helios,
                map_position: Vec2::new(-250.0, 120.0),
                star_color: Color::srgb(1.0, 0.82, 0.52),
                sky_color: Color::srgb(0.02, 0.02, 0.05),
                planet_sprite: "planet/spr_planet02.png",
                planet_position: Vec2::new(760.0, 240.0),
                planet_scale: 1.9,
                stations: vec![
                    StationDefinition { name: "Hearth Port", owner: Faction::Helios, sprite: "stations/SS1.png", offset: Vec2::new(150.0, -24.0) },
                ],
            },
            SystemDefinition {
                name: "Lumen Crossing",
                faction: Faction::Helios,
                map_position: Vec2::new(-90.0, -20.0),
                star_color: Color::srgb(1.0, 0.92, 0.68),
                sky_color: Color::srgb(0.02, 0.025, 0.055),
                planet_sprite: "planet/planet18.png",
                planet_position: Vec2::new(840.0, -120.0),
                planet_scale: 2.2,
                stations: vec![
                    StationDefinition { name: "Iris Anchor", owner: Faction::Helios, sprite: "stations/WB_baseu2_d0.png", offset: Vec2::new(170.0, 42.0) },
                ],
            },
            SystemDefinition {
                name: "Gray Expanse",
                faction: Faction::Helios,
                map_position: Vec2::new(-20.0, 205.0),
                star_color: Color::srgb(0.86, 0.93, 1.0),
                sky_color: Color::srgb(0.015, 0.02, 0.045),
                planet_sprite: "planet/spr_planet05.png",
                planet_position: Vec2::new(700.0, 310.0),
                planet_scale: 1.7,
                stations: vec![
                    StationDefinition { name: "Pillar Rest", owner: Faction::Helios, sprite: "stations/WB_base_d0.png", offset: Vec2::new(135.0, 82.0) },
                ],
            },
            SystemDefinition {
                name: "Brimhold",
                faction: Faction::Vanta,
                map_position: Vec2::new(120.0, 140.0),
                star_color: Color::srgb(0.74, 0.86, 1.0),
                sky_color: Color::srgb(0.015, 0.03, 0.055),
                planet_sprite: "planet/planet24.png",
                planet_position: Vec2::new(880.0, 190.0),
                planet_scale: 2.0,
                stations: vec![
                    StationDefinition { name: "Black Quarry", owner: Faction::Vanta, sprite: "stations/starbase-tex.png", offset: Vec2::new(155.0, -58.0) },
                ],
            },
            SystemDefinition {
                name: "Dusk Meridian",
                faction: Faction::Vanta,
                map_position: Vec2::new(210.0, -95.0),
                star_color: Color::srgb(0.62, 0.78, 1.0),
                sky_color: Color::srgb(0.012, 0.022, 0.05),
                planet_sprite: "planet/planet31.png",
                planet_position: Vec2::new(820.0, -250.0),
                planet_scale: 2.3,
                stations: vec![
                    StationDefinition { name: "Ashen Ring", owner: Faction::Vanta, sprite: "stations/WB_baseu2_d0.png", offset: Vec2::new(185.0, 32.0) },
                ],
            },
        ],
        current_system: 0,
        selected_system: 0,
        map_open: false,
    }
}
