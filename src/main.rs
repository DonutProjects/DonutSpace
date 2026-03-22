use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowResolution};
use rand::{rngs::StdRng, Rng, SeedableRng};

const STAR_COUNT: usize = 1400;
const STAR_FIELD_RADIUS: f32 = 9000.0;
const SHIP_ACCELERATION: f32 = 380.0;
const SHIP_DRAG: f32 = 0.985;
const SHIP_MAX_SPEED: f32 = 520.0;
const CAMERA_LERP: f32 = 6.5;
const SHIP_SCALE: f32 = 0.28;
const TURN_SPEED: f32 = 4.8;
const ENGINE_OFFSET_X: f32 = 48.5;
const ENGINE_OFFSET_Y: f32 = -136.5;
const FLAME_BASE_LENGTH: f32 = 16.0;
const FLAME_MAX_LENGTH: f32 = 34.0;
const THRUST_RAMP_UP: f32 = 1.8;
const THRUST_RAMP_DOWN: f32 = 2.8;

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.01, 0.015, 0.03)))
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
        .add_systems(Update, (player_input_system, engine_flame_system, camera_follow_system))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, MainCamera));

    spawn_starfield(&mut commands);

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
}

fn spawn_starfield(commands: &mut Commands) {
    let mut rng = StdRng::seed_from_u64(42);

    for _ in 0..STAR_COUNT {
        let x = rng.gen_range(-STAR_FIELD_RADIUS..STAR_FIELD_RADIUS);
        let y = rng.gen_range(-STAR_FIELD_RADIUS..STAR_FIELD_RADIUS);
        let size = rng.gen_range(1.0..3.8);
        let brightness = rng.gen_range(0.45..1.0);
        let tint = Color::srgba(
            brightness,
            brightness,
            brightness + 0.05,
            rng.gen_range(0.45..1.0),
        );

        commands.spawn((
            Sprite::from_color(tint, Vec2::splat(size)),
            Transform::from_xyz(x, y, -20.0),
        ));
    }
}

fn player_input_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut thruster_state)) = query.single_mut() else {
        return;
    };
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
    let ramp_speed = if wants_thrust {
        THRUST_RAMP_UP
    } else {
        THRUST_RAMP_DOWN
    };
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
            transform.scale = Vec3::ONE;

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
