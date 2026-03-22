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
const ENGINE_OFFSET_X: f32 = 48.5;
const ENGINE_OFFSET_Y: f32 = -136.5;
const FLAME_BASE_LENGTH: f32 = 16.0;
const FLAME_MAX_LENGTH: f32 = 34.0;
const THRUST_RAMP_UP: f32 = 1.8;
const THRUST_RAMP_DOWN: f32 = 2.8;
const MAP_NODE_RADIUS: f32 = 18.0;
const MAP_LEFT_SHIFT: f32 = -150.0;
const MAP_INFO_SHIFT: f32 = 345.0;
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
                player_input_system,
                engine_flame_system,
                camera_follow_system,
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

    commands.spawn((
        Sprite::from_image(asset_server.load(galaxy.systems[galaxy.selected_system].planet_sprite)),
        Transform::from_xyz(345.0, 85.0, 154.0).with_scale(Vec3::splat(2.4)),
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
    mut clear_color: ResMut<ClearColor>,
    mut galaxy: ResMut<GalaxyMap>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
    mut star_query: Query<(&StarfieldStar, &mut Sprite), Without<MapPlanetPreview>>,
    mut preview_query: Query<&mut Sprite, (With<MapPlanetPreview>, Without<StarfieldStar>)>,
) {
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
                            warp_to_system(
                                index,
                                &mut galaxy,
                                &asset_server,
                                &mut clear_color,
                                &mut player_query,
                                &mut star_query,
                                &mut preview_query,
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
        warp_to_system(
            galaxy.selected_system,
            &mut galaxy,
            &asset_server,
            &mut clear_color,
            &mut player_query,
            &mut star_query,
            &mut preview_query,
        );
    }
}

fn warp_to_system(
    target_index: usize,
    galaxy: &mut GalaxyMap,
    asset_server: &AssetServer,
    clear_color: &mut ClearColor,
    player_query: &mut Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
    star_query: &mut Query<(&StarfieldStar, &mut Sprite), Without<MapPlanetPreview>>,
    preview_query: &mut Query<&mut Sprite, (With<MapPlanetPreview>, Without<StarfieldStar>)>,
) {
    galaxy.current_system = target_index;
    galaxy.selected_system = target_index;
    galaxy.map_open = false;

    let system = &galaxy.systems[target_index];
    clear_color.0 = system.sky_color;

    if let Ok((mut transform, mut velocity, mut thruster_state)) = player_query.single_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
        velocity.0 = Vec2::ZERO;
        thruster_state.thrusting = false;
        thruster_state.intensity = 0.0;
    }

    for (star, mut sprite) in star_query.iter_mut() {
        sprite.color = tint_star(system.star_color, star.brightness);
        sprite.custom_size = Some(Vec2::splat(star.size));
    }

    if let Ok(mut sprite) = preview_query.single_mut() {
        sprite.image = asset_server.load(system.planet_sprite);
    }
}

fn player_input_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    time: Res<Time>,
    galaxy: Res<GalaxyMap>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut thruster_state)) = query.single_mut() else {
        return;
    };

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
        transform.translation = (camera_center + Vec2::new(345.0, 85.0)).extend(154.0);
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
    camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut system_text_query: Query<(&mut Text2d, &mut TextColor, &mut Transform), (With<HudSystemText>, Without<HudHintText>)>,
    mut hint_text_query: Query<&mut Transform, (With<HudHintText>, Without<HudSystemText>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_center = camera_transform.translation().truncate();
    let current_system = &galaxy.systems[galaxy.current_system];

    if let Ok((mut text, mut color, mut transform)) = system_text_query.single_mut() {
        *text = Text2d::new(format!(
            "Current system: {}\nHolding: {}\nVisible stations: {}",
            current_system.name,
            current_system.faction.name(),
            current_system.stations.len()
        ));
        *color = TextColor(current_system.faction.color());
        transform.translation = (camera_center + Vec2::new(-770.0, 430.0)).extend(200.0);
    }

    if let Ok(mut transform) = hint_text_query.single_mut() {
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
