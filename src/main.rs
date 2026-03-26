use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowResolution};
use rand::{rngs::StdRng, Rng, SeedableRng};

// ── Gameplay constants ─────────────────────────────────────────────────────
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
const PLANET_RENDER_Z: f32 = -5.0;
const STATION_RENDER_Z: f32 = -4.0;
const STATION_UNIFORM_SIZE: f32 = 250.0;
const STATION_DOCK_RADIUS: f32 = 92.0;
const TARGET_MARKER_Z: f32 = 25.0;
const TARGET_SELECT_RADIUS: f32 = 130.0;
const AUTO_DOCK_SPEED: f32 = 310.0;
const PLAYER_CARGO_CAPACITY: i32 = 80;
const MAP_LEFT_SHIFT: f32 = -150.0;
const MAP_PREVIEW_Z: f32 = 151.5;

// ── Palette ────────────────────────────────────────────────────────────────
const C_BG: Color      = Color::srgb(0.020, 0.040, 0.070);
const C_PANEL: Color   = Color::srgba(0.04, 0.09, 0.14, 0.94);
const C_ACCENT: Color  = Color::srgb(0.00, 0.71, 1.00);
const C_BORDER: Color  = Color::srgba(0.10, 0.26, 0.42, 0.85);
const C_BORDER2: Color = Color::srgba(0.00, 0.71, 1.00, 0.60);
const C_TEXT: Color    = Color::srgba(0.76, 0.87, 0.96, 0.97);
const C_MUTED: Color   = Color::srgba(0.36, 0.52, 0.66, 0.90);
const C_WARN: Color    = Color::srgb(1.00, 0.72, 0.28);
const C_SUCCESS: Color = Color::srgb(0.16, 1.00, 0.64);
const C_DANGER: Color  = Color::srgb(1.00, 0.28, 0.40);
const C_HELIOS: Color  = Color::srgb(1.00, 0.73, 0.28);
const C_VANTA: Color   = Color::srgb(0.42, 0.71, 1.00);
const C_BTN: Color     = Color::srgba(0.08, 0.20, 0.34, 0.92);
const C_BTN_HOV: Color = Color::srgba(0.12, 0.30, 0.50, 0.98);
const C_BTN_ACT: Color = Color::srgba(0.04, 0.34, 0.20, 0.98);
const C_NONE: Color    = Color::srgba(0.0, 0.0, 0.0, 0.0);

// ── World-space gameplay components ───────────────────────────────────────
#[derive(Component)] struct Player;
#[derive(Component, Default)] struct Velocity(Vec2);
#[derive(Component, Default)] struct ThrusterState { thrusting: bool, intensity: f32 }
#[derive(Component)] struct MainCamera;
#[derive(Component, Clone, Copy)] struct EngineFlame { side: f32 }
#[derive(Component)] struct StarfieldStar { size: f32, brightness: f32 }
#[derive(Component)] struct CurrentSystemPlanet;
#[derive(Component)] struct CurrentSystemStation { slot: usize }
#[derive(Component)] struct TargetMarker;

// ── World-space map components ─────────────────────────────────────────────
#[derive(Component)] struct MapElement;
#[derive(Component)] struct MapBackdrop;
#[derive(Component)] struct MapSelection;
#[derive(Component)] struct MapLink { a: usize, b: usize }
#[derive(Component)] struct MapNode { index: usize }
#[derive(Component)] struct MapNodeLabel { index: usize }
#[derive(Component)] struct MapPlanetPreview;

// ── Bevy UI tags ───────────────────────────────────────────────────────────
#[derive(Component)] struct HudRoot;
#[derive(Component)] struct HudSystemText;
#[derive(Component)] struct HudHintText;

#[derive(Component)] struct TargetPanelRoot;
#[derive(Component)] struct TargetPanelTitle;
#[derive(Component)] struct TargetPanelOwner;
#[derive(Component)] struct TargetActionBtn { action: TargetAction }

#[derive(Component)] struct StationUiRoot;
#[derive(Component)] struct StationHeaderTitle;
#[derive(Component)] struct StationHeaderMeta;
#[derive(Component)] struct StationNameText;
#[derive(Component)] struct StationFactionText;
#[derive(Component)] struct StationTabBtn { tab: StationServiceTab }
#[derive(Component)] struct StationTabLabel;
#[derive(Component)] struct StationTradeRoot;
#[derive(Component)] struct StationDetailRoot;
#[derive(Component)] struct StationPanelTitle;
#[derive(Component)] struct StationPanelBody;
#[derive(Component)] struct StationCreditsCardText;
#[derive(Component)] struct StationCargoCardText;
#[derive(Component)] struct PlayerCreditsText;
#[derive(Component)] struct PlayerCargoText;
#[derive(Component)] struct PlayerHullText;
#[derive(Component, Clone, Copy)] struct StationTradePriceText { commodity: CommodityId }
#[derive(Component, Clone, Copy)] struct StationTradeStockText { commodity: CommodityId }
#[derive(Component, Clone, Copy)] struct StationTradeTrendText { commodity: CommodityId }
#[derive(Component, Clone, Copy)] struct StationBuyBtn { commodity: CommodityId }
#[derive(Component)] struct StationBuyBtnLabel;
#[derive(Component)] struct StationUndockBtn;
#[derive(Component)] struct StationUndockBtnLabel;

#[derive(Component)] struct MapUiRoot;
#[derive(Component)] struct MapInfoTitle;
#[derive(Component)] struct MapInfoFaction;
#[derive(Component)] struct MapInfoPlanet;
#[derive(Component)] struct MapInfoClass;
#[derive(Component)] struct MapInfoStations;
#[derive(Component)] struct MapInfoStatus;
#[derive(Component)] struct MapInfoSecurity;
#[derive(Component)] struct MapWarpBtn;
#[derive(Component)] struct MapWarpBtnLabel;

// ── Resources ──────────────────────────────────────────────────────────────
#[derive(Resource)]
struct GalaxyMap {
    systems: Vec<SystemDefinition>,
    links: Vec<(usize, usize)>,
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

#[derive(Resource, Default)] struct StationLanding { docked: bool }

#[derive(Resource)]
struct StationServices {
    selected_tab: StationServiceTab,
    credits: i32,
    cargo: i32,
    hull: i32,
    mission: Option<StationMission>,
}

#[derive(Resource, Default)]
struct TargetingState { selected_station: Option<usize>, auto_dock: bool }

// ── Enums ──────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum WarpPhase { #[default] Idle, Align, Depart, Cruise, Arrive }

#[derive(Clone, Copy, PartialEq, Eq)]
enum StationServiceTab { Trade, Missions, Repair }

#[derive(Clone, Copy)]
struct StationMission { target_system: usize, reward: i32, accepted_in: usize }

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetAction { Approach, Dock, Clear }

impl StationServiceTab {
    fn title(self) -> &'static str {
        match self {
            StationServiceTab::Trade    => "TRADE",
            StationServiceTab::Missions => "MISSIONS",
            StationServiceTab::Repair   => "REPAIR",
        }
    }
}

impl TargetAction {
    fn label(self) -> &'static str {
        match self {
            TargetAction::Approach => "->  Approach",
            TargetAction::Dock     => "o   Dock  [E]",
            TargetAction::Clear    => "x   Clear  [Q]",
        }
    }
}

impl Default for StationServices {
    fn default() -> Self {
        Self { selected_tab: StationServiceTab::Trade, credits: 2_000, cargo: 0, hull: 76, mission: None }
    }
}

// ── World data ─────────────────────────────────────────────────────────────
#[derive(Clone)]
struct SystemDefinition {
    name: &'static str,
    faction: Faction,
    map_position: Vec2,
    star_color: Color,
    sky_color: Color,
    planet_sprite: &'static str,
    planet_name: &'static str,
    planet_class: &'static str,
    security: &'static str,
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
    market: Vec<MarketOffer>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Faction { Helios, Vanta }

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommodityId { Ore, Fuel, Tech, Food }

#[derive(Clone, Copy)]
struct MarketOffer {
    commodity: CommodityId,
    price: i32,
    stock: i32,
    trend: i32,
}

impl Faction {
    fn name(self) -> &'static str {
        match self { Faction::Helios => "Helios Pact", Faction::Vanta => "Vanta Clade" }
    }
    fn color(self) -> Color {
        match self { Faction::Helios => C_HELIOS, Faction::Vanta => C_VANTA }
    }
}

impl CommodityId {
    fn label(self) -> &'static str {
        match self {
            CommodityId::Ore => "Ore",
            CommodityId::Fuel => "Fuel",
            CommodityId::Tech => "Tech",
            CommodityId::Food => "Food",
        }
    }
}

// ── Main ───────────────────────────────────────────────────────────────────
fn main() {
    App::new()
        .insert_resource(ClearColor(C_BG))
        .insert_resource(authored_galaxy())
        .insert_resource(WarpDrive::default())
        .insert_resource(StationLanding::default())
        .insert_resource(StationServices::default())
        .insert_resource(TargetingState::default())
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
        .add_systems(Update, (
            map_input_system,
            warp_travel_system,
            station_target_input_system,
            station_auto_dock_system,
            station_landing_system,
            station_services_input_system,
            player_input_system,
            engine_flame_system,
            camera_follow_system,
        ).chain())
        .add_systems(Update, (
            update_local_system_visuals_system,
            update_map_nodes_system,
            update_map_panels_system,
            update_hud_system,
            update_target_panel_system,
            target_button_interaction_system,
            update_station_visibility_system,
            update_station_header_title_system,
            update_station_header_meta_system,
            update_station_name_system,
            update_station_faction_system,
        ))
        .add_systems(Update, (
            update_station_resource_text_system,
            update_station_tab_visuals_system,
            update_station_detail_ui_system,
            update_station_trade_ui_system,
            update_station_hull_text_system,
            update_station_button_visuals_system,
            station_buy_button_system,
            station_undock_button_system,
            station_tab_button_system,
            update_map_ui_system,
            update_map_warp_button_visual_system,
            map_warp_button_system,
        ))
        .run();
}

// ── Setup ──────────────────────────────────────────────────────────────────
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, galaxy: Res<GalaxyMap>) {
    commands.spawn((Camera2d, MainCamera));
    spawn_starfield(&mut commands, &galaxy);

    // Ship
    commands.spawn((
        Sprite::from_image(asset_server.load("ship1.png")),
        Transform::from_xyz(0.0, 0.0, 10.0).with_scale(Vec3::splat(SHIP_SCALE)),
        Player, Velocity::default(), ThrusterState::default(),
    ));

    // Engine flames
    for side in [-1.0_f32, 1.0] {
        commands.spawn((
            Sprite::from_color(Color::srgba(0.45, 0.82, 1.0, 0.75), Vec2::new(8.0, FLAME_BASE_LENGTH)),
            bevy::sprite::Anchor::TOP_CENTER,
            Transform::from_xyz(side * ENGINE_OFFSET_X * SHIP_SCALE, ENGINE_OFFSET_Y * SHIP_SCALE, 9.0),
            Visibility::Hidden,
            EngineFlame { side },
        ));
    }

    // Planet + station sprites
    let cs = &galaxy.systems[galaxy.current_system];
    commands.spawn((
        Sprite::from_image(asset_server.load(cs.planet_sprite)),
        Transform::from_xyz(cs.planet_position.x, cs.planet_position.y, PLANET_RENDER_Z)
            .with_scale(Vec3::splat(cs.planet_scale)),
        CurrentSystemPlanet,
    ));
    let mut ss = Sprite::from_image(asset_server.load(
        cs.stations.first().map(|s| s.sprite).unwrap_or("stations/SS1.png"),
    ));
    ss.custom_size = Some(Vec2::splat(STATION_UNIFORM_SIZE));
    commands.spawn((
        ss,
        Transform::from_xyz(cs.planet_position.x, cs.planet_position.y, STATION_RENDER_Z),
        if cs.stations.is_empty() { Visibility::Hidden } else { Visibility::Inherited },
        CurrentSystemStation { slot: 0 },
    ));

    // Target marker (world-space)
    commands.spawn((
        Sprite::from_color(Color::srgba(0.0, 0.71, 1.0, 0.14), Vec2::new(290.0, 290.0)),
        Transform::from_xyz(0.0, 0.0, TARGET_MARKER_Z),
        Visibility::Hidden,
        TargetMarker,
    ));

    // World-space map
    spawn_map_world(&mut commands, &asset_server, &galaxy);

    // ── Bevy UI ───────────────────────────────────────────────────────────
    spawn_hud(&mut commands);
    spawn_target_panel(&mut commands);
    spawn_station_panel(&mut commands, &galaxy);
    spawn_map_ui_sidebar(&mut commands);
}

// ── UI helpers ─────────────────────────────────────────────────────────────
fn txt(s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (Text::new(s.into()), TextFont { font_size: size, ..default() }, TextColor(color))
}

fn spawn_hud(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        HudRoot,
        Pickable::IGNORE,
    )).with_children(|root| {
        // Top row
        root.spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            ..default()
        }).with_children(|top| {
            // Top-left panel
            top.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(3.0),
                    border: UiRect { top: Val::Px(2.0), ..UiRect::all(Val::Px(1.0)) },
                    min_width: Val::Px(220.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.09, 0.14, 0.90)),
                BorderColor { top: C_ACCENT, ..BorderColor::all(C_BORDER) },
            )).with_children(|p| {
                p.spawn((txt("", 13.0, C_TEXT), HudSystemText));
            });
        });

        // Bottom hint bar
        root.spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::axes(Val::Px(16.0), Val::Px(7.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.09, 0.14, 0.85)),
            BorderColor::all(C_BORDER),
        )).with_children(|bar| {
            bar.spawn((txt("", 12.0, C_MUTED), HudHintText));
        });
    });
}

fn spawn_target_panel(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            top: Val::Px(80.0),
            width: Val::Px(260.0),
            flex_direction: FlexDirection::Column,
            border: UiRect { top: Val::Px(2.0), ..UiRect::all(Val::Px(1.0)) },
            ..default()
        },
        BackgroundColor(C_PANEL),
        BorderColor { top: C_ACCENT, ..BorderColor::all(C_BORDER2) },
        Visibility::Hidden,
        TargetPanelRoot,
    )).with_children(|panel| {
        // Header
        panel.spawn((
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                border: UiRect { bottom: Val::Px(1.0), ..default() },
                ..default()
            },
            BorderColor::all(C_BORDER),
        )).with_children(|h| {
            h.spawn((txt("NO TARGET", 14.0, C_ACCENT), TargetPanelTitle));
            h.spawn((txt("", 11.0, C_MUTED), TargetPanelOwner));
        });

        // Buttons
        panel.spawn(Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            row_gap: Val::Px(6.0),
            ..default()
        }).with_children(|body| {
            for action in [TargetAction::Approach, TargetAction::Dock, TargetAction::Clear] {
                body.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(9.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(C_BTN),
                    BorderColor::all(C_BORDER),
                    TargetActionBtn { action },
                )).with_children(|btn| {
                    btn.spawn(txt(action.label(), 12.0, C_TEXT));
                });
            }
        });
    });
}

fn spawn_station_panel(commands: &mut Commands, galaxy: &Res<GalaxyMap>) {
    let cs = &galaxy.systems[galaxy.current_system];
    let station_name = cs.stations.first().map(|s| s.name).unwrap_or("Station");

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.88)),
        Visibility::Hidden,
        StationUiRoot,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            border: UiRect { top: Val::Px(1.0), ..default() },
            ..default()
        },
        BackgroundColor(C_PANEL),
        BorderColor { top: C_ACCENT, ..BorderColor::all(C_BORDER) },
        Visibility::Hidden,
        StationUiRoot,
    )).with_children(|panel| {
        panel.spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                border: UiRect { bottom: Val::Px(1.0), ..default() },
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.10, 0.17, 0.98)),
            BorderColor::all(C_BORDER),
        )).with_children(|header| {
            header.spawn((txt("STATION: HEARTH PORT", 13.0, C_ACCENT), StationHeaderTitle));
            header.spawn((txt("Helios Pact   |   System Hearthlight", 11.0, C_MUTED), StationHeaderMeta));
        });

        panel.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        }).with_children(|frame| {
            frame.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect { right: Val::Px(1.0), ..default() },
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.07, 0.12, 0.98)),
                BorderColor::all(C_BORDER),
            )).with_children(|sidebar| {
                sidebar.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(7.0),
                        border: UiRect { bottom: Val::Px(1.0), ..default() },
                        ..default()
                    },
                    BorderColor::all(C_BORDER),
                )).with_children(|station_box| {
                    station_box.spawn((
                        Node {
                            width: Val::Px(54.0),
                            height: Val::Px(54.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(C_PANEL),
                        BorderColor::all(C_BORDER2),
                    )).with_children(|hex| { hex.spawn(txt("⬡", 17.0, C_TEXT)); });
                    station_box.spawn((txt(station_name, 13.0, C_ACCENT), StationNameText));
                    station_box.spawn((txt(cs.faction.name(), 10.0, cs.faction.color()), StationFactionText));
                });

                for tab in [StationServiceTab::Trade, StationServiceTab::Missions, StationServiceTab::Repair] {
                    sidebar.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                            border: UiRect { left: Val::Px(3.0), bottom: Val::Px(1.0), ..default() },
                            ..default()
                        },
                        BackgroundColor(C_NONE),
                        BorderColor { left: C_NONE, bottom: C_BORDER, ..BorderColor::all(C_NONE) },
                        StationTabBtn { tab },
                    )).with_children(|btn| { btn.spawn((txt(tab.title(), 12.0, C_MUTED), StationTabLabel)); });
                }

                sidebar.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(14.0)),
                        row_gap: Val::Px(5.0),
                        margin: UiRect { top: Val::Auto, ..default() },
                        border: UiRect { top: Val::Px(1.0), ..default() },
                        ..default()
                    },
                    BorderColor::all(C_BORDER),
                )).with_children(|stats| {
                    stats.spawn(txt("PLAYER", 10.0, C_MUTED));
                    stats.spawn((txt("Credits: 2 000 C", 12.0, C_WARN), PlayerCreditsText));
                    stats.spawn((txt("Cargo: 0 / 80 T", 12.0, C_TEXT), PlayerCargoText));
                    stats.spawn((txt("Hull: 76%", 12.0, C_SUCCESS), PlayerHullText));
                });
            });

            frame.spawn(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(18.0)),
                row_gap: Val::Px(16.0),
                ..default()
            }).with_children(|content| {
                content.spawn(Node {
                    width: Val::Percent(100.0),
                    column_gap: Val::Px(16.0),
                    ..default()
                }).with_children(|cards| {
                    cards.spawn((
                        Node {
                            width: Val::Px(230.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(8.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.03, 0.10, 0.18, 0.98)),
                        BorderColor::all(C_BORDER),
                    )).with_children(|card| {
                        card.spawn(txt("CREDITS", 10.0, C_MUTED));
                        card.spawn((txt("2 000 C", 22.0, C_WARN), StationCreditsCardText));
                    });

                    cards.spawn((
                        Node {
                            width: Val::Px(230.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(8.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.03, 0.10, 0.18, 0.98)),
                        BorderColor::all(C_BORDER),
                    )).with_children(|card| {
                        card.spawn(txt("CARGO", 10.0, C_MUTED));
                        card.spawn((txt("0 T", 22.0, C_ACCENT), StationCargoCardText));
                    });
                });

                content.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        border: UiRect { top: Val::Px(1.0), bottom: Val::Px(1.0), ..default() },
                        ..default()
                    },
                    BorderColor::all(C_BORDER),
                    StationTradeRoot,
                )).with_children(|trade| {
                    trade.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                            column_gap: Val::Px(12.0),
                            ..default()
                        },
                        BorderColor::all(C_BORDER),
                    )).with_children(|hdr| {
                        hdr.spawn(Node { width: Val::Px(150.0), ..default() }).with_children(|c| { c.spawn(txt("ITEM", 11.0, C_MUTED)); });
                        hdr.spawn(Node { width: Val::Px(110.0), ..default() }).with_children(|c| { c.spawn(txt("PRICE", 11.0, C_MUTED)); });
                        hdr.spawn(Node { width: Val::Px(120.0), ..default() }).with_children(|c| { c.spawn(txt("STOCK", 11.0, C_MUTED)); });
                        hdr.spawn(Node { width: Val::Px(90.0), ..default() }).with_children(|c| { c.spawn(txt("TREND", 11.0, C_MUTED)); });
                        hdr.spawn(Node { flex_grow: 1.0, ..default() });
                    });

                    for commodity in [CommodityId::Ore, CommodityId::Fuel, CommodityId::Tech, CommodityId::Food] {
                        trade.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                                column_gap: Val::Px(12.0),
                                border: UiRect { bottom: Val::Px(1.0), ..default() },
                                ..default()
                            },
                            BorderColor::all(C_BORDER),
                        )).with_children(|row| {
                            row.spawn(Node { width: Val::Px(150.0), ..default() }).with_children(|c| {
                                c.spawn(txt(commodity.label(), 13.0, C_TEXT));
                            });
                            row.spawn(Node { width: Val::Px(110.0), ..default() }).with_children(|c| {
                                c.spawn((txt("--", 12.0, C_TEXT), StationTradePriceText { commodity }));
                            });
                            row.spawn(Node { width: Val::Px(120.0), ..default() }).with_children(|c| {
                                c.spawn((txt("--", 12.0, C_TEXT), StationTradeStockText { commodity }));
                            });
                            row.spawn(Node { width: Val::Px(90.0), ..default() }).with_children(|c| {
                                c.spawn((txt("--", 12.0, C_MUTED), StationTradeTrendText { commodity }));
                            });
                            row.spawn(Node {
                                flex_grow: 1.0,
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                            }).with_children(|cta| {
                                cta.spawn((
                                    Button,
                                    Node {
                                        min_width: Val::Px(112.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::axes(Val::Px(16.0), Val::Px(7.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(C_NONE),
                                    BorderColor::all(C_BORDER),
                                    StationBuyBtn { commodity },
                                )).with_children(|btn| {
                                    btn.spawn((txt("BUY", 12.0, C_TEXT), StationBuyBtnLabel));
                                });
                            });
                        });
                    }
                });

                content.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    StationDetailRoot,
                )).with_children(|detail| {
                    detail.spawn((txt("TRADE / HEARTHLIGHT", 18.0, C_ACCENT), StationPanelTitle));
                    detail.spawn((txt("", 13.0, C_TEXT), StationPanelBody));
                });

                content.spawn(Node { margin: UiRect { top: Val::Auto, ..default() }, ..default() })
                .with_children(|footer| {
                    footer.spawn((
                        Button,
                        Node {
                            min_width: Val::Px(194.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(11.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(C_NONE),
                        BorderColor::all(C_BORDER),
                        StationUndockBtn,
                    )).with_children(|btn| {
                        btn.spawn((txt("UNDOCK [E]", 13.0, C_TEXT), StationUndockBtnLabel));
                    });
                });
            });
        });
    });
}

fn spawn_map_ui_sidebar(commands: &mut Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            top: Val::Px(54.0),
            bottom: Val::Px(18.0),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(14.0)),
            column_gap: Val::Px(12.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.07, 0.12, 0.82)),
        BorderColor::all(C_BORDER),
        Visibility::Hidden,
        MapUiRoot,
    )).with_children(|root| {
        root.spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(C_NONE),
            BorderColor::all(C_BORDER),
        ));

        root.spawn((
            Node {
                width: Val::Px(280.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(C_PANEL),
            BorderColor::all(C_BORDER),
        )).with_children(|sb| {
            sb.spawn((txt("--", 22.0, C_ACCENT), MapInfoTitle));
            sb.spawn((txt("", 11.0, C_HELIOS), MapInfoFaction));
            sb.spawn((Node { height: Val::Px(1.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(C_BORDER)));

            for (label, tag_kind) in [
                ("Planet", 0usize),
                ("Class", 1usize),
                ("Stations", 2usize),
                ("Status", 3usize),
                ("Security", 4usize),
            ] {
                sb.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect { bottom: Val::Px(6.0), ..default() },
                        border: UiRect { bottom: Val::Px(1.0), ..default() },
                        ..default()
                    },
                    BorderColor::all(C_BORDER),
                )).with_children(|row| {
                    row.spawn(txt(label, 12.0, C_MUTED));
                    match tag_kind {
                        0 => { row.spawn((txt("--", 12.0, C_TEXT), MapInfoPlanet)); }
                        1 => { row.spawn((txt("--", 12.0, C_TEXT), MapInfoClass)); }
                        2 => { row.spawn((txt("--", 12.0, C_TEXT), MapInfoStations)); }
                        3 => { row.spawn((txt("--", 12.0, C_SUCCESS), MapInfoStatus)); }
                        _ => { row.spawn((txt("--", 12.0, C_SUCCESS), MapInfoSecurity)); }
                    }
                });
            }

            sb.spawn(Node { margin: UiRect { top: Val::Auto, ..default() }, ..default() })
            .with_children(|footer| {
                footer.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(11.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(C_NONE),
                    BorderColor::all(C_BORDER),
                    MapWarpBtn,
                )).with_children(|btn| {
                    btn.spawn((txt("WARP TRANSIT", 13.0, C_MUTED), MapWarpBtnLabel));
                });
            });
        });
    });
}

fn spawn_map_world(commands: &mut Commands, asset_server: &Res<AssetServer>, galaxy: &Res<GalaxyMap>) {
    commands.spawn((
        Sprite::from_color(Color::srgba(0.02, 0.05, 0.09, 0.96), Vec2::new(760.0, 640.0)),
        Transform::from_xyz(-150.0, 0.0, 150.0),
        Visibility::Hidden, MapElement, MapBackdrop,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgba(0.12, 0.18, 0.26, 0.85), Vec2::new(760.0, 1.0)),
        Transform::from_xyz(-150.0, 321.0, 150.5),
        Visibility::Hidden, MapElement, MapBackdrop,
    ));

    for &(a, b) in &galaxy.links {
        commands.spawn((
            Sprite::from_color(Color::srgba(0.12, 0.31, 0.55, 0.35), Vec2::new(1.0, 1.0)),
            Transform::from_xyz(MAP_LEFT_SHIFT, 0.0, 151.0),
            Visibility::Hidden, MapElement, MapLink { a, b },
        ));
    }

    commands.spawn((
        Sprite::from_color(Color::srgba(0.98, 0.71, 0.18, 0.15), Vec2::splat(28.0)),
        Transform::from_xyz(MAP_LEFT_SHIFT, 0.0, 153.0),
        Visibility::Hidden, MapElement, MapSelection,
    ));
    for (index, system) in galaxy.systems.iter().enumerate() {
        commands.spawn((
            Sprite::from_color(system.faction.color(), Vec2::splat(10.0)),
            Transform::from_xyz(MAP_LEFT_SHIFT + system.map_position.x, system.map_position.y, 152.0),
            Visibility::Hidden, MapElement, MapNode { index },
        ));
        commands.spawn((
            Text2d::new(system.name),
            TextFont::from_font_size(12.0),
            TextColor(C_MUTED),
            bevy::sprite::Anchor::TOP_CENTER,
            Transform::from_xyz(MAP_LEFT_SHIFT + system.map_position.x, system.map_position.y - 14.0, 154.0),
            Visibility::Hidden, MapElement, MapNodeLabel { index },
        ));
    }
    let mut preview = Sprite::from_image(asset_server.load(galaxy.systems[galaxy.selected_system].planet_sprite));
    preview.color = Color::srgba(1.0, 1.0, 1.0, 0.18);
    commands.spawn((
        preview,
        Transform::from_xyz(-380.0, 40.0, MAP_PREVIEW_Z).with_scale(Vec3::splat(1.2)),
        Visibility::Hidden, MapElement, MapPlanetPreview,
    ));
}

// ── UI update systems ──────────────────────────────────────────────────────
fn update_hud_system(
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    landing: Res<StationLanding>,
    targeting: Res<TargetingState>,
    player_query: Query<&Transform, With<Player>>,
    mut sys_q: Query<&mut Text, (With<HudSystemText>, Without<HudHintText>)>,
    mut hint_q: Query<&mut Text, (With<HudHintText>, Without<HudSystemText>)>,
) {
    let current = &galaxy.systems[galaxy.current_system];

    let warp_line = if warp.active {
        let dest = &galaxy.systems[warp.target_system];
        let phase = match warp.phase {
            WarpPhase::Align => "ALIGN", WarpPhase::Depart => "DEPART",
            WarpPhase::Cruise => "CRUISE", WarpPhase::Arrive => "ARRIVE", WarpPhase::Idle => "IDLE",
        };
        format!("WARP   {} -> {}\nPHASE  {}", current.name, dest.name, phase)
    } else { "WARP   OFFLINE".to_string() };

    let dock_line = if landing.docked { "DOCK   LANDED".to_string() } else { "DOCK   IN FLIGHT".to_string() };
    let target_line = match targeting.selected_station {
        Some(slot) => {
            let name = current.stations.get(slot).map(|s| s.name).unwrap_or("UNKNOWN");
            if targeting.auto_dock { format!("TGT    {} [AUTO]", name) } else { format!("TGT    {}", name) }
        }
        None => "TGT    NONE".to_string(),
    };

    if let Ok(mut t) = sys_q.single_mut() {
        *t = Text::new(format!(
            "SYS    {}\nFACT   {}\n---------------------\n{}\n{}\n{}",
            current.name, current.faction.name(), warp_line, dock_line, target_line
        ));
    }

    let near_station = current.stations.first().and_then(|station| {
        let Ok(pt) = player_query.single() else { return None; };
        Some(pt.translation.truncate().distance(current.planet_position + station.offset) <= STATION_DOCK_RADIUS)
    }).unwrap_or(false);

    let hint = if landing.docked {
        "[ 1 ] Trade    [ 2 ] Missions    [ 3 ] Repair    [ E ] Undock"
    } else if warp.active { "Warp in progress..." }
    else if near_station { "[ LMB ] Thrust    [ E ] Dock    [ TAB ] Map    [ Q ] Clear target" }
    else if galaxy.map_open { "[ TAB / ESC ] Close map    [ Click x2 / Enter ] Warp to selected system" }
    else { "[ TAB ] Star map    [ LMB ] Thrust / Select    [ E ] Dock    [ Q ] Clear target" };

    if let Ok(mut t) = hint_q.single_mut() { *t = Text::new(hint); }
}

fn update_target_panel_system(
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    landing: Res<StationLanding>,
    targeting: Res<TargetingState>,
    mut panel_vis: Query<&mut Visibility, With<TargetPanelRoot>>,
    mut title_q: Query<&mut Text, (With<TargetPanelTitle>, Without<TargetPanelOwner>)>,
    mut owner_q: Query<&mut Text, (With<TargetPanelOwner>, Without<TargetPanelTitle>)>,
    mut marker_q: Query<(&mut Transform, &mut Visibility), (With<TargetMarker>, Without<TargetPanelRoot>)>,
) {
    let show = targeting.selected_station.is_some() && !warp.active && !landing.docked && !galaxy.map_open;
    if let Ok(mut vis) = panel_vis.single_mut() {
        *vis = if show { Visibility::Inherited } else { Visibility::Hidden };
    }
    let station = targeting.selected_station
        .and_then(|slot| galaxy.systems[galaxy.current_system].stations.get(slot));
    if let Ok(mut t) = title_q.single_mut() {
        *t = Text::new(station.map(|s| s.name.to_uppercase()).unwrap_or("NO TARGET".into()));
    }
    if let Ok(mut t) = owner_q.single_mut() {
        *t = Text::new(station.map(|s| format!("{} - Station", s.owner.name())).unwrap_or_default());
    }
    if let Ok((mut mt, mut mv)) = marker_q.single_mut() {
        if let Some(slot) = targeting.selected_station {
            if let Some(sdef) = galaxy.systems[galaxy.current_system].stations.get(slot) {
                let pos = galaxy.systems[galaxy.current_system].planet_position + sdef.offset;
                mt.translation = pos.extend(TARGET_MARKER_Z);
                *mv = if show { Visibility::Inherited } else { Visibility::Hidden };
                return;
            }
        }
        *mv = Visibility::Hidden;
    }
}

fn target_button_interaction_system(
    mut btn_q: Query<(&Interaction, &mut BackgroundColor, &mut BorderColor, &TargetActionBtn), Changed<Interaction>>,
    targeting: Res<TargetingState>,
) {
    for (interaction, mut bg, mut bc, tab) in &mut btn_q {
        let is_active = tab.action == TargetAction::Dock && targeting.auto_dock;
        match interaction {
            Interaction::Hovered => { *bg = BackgroundColor(C_BTN_HOV); *bc = BorderColor::all(C_ACCENT); }
            Interaction::None => {
                *bg = BackgroundColor(if is_active { C_BTN_ACT } else { C_BTN });
                *bc = BorderColor::all(if is_active { C_SUCCESS } else { C_BORDER });
            }
            Interaction::Pressed => { *bg = BackgroundColor(C_BTN_ACT); }
        }
    }
}

fn update_station_visibility_system(
    landing: Res<StationLanding>,
    services: Res<StationServices>,
    mut vis_q: ParamSet<(
        Query<&mut Visibility, With<StationUiRoot>>,
        Query<&mut Visibility, With<StationTradeRoot>>,
        Query<&mut Visibility, With<StationDetailRoot>>,
    )>,
) {
    let root_vis = if landing.docked { Visibility::Inherited } else { Visibility::Hidden };
    for mut vis in &mut vis_q.p0() { *vis = root_vis; }
    if !landing.docked { return; }
    for mut vis in &mut vis_q.p1() {
        *vis = if services.selected_tab == StationServiceTab::Trade { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut vis in &mut vis_q.p2() {
        *vis = if services.selected_tab == StationServiceTab::Trade { Visibility::Hidden } else { Visibility::Inherited };
    }
}

fn update_station_header_title_system(
    galaxy: Res<GalaxyMap>,
    landing: Res<StationLanding>,
    mut title_q: Query<&mut Text, With<StationHeaderTitle>>,
) {
    if !landing.docked { return; }

    let sys = &galaxy.systems[galaxy.current_system];
    let station = sys.stations.first();
    let station_name = station.map(|s| s.name).unwrap_or("Station");

    if let Ok(mut t) = title_q.single_mut() {
        *t = Text::new(format!("STATION: {}", station_name.to_uppercase()));
    }
}

fn update_station_header_meta_system(
    galaxy: Res<GalaxyMap>,
    landing: Res<StationLanding>,
    mut meta_q: Query<&mut Text, With<StationHeaderMeta>>,
) {
    if !landing.docked { return; }
    let sys = &galaxy.systems[galaxy.current_system];
    let station_owner = sys.stations.first().map(|s| s.owner).unwrap_or(sys.faction);
    if let Ok(mut t) = meta_q.single_mut() {
        *t = Text::new(format!("{}   |   System {}", station_owner.name(), sys.name));
    }
}

fn update_station_name_system(
    galaxy: Res<GalaxyMap>,
    landing: Res<StationLanding>,
    mut name_q: Query<&mut Text, With<StationNameText>>,
) {
    if !landing.docked { return; }
    let sys = &galaxy.systems[galaxy.current_system];
    let station = sys.stations.first();
    let station_name = station.map(|s| s.name).unwrap_or("Station");

    if let Ok(mut t) = name_q.single_mut() {
        *t = Text::new(station_name);
    }
}

fn update_station_faction_system(
    galaxy: Res<GalaxyMap>,
    landing: Res<StationLanding>,
    mut faction_q: Query<(&mut Text, &mut TextColor), With<StationFactionText>>,
) {
    if !landing.docked { return; }
    let sys = &galaxy.systems[galaxy.current_system];
    let station_owner = sys.stations.first().map(|s| s.owner).unwrap_or(sys.faction);
    if let Ok((mut t, mut tc)) = faction_q.single_mut() {
        *t = Text::new(station_owner.name());
        tc.0 = station_owner.color();
    }
}

fn update_station_resource_text_system(
    landing: Res<StationLanding>,
    services: Res<StationServices>,
    mut text_q: ParamSet<(
        Query<&mut Text, With<StationCreditsCardText>>,
        Query<&mut Text, With<StationCargoCardText>>,
        Query<&mut Text, With<PlayerCreditsText>>,
        Query<&mut Text, With<PlayerCargoText>>,
    )>,
) {
    if !landing.docked { return; }
    if let Ok(mut t) = text_q.p0().single_mut() { *t = Text::new(format_credits(services.credits)); }
    if let Ok(mut t) = text_q.p1().single_mut() { *t = Text::new(format!("{} T", services.cargo)); }
    if let Ok(mut t) = text_q.p2().single_mut() { *t = Text::new(format!("Credits: {}", format_credits(services.credits))); }
    if let Ok(mut t) = text_q.p3().single_mut() { *t = Text::new(format!("Cargo: {} / {} T", services.cargo, PLAYER_CARGO_CAPACITY)); }
}

fn update_station_tab_visuals_system(
    landing: Res<StationLanding>,
    services: Res<StationServices>,
    mut tab_q: Query<(&StationTabBtn, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut tab_txt_q: Query<&mut TextColor, With<StationTabLabel>>,
) {
    if !landing.docked { return; }
    for (tab_btn, mut bg, mut bc, children) in &mut tab_q {
        let active = tab_btn.tab == services.selected_tab;
        *bg = BackgroundColor(if active { Color::srgba(0.0, 0.71, 1.0, 0.10) } else { C_NONE });
        *bc = BorderColor {
            left: if active { C_ACCENT } else { C_NONE },
            bottom: C_BORDER,
            ..BorderColor::all(C_NONE)
        };
        for child in children.iter() {
            if let Ok(mut tc) = tab_txt_q.get_mut(child) {
                tc.0 = if active { C_ACCENT } else { C_MUTED };
            }
        }
    }
}

fn update_station_hull_text_system(
    landing: Res<StationLanding>,
    services: Res<StationServices>,
    mut hull_q: Query<&mut Text, With<PlayerHullText>>,
) {
    if !landing.docked { return; }
    if let Ok(mut t) = hull_q.single_mut() {
        *t = Text::new(format!("Hull: {}%", services.hull));
    }
}

fn update_station_detail_ui_system(
    galaxy: Res<GalaxyMap>,
    landing: Res<StationLanding>,
    services: Res<StationServices>,
    mut text_q: ParamSet<(
        Query<&mut Text, With<StationPanelTitle>>,
        Query<&mut Text, With<StationPanelBody>>,
    )>,
) {
    if !landing.docked || services.selected_tab == StationServiceTab::Trade { return; }
    let sys = &galaxy.systems[galaxy.current_system];
    if let Ok(mut t) = text_q.p0().single_mut() {
        *t = Text::new(format!("{} / {}", services.selected_tab.title(), sys.name.to_uppercase()));
    }

    let mission_line = if let Some(m) = services.mission {
        format!("Active delivery: {}  (+{})\n", galaxy.systems[m.target_system].name, format_credits(m.reward))
    } else { "Active delivery: none\n".to_string() };

    let body = match services.selected_tab {
        StationServiceTab::Trade => String::new(),
        StationServiceTab::Missions => if let Some(m) = services.mission {
            format!(
                "Active contract\n\nFrom: {}\nTo: {}\nReward: {}\n\n[C] Complete contract in target system\n\n{}",
                galaxy.systems[m.accepted_in].name,
                galaxy.systems[m.target_system].name,
                format_credits(m.reward),
                mission_line
            )
        } else {
            format!(
                "Available contracts are limited right now.\n\n[M] Accept courier mission\nReward: {}\n\n{}",
                format_credits(640),
                mission_line
            )
        },
        StationServiceTab::Repair => format!(
            "Service bay\n\nCurrent hull: {}%\n[R] Full repair to 100%  (4 C per point)\n[F] Quick patch +18%  (-120 C)\n\n{}",
            services.hull,
            mission_line
        ),
    };
    if let Ok(mut t) = text_q.p1().single_mut() { *t = Text::new(body); }
}

fn update_station_trade_ui_system(
    galaxy: Res<GalaxyMap>,
    landing: Res<StationLanding>,
    mut trade_q: ParamSet<(
        Query<(&StationTradePriceText, &mut Text)>,
        Query<(&StationTradeStockText, &mut Text)>,
        Query<(&StationTradeTrendText, &mut Text, &mut TextColor)>,
    )>,
) {
    if !landing.docked { return; }
    let Some(station) = galaxy.systems[galaxy.current_system].stations.first() else { return; };

    for (tag, mut text) in &mut trade_q.p0() {
        if let Some(offer) = station.market.iter().find(|offer| offer.commodity == tag.commodity) {
            *text = Text::new(format!("{} C", offer.price));
        }
    }
    for (tag, mut text) in &mut trade_q.p1() {
        if let Some(offer) = station.market.iter().find(|offer| offer.commodity == tag.commodity) {
            *text = Text::new(format!("{} T", offer.stock));
        }
    }
    for (tag, mut text, mut color) in &mut trade_q.p2() {
        if let Some(offer) = station.market.iter().find(|offer| offer.commodity == tag.commodity) {
            let (label, tint) = format_trend(offer.trend);
            *text = Text::new(label);
            color.0 = tint;
        }
    }
}

fn update_station_button_visuals_system(
    landing: Res<StationLanding>,
    services: Res<StationServices>,
    galaxy: Res<GalaxyMap>,
    mut buy_q: Query<(&StationBuyBtn, &mut BackgroundColor, &mut BorderColor, &Children)>,
    mut undock_q: Query<(&mut BackgroundColor, &mut BorderColor, &Children), (With<StationUndockBtn>, Without<StationBuyBtn>)>,
    mut label_q: Query<&mut TextColor>,
) {
    if !landing.docked { return; }
    let station = galaxy.systems[galaxy.current_system].stations.first();
    for (btn, mut bg, mut border, children) in &mut buy_q {
        let can_buy = services.selected_tab == StationServiceTab::Trade
            && station.and_then(|s| s.market.iter().find(|offer| offer.commodity == btn.commodity)).is_some_and(|offer| {
                offer.stock > 0 && services.credits >= offer.price && services.cargo < PLAYER_CARGO_CAPACITY
            });
        *bg = BackgroundColor(if can_buy { Color::srgba(0.05, 0.11, 0.19, 0.98) } else { Color::srgba(0.03, 0.06, 0.10, 0.96) });
        *border = BorderColor::all(if can_buy { C_BORDER2 } else { C_BORDER });
        for child in children.iter() {
            if let Ok(mut tc) = label_q.get_mut(child) {
                tc.0 = if can_buy { C_TEXT } else { C_MUTED };
            }
        }
    }
    for (mut bg, mut border, children) in &mut undock_q {
        *bg = BackgroundColor(C_NONE);
        *border = BorderColor::all(C_BORDER);
        for child in children.iter() {
            if let Ok(mut tc) = label_q.get_mut(child) {
                tc.0 = C_TEXT;
            }
        }
    }
}

fn station_buy_button_system(
    landing: Res<StationLanding>,
    mut galaxy: ResMut<GalaxyMap>,
    mut services: ResMut<StationServices>,
    btn_q: Query<(&Interaction, &StationBuyBtn), Changed<Interaction>>,
) {
    if !landing.docked || services.selected_tab != StationServiceTab::Trade { return; }
    for (interaction, btn) in &btn_q {
        if *interaction != Interaction::Pressed { continue; }
        buy_commodity(&mut galaxy, &mut services, btn.commodity);
    }
}

fn station_undock_button_system(
    mut landing_state: ResMut<StationLanding>,
    mut targeting: ResMut<TargetingState>,
    btn_q: Query<&Interaction, (Changed<Interaction>, With<StationUndockBtn>)>,
) {
    if !landing_state.docked { return; }
    for interaction in &btn_q {
        if *interaction == Interaction::Pressed {
            landing_state.docked = false;
            targeting.selected_station = None;
            targeting.auto_dock = false;
        }
    }
}

fn station_tab_button_system(
    mut services: ResMut<StationServices>,
    btn_q: Query<(&Interaction, &StationTabBtn), Changed<Interaction>>,
) {
    for (interaction, tab_btn) in &btn_q {
        if *interaction == Interaction::Pressed { services.selected_tab = tab_btn.tab; }
    }
}

fn update_map_ui_system(
    galaxy: Res<GalaxyMap>,
    mut roots: Query<&mut Visibility, With<MapUiRoot>>,
    mut info_q: ParamSet<(
        Query<&mut Text, With<MapInfoTitle>>,
        Query<(&mut Text, &mut TextColor), (With<MapInfoFaction>, Without<MapInfoTitle>)>,
        Query<&mut Text, With<MapInfoPlanet>>,
        Query<&mut Text, With<MapInfoClass>>,
        Query<&mut Text, With<MapInfoStations>>,
        Query<(&mut Text, &mut TextColor), With<MapInfoStatus>>,
        Query<(&mut Text, &mut TextColor), With<MapInfoSecurity>>,
    )>,
) {
    let vis = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in &mut roots { *v = vis; }
    if !galaxy.map_open { return; }
    let sel = &galaxy.systems[galaxy.selected_system];
    if let Ok(mut t) = info_q.p0().single_mut() { *t = Text::new(sel.name.to_lowercase()); }
    if let Ok((mut t, mut tc)) = info_q.p1().single_mut() {
        *t = Text::new(sel.faction.name().to_uppercase()); tc.0 = sel.faction.color();
    }
    if let Ok(mut t) = info_q.p2().single_mut() { *t = Text::new(sel.planet_name); }
    if let Ok(mut t) = info_q.p3().single_mut() { *t = Text::new(sel.planet_class); }
    if let Ok(mut t) = info_q.p4().single_mut() { *t = Text::new(sel.stations.len().to_string()); }
    if let Ok((mut t, mut tc)) = info_q.p5().single_mut() {
        let is_current = galaxy.selected_system == galaxy.current_system;
        *t = Text::new(if is_current { "Current" } else { "Reachable" });
        tc.0 = if is_current { C_SUCCESS } else { C_ACCENT };
    }
    if let Ok((mut t, mut tc)) = info_q.p6().single_mut() {
        *t = Text::new(sel.security);
        tc.0 = if sel.security == "High" { C_SUCCESS } else if sel.security == "Medium" { C_WARN } else { C_DANGER };
    }
}

fn update_map_warp_button_visual_system(
    galaxy: Res<GalaxyMap>,
    mut btn_q: Query<(&mut BackgroundColor, &mut BorderColor, &Children), With<MapWarpBtn>>,
    mut label_q: Query<&mut TextColor, With<MapWarpBtnLabel>>,
) {
    let can_warp = galaxy.map_open && galaxy.selected_system != galaxy.current_system;
    for (mut bg, mut border, children) in &mut btn_q {
        *bg = BackgroundColor(if can_warp { Color::srgba(0.05, 0.12, 0.20, 0.98) } else { C_NONE });
        *border = BorderColor::all(if can_warp { C_BORDER2 } else { C_BORDER });
        for child in children.iter() {
            if let Ok(mut tc) = label_q.get_mut(child) {
                tc.0 = if can_warp { C_TEXT } else { C_MUTED };
            }
        }
    }
}

fn map_warp_button_system(
    mut galaxy: ResMut<GalaxyMap>,
    mut warp: ResMut<WarpDrive>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
    btn_q: Query<&Interaction, (Changed<Interaction>, With<MapWarpBtn>)>,
) {
    if !galaxy.map_open || galaxy.selected_system == galaxy.current_system { return; }
    for interaction in &btn_q {
        if *interaction == Interaction::Pressed {
            initiate_warp(&mut galaxy, &mut warp, &mut player_query);
        }
    }
}

// ── World-space map updates ────────────────────────────────────────────────
fn update_map_panels_system(
    galaxy: Res<GalaxyMap>,
    camera_q: Query<&GlobalTransform, With<MainCamera>>,
    mut map_q: ParamSet<(
        Query<(&mut Transform, &mut Visibility), (With<MapBackdrop>, Without<MapSelection>, Without<MapNode>, Without<MapNodeLabel>, Without<MapPlanetPreview>)>,
        Query<(&mut Transform, &mut Visibility), (With<MapSelection>, Without<MapBackdrop>)>,
        Query<(&mut Sprite, &mut Transform, &mut Visibility), With<MapPlanetPreview>>,
    )>,
    asset_server: Res<AssetServer>,
) {
    let Ok(ct) = camera_q.single() else { return; };
    let cc = ct.translation().truncate();
    let vis = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    for (mut t, mut v) in &mut map_q.p0() { t.translation = (cc + Vec2::new(-150.0, 0.0)).extend(150.0); *v = vis; }
    if let Ok((mut t, mut v)) = map_q.p1().single_mut() {
        let pos = cc + Vec2::new(MAP_LEFT_SHIFT, 0.0) + galaxy.systems[galaxy.selected_system].map_position;
        t.translation = pos.extend(153.0); *v = vis;
    }
    if let Ok((mut sprite, mut t, mut v)) = map_q.p2().single_mut() {
        sprite.image = asset_server.load(galaxy.systems[galaxy.selected_system].planet_sprite);
        t.translation = (cc + Vec2::new(-420.0, 40.0)).extend(MAP_PREVIEW_Z);
        t.scale = Vec3::splat(1.2); *v = vis;
    }
}

fn update_map_nodes_system(
    galaxy: Res<GalaxyMap>,
    camera_q: Query<&GlobalTransform, With<MainCamera>>,
    mut link_q: Query<(&MapLink, &mut Transform, &mut Sprite, &mut Visibility), (Without<MapNode>, Without<MapNodeLabel>)>,
    mut node_q: Query<(&MapNode, &mut Transform, &mut Sprite, &mut Visibility), (Without<MapNodeLabel>, Without<MapLink>)>,
    mut label_q: Query<(&MapNodeLabel, &mut Transform, &mut Visibility), Without<MapNode>>,
) {
    let Ok(ct) = camera_q.single() else { return; };
    let cc = ct.translation().truncate();
    let vis = if galaxy.map_open { Visibility::Inherited } else { Visibility::Hidden };
    for (link, mut t, mut sprite, mut v) in &mut link_q {
        let a = galaxy.systems[link.a].map_position;
        let b = galaxy.systems[link.b].map_position;
        let delta = b - a;
        let center = (a + b) * 0.5;
        t.translation = (cc + Vec2::new(MAP_LEFT_SHIFT, 0.0) + center).extend(151.0);
        t.rotation = Quat::from_rotation_z(delta.y.atan2(delta.x));
        sprite.custom_size = Some(Vec2::new(delta.length().max(1.0), 1.5));
        *v = vis;
    }
    for (node, mut t, mut sprite, mut v) in &mut node_q {
        let sys = &galaxy.systems[node.index];
        t.translation = (cc + Vec2::new(MAP_LEFT_SHIFT, 0.0) + sys.map_position).extend(152.0);
        let is_cur = node.index == galaxy.current_system;
        let is_sel = node.index == galaxy.selected_system;
        sprite.color = if is_cur { C_WARN } else if is_sel { Color::srgb(1.0, 0.78, 0.34) } else { sys.faction.color() };
        sprite.custom_size = Some(Vec2::splat(if is_cur { 20.0 } else if is_sel { 14.0 } else { 10.0 }));
        *v = vis;
    }
    for (label, mut t, mut v) in &mut label_q {
        let sys = &galaxy.systems[label.index];
        t.translation = (cc + Vec2::new(MAP_LEFT_SHIFT, 0.0) + sys.map_position + Vec2::new(0.0, -14.0)).extend(154.0);
        *v = vis;
    }
}

// ── Original gameplay systems (100% unchanged logic) ───────────────────────
fn spawn_starfield(commands: &mut Commands, galaxy: &GalaxyMap) {
    let mut rng = StdRng::seed_from_u64(42);
    let cs = &galaxy.systems[galaxy.current_system];
    for _ in 0..STAR_COUNT {
        let x = rng.gen_range(-STAR_FIELD_RADIUS..STAR_FIELD_RADIUS);
        let y = rng.gen_range(-STAR_FIELD_RADIUS..STAR_FIELD_RADIUS);
        let size = rng.gen_range(1.0..3.8);
        let brightness = rng.gen_range(0.45..1.0);
        commands.spawn((
            Sprite::from_color(tint_star(cs.star_color, brightness), Vec2::splat(size)),
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
    landing: Res<StationLanding>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
    mut preview_query: Query<&mut Sprite, (With<MapPlanetPreview>, Without<StarfieldStar>)>,
) {
    if landing.docked { galaxy.map_open = false; return; }
    if warp.active { galaxy.map_open = false; return; }
    if keyboard.just_pressed(KeyCode::Tab) { galaxy.map_open = !galaxy.map_open; }
    if !galaxy.map_open { return; }
    if keyboard.just_pressed(KeyCode::Escape) { galaxy.map_open = false; return; }

    let Ok(cc) = camera_query.single().map(|(_, ct)| ct.translation().truncate()) else { return; };

    if keyboard.just_pressed(KeyCode::Enter) && galaxy.selected_system != galaxy.current_system {
        initiate_warp(&mut galaxy, &mut warp, &mut player_query);
        return;
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let Ok(window) = primary_window.single() else { return; };
        let Ok((camera, ct)) = camera_query.single() else { return; };
        let Some(cursor) = window.cursor_position() else { return; };
        let Ok(cw) = camera.viewport_to_world_2d(ct, cursor) else { return; };
        let mut closest = None;
        let mut closest_dist = 40.0_f32;
        for (i, sys) in galaxy.systems.iter().enumerate() {
            let node_pos = cc + Vec2::new(MAP_LEFT_SHIFT, 0.0) + sys.map_position;
            let d = cw.distance(node_pos);
            if d < closest_dist { closest_dist = d; closest = Some(i); }
        }
        if let Some(idx) = closest {
            if galaxy.selected_system == idx && idx != galaxy.current_system {
                initiate_warp(&mut galaxy, &mut warp, &mut player_query);
            } else {
                galaxy.selected_system = idx;
                if let Ok(mut sprite) = preview_query.single_mut() {
                    sprite.image = asset_server.load(galaxy.systems[idx].planet_sprite);
                }
            }
        }
    }
}

fn initiate_warp(
    galaxy: &mut ResMut<GalaxyMap>,
    warp: &mut ResMut<WarpDrive>,
    player_query: &mut Query<(&mut Transform, &mut Velocity, &mut ThrusterState), (With<Player>, Without<MainCamera>)>,
) {
    let Ok((player_transform, _, _)) = player_query.single_mut() else { return; };
    let departure = player_transform.translation.truncate();
    let target_idx = galaxy.selected_system;
    let arrival = warp_arrival_point(&galaxy.systems[target_idx], target_idx);
    let direction = (arrival - departure).normalize_or_zero();
    warp.active = true; warp.phase = WarpPhase::Align; warp.target_system = target_idx;
    warp.departure_origin = departure; warp.travel_direction = direction; warp.arrival_point = arrival;
    galaxy.map_open = false;
}

fn station_target_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    landing: Res<StationLanding>,
    mut targeting: ResMut<TargetingState>,
    btn_q: Query<(&Interaction, &TargetActionBtn), Changed<Interaction>>,
) {
    if warp.active || landing.docked || galaxy.map_open { return; }
    if keyboard.just_pressed(KeyCode::KeyQ) { targeting.selected_station = None; targeting.auto_dock = false; return; }
    for (interaction, tab) in &btn_q {
        if *interaction == Interaction::Pressed {
            match tab.action {
                TargetAction::Approach => { targeting.auto_dock = false; }
                TargetAction::Dock => { targeting.auto_dock = true; }
                TargetAction::Clear => { targeting.selected_station = None; targeting.auto_dock = false; }
            }
        }
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let Ok(window) = primary_window.single() else { return; };
        let Ok((camera, ct)) = camera_query.single() else { return; };
        let Some(cursor) = window.cursor_position() else { return; };
        let Ok(cw) = camera.viewport_to_world_2d(ct, cursor) else { return; };
        let cs = &galaxy.systems[galaxy.current_system];
        for (slot, station) in cs.stations.iter().enumerate() {
            if cw.distance(cs.planet_position + station.offset) <= TARGET_SELECT_RADIUS {
                targeting.selected_station = Some(slot);
                break;
            }
        }
    }
}

fn station_auto_dock_system(
    time: Res<Time>,
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    landing: Res<StationLanding>,
    targeting: Res<TargetingState>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
) {
    if !targeting.auto_dock || warp.active || landing.docked { return; }
    let Ok((mut t, mut v, mut ts)) = player_query.single_mut() else { return; };
    let Some(slot) = targeting.selected_station else { return; };
    let Some(station) = galaxy.systems[galaxy.current_system].stations.get(slot) else { return; };
    let station_pos = galaxy.systems[galaxy.current_system].planet_position + station.offset;
    let ship_pos = t.translation.truncate();
    let to = station_pos - ship_pos;
    let dist = to.length();
    if dist < 1.0 { return; }
    let dir = to / dist;
    let desired_speed = (dist * 2.0).min(AUTO_DOCK_SPEED);
    v.0 = v.0.lerp(dir * desired_speed, 4.0 * time.delta_secs());
    t.translation.x += v.0.x * time.delta_secs();
    t.translation.y += v.0.y * time.delta_secs();
    let desired_angle = dir.y.atan2(dir.x);
    let current_angle = t.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    let delta = shortest_angle_delta(current_angle, desired_angle);
    let step = TURN_SPEED * time.delta_secs();
    t.rotation = Quat::from_rotation_z(current_angle + delta.clamp(-step, step) - std::f32::consts::FRAC_PI_2);
    ts.thrusting = true;
    ts.intensity = (ts.intensity + THRUST_RAMP_UP * time.delta_secs()).clamp(0.0, 1.0);
}

fn station_landing_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    mut landing: ResMut<StationLanding>,
    mut targeting: ResMut<TargetingState>,
    mut services: ResMut<StationServices>,
    player_query: Query<&Transform, With<Player>>,
) {
    if warp.active { return; }
    if landing.docked {
        if keyboard.just_pressed(KeyCode::KeyE) { landing.docked = false; targeting.selected_station = None; targeting.auto_dock = false; }
        return;
    }
    let Ok(pt) = player_query.single() else { return; };
    let ship_pos = pt.translation.truncate();
    let cs = &galaxy.systems[galaxy.current_system];
    if let Some(station) = cs.stations.first() {
        if ship_pos.distance(cs.planet_position + station.offset) <= STATION_DOCK_RADIUS && keyboard.just_pressed(KeyCode::KeyE) {
            landing.docked = true;
            if let Some(m) = services.mission {
                if m.target_system == galaxy.current_system { services.credits += m.reward; services.mission = None; }
            }
        }
    }
}

fn station_services_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    landing: Res<StationLanding>,
    mut galaxy_mut: ResMut<GalaxyMap>,
    mut services: ResMut<StationServices>,
) {
    if !landing.docked { return; }
    if keyboard.just_pressed(KeyCode::Digit1) { services.selected_tab = StationServiceTab::Trade; }
    if keyboard.just_pressed(KeyCode::Digit2) { services.selected_tab = StationServiceTab::Missions; }
    if keyboard.just_pressed(KeyCode::Digit3) { services.selected_tab = StationServiceTab::Repair; }
    match services.selected_tab {
        StationServiceTab::Trade => {
            if keyboard.just_pressed(KeyCode::KeyT) {
                buy_commodity(&mut galaxy_mut, &mut services, CommodityId::Ore);
            }
        }
        StationServiceTab::Missions => {
            if keyboard.just_pressed(KeyCode::KeyM) && services.mission.is_none() {
                let next = (galaxy_mut.current_system + 1) % galaxy_mut.systems.len();
                if next != galaxy_mut.current_system {
                    services.mission = Some(StationMission { target_system: next, reward: 640, accepted_in: galaxy_mut.current_system });
                }
            }
            if keyboard.just_pressed(KeyCode::KeyC) {
                if let Some(m) = services.mission {
                    if m.target_system == galaxy_mut.current_system { services.credits += m.reward; services.mission = None; }
                }
            }
        }
        StationServiceTab::Repair => {
            if keyboard.just_pressed(KeyCode::KeyR) {
                let missing = 100 - services.hull;
                let cost = missing * 4;
                if services.credits >= cost { services.credits -= cost; services.hull = 100; }
            }
            if keyboard.just_pressed(KeyCode::KeyF) && services.credits >= 120 {
                services.credits -= 120; services.hull = (services.hull + 18).min(100);
            }
        }
    }
}

fn player_input_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    time: Res<Time>,
    galaxy: Res<GalaxyMap>,
    warp: Res<WarpDrive>,
    landing: Res<StationLanding>,
    targeting: Res<TargetingState>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
) {
    let Ok((mut t, mut v, mut ts)) = query.single_mut() else { return; };
    if warp.active || targeting.auto_dock { return; }
    if landing.docked { ts.thrusting = false; ts.intensity = 0.0; v.0 = Vec2::ZERO; return; }
    if galaxy.map_open {
        ts.thrusting = false;
        ts.intensity = (ts.intensity - THRUST_RAMP_DOWN * time.delta_secs()).max(0.0);
        v.0 *= SHIP_DRAG;
        t.translation.x += v.0.x * time.delta_secs();
        t.translation.y += v.0.y * time.delta_secs();
        return;
    }
    let Ok(window) = primary_window.single() else { return; };
    let Ok((camera, ct)) = camera_query.single() else { return; };
    let ship_pos = t.translation.truncate();
    let current_angle = t.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    if let Some(cursor) = window.cursor_position() {
        if let Ok(cw) = camera.viewport_to_world_2d(ct, cursor) {
            let to = cw - ship_pos;
            if to.length_squared() > 1.0 {
                let desired = to.y.atan2(to.x);
                let delta = shortest_angle_delta(current_angle, desired);
                let step = TURN_SPEED * time.delta_secs();
                t.rotation = Quat::from_rotation_z(current_angle + delta.clamp(-step, step) - std::f32::consts::FRAC_PI_2);
            }
        }
    }
    let wants = mouse_buttons.pressed(MouseButton::Left);
    let ramp = if wants { THRUST_RAMP_UP } else { THRUST_RAMP_DOWN };
    ts.intensity += (if wants { 1.0 } else { 0.0 } - ts.intensity) * ramp * time.delta_secs();
    ts.intensity = ts.intensity.clamp(0.0, 1.0);
    ts.thrusting = ts.intensity > 0.03;
    if ts.thrusting {
        let facing = Vec2::from_angle(t.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2);
        v.0 += facing * SHIP_ACCELERATION * ts.intensity * time.delta_secs();
    }
    v.0 *= SHIP_DRAG;
    if v.0.length() > SHIP_MAX_SPEED { v.0 = v.0.normalize() * SHIP_MAX_SPEED; }
    t.translation.x += v.0.x * time.delta_secs();
    t.translation.y += v.0.y * time.delta_secs();
}

fn warp_travel_system(
    time: Res<Time>,
    mut clear_color: ResMut<ClearColor>,
    mut galaxy: ResMut<GalaxyMap>,
    mut warp: ResMut<WarpDrive>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut ThrusterState), With<Player>>,
    mut star_query: Query<(&StarfieldStar, &mut Sprite), Without<MapPlanetPreview>>,
) {
    if !warp.active { return; }
    let Ok((mut t, mut v, mut ts)) = player_query.single_mut() else { return; };
    let dt = time.delta_secs();
    let ship_pos = t.translation.truncate();
    let current_angle = t.rotation.to_euler(EulerRot::XYZ).2 + std::f32::consts::FRAC_PI_2;
    let target_dir = match warp.phase {
        WarpPhase::Align | WarpPhase::Depart | WarpPhase::Cruise => warp.travel_direction,
        WarpPhase::Arrive => (warp.arrival_point - ship_pos).normalize_or_zero(),
        WarpPhase::Idle => Vec2::ZERO,
    };
    if target_dir.length_squared() > 0.0 {
        let desired = target_dir.y.atan2(target_dir.x);
        let delta = shortest_angle_delta(current_angle, desired);
        let step = TURN_SPEED * 1.85 * dt;
        t.rotation = Quat::from_rotation_z(current_angle + delta.clamp(-step, step) - std::f32::consts::FRAC_PI_2);
    }
    match warp.phase {
        WarpPhase::Align => {
            ts.thrusting = false;
            ts.intensity = (ts.intensity - THRUST_RAMP_DOWN * dt).max(0.0);
            v.0 *= 0.97;
            if shortest_angle_delta(current_angle, warp.travel_direction.y.atan2(warp.travel_direction.x)).abs() <= WARP_ALIGNMENT_EPSILON {
                warp.phase = WarpPhase::Depart;
            }
        }
        WarpPhase::Depart => {
            ts.thrusting = true;
            ts.intensity = (ts.intensity + THRUST_RAMP_UP * dt).clamp(0.0, 1.0);
            v.0 += warp.travel_direction * WARP_ACCELERATION * dt;
            v.0 = v.0.clamp_length_max(WARP_MAX_SPEED);
            let np = ship_pos + v.0 * dt;
            t.translation.x = np.x; t.translation.y = np.y;
            if np.distance(warp.departure_origin) >= WARP_DEPART_DISTANCE && v.0.length() >= WARP_ENTRY_SPEED {
                galaxy.current_system = warp.target_system;
                apply_system_palette(&galaxy.systems[warp.target_system], &mut clear_color, &mut star_query);
                let entry = warp.arrival_point - warp.travel_direction * WARP_ARRIVAL_DISTANCE;
                t.translation.x = entry.x; t.translation.y = entry.y;
                v.0 = warp.travel_direction * WARP_MAX_SPEED;
                warp.phase = WarpPhase::Cruise;
            }
        }
        WarpPhase::Cruise => {
            ts.thrusting = true;
            ts.intensity = (ts.intensity + THRUST_RAMP_UP * 0.55 * dt).clamp(0.0, 1.0);
            v.0 = v.0.lerp(warp.travel_direction * WARP_MAX_SPEED, 1.6 * dt);
            let np = ship_pos + v.0 * dt;
            t.translation.x = np.x; t.translation.y = np.y;
            let remaining = warp.arrival_point.distance(np);
            let brake = (v.0.length().powi(2) / (2.0 * WARP_BRAKE_ACCELERATION)).max(WARP_ARRIVAL_RADIUS);
            if remaining <= brake { warp.phase = WarpPhase::Arrive; }
        }
        WarpPhase::Arrive => {
            let to = warp.arrival_point - ship_pos;
            let dist = to.length();
            let dir = to.normalize_or_zero();
            let braking = (2.0 * WARP_BRAKE_ACCELERATION * dist).sqrt();
            let af = (dist / 1800.0).clamp(0.0, 1.0);
            let desired_speed = (braking.min(WARP_MAX_SPEED) * af).max(if dist > 180.0 { 70.0 } else { 0.0 });
            v.0 = v.0.lerp(dir * desired_speed, (WARP_GUIDANCE * dt).clamp(0.0, 1.0));
            ts.thrusting = v.0.length() > 40.0 || dist > WARP_FINISH_RADIUS;
            let di = (dist / 2500.0).clamp(0.12, 0.72);
            ts.intensity += (di - ts.intensity) * THRUST_RAMP_UP * 0.7 * dt;
            ts.intensity = ts.intensity.clamp(0.0, 1.0);
            let np = ship_pos + v.0 * dt;
            t.translation.x = np.x; t.translation.y = np.y;
            if dist <= WARP_FINISH_RADIUS && v.0.length() <= 28.0 {
                t.translation.x = warp.arrival_point.x; t.translation.y = warp.arrival_point.y;
                v.0 = Vec2::ZERO; ts.thrusting = false; ts.intensity = 0.0;
                warp.active = false; warp.phase = WarpPhase::Idle;
            }
        }
        WarpPhase::Idle => {}
    }
}

fn engine_flame_system(
    player_query: Query<(&Transform, &ThrusterState), With<Player>>,
    mut flame_query: Query<(&EngineFlame, &mut Transform, &mut Sprite, &mut Visibility), Without<Player>>,
) {
    let Ok((pt, ts)) = player_query.single() else { return; };
    let rot = pt.rotation.to_euler(EulerRot::XYZ).2;
    let rm = Mat2::from_angle(rot);
    let fl = FLAME_BASE_LENGTH + (FLAME_MAX_LENGTH - FLAME_BASE_LENGTH) * ts.intensity;
    for (flame, mut t, mut sprite, mut v) in &mut flame_query {
        if ts.thrusting {
            *v = Visibility::Inherited;
            let lo = Vec2::new(flame.side * ENGINE_OFFSET_X * SHIP_SCALE, ENGINE_OFFSET_Y * SHIP_SCALE);
            let wo = rm * lo;
            t.translation.x = pt.translation.x + wo.x;
            t.translation.y = pt.translation.y + wo.y;
            t.translation.z = pt.translation.z - 1.0;
            t.rotation = pt.rotation;
            sprite.custom_size = Some(Vec2::new(8.0, fl));
            sprite.color = Color::srgba(0.45 + 0.25 * ts.intensity, 0.82 + 0.10 * ts.intensity, 1.0, 0.72);
        } else { *v = Visibility::Hidden; }
    }
}

fn camera_follow_system(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(pt) = player_query.single() else { return; };
    let Ok(mut ct) = camera_query.single_mut() else { return; };
    let next = ct.translation.lerp(Vec3::new(pt.translation.x, pt.translation.y, ct.translation.z), CAMERA_LERP * time.delta_secs());
    ct.translation = next;
}

fn update_local_system_visuals_system(
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    galaxy: Res<GalaxyMap>,
    mut planet_query: Query<(&mut Sprite, &mut Transform), With<CurrentSystemPlanet>>,
    mut station_query: Query<(&CurrentSystemStation, &mut Sprite, &mut Transform, &mut Visibility), Without<CurrentSystemPlanet>>,
) {
    let sys = &galaxy.systems[galaxy.current_system];
    if let Ok((mut sprite, mut t)) = planet_query.single_mut() {
        sprite.image = asset_server.load(sys.planet_sprite);
        t.translation = sys.planet_position.extend(PLANET_RENDER_Z);
        t.scale = Vec3::splat(sys.planet_scale);
    }
    for (slot, mut sprite, mut t, mut v) in &mut station_query {
        if slot.slot == 0 {
            if let Some(station) = sys.stations.first() {
                sprite.image = asset_server.load(station.sprite);
                sprite.custom_size = station_uniform_size(&sprite.image, &images);
                t.scale = Vec3::ONE;
                t.translation = (sys.planet_position + station.offset).extend(STATION_RENDER_Z);
                t.rotation = Quat::IDENTITY;
                *v = Visibility::Inherited;
            } else { *v = Visibility::Hidden; }
        } else { *v = Visibility::Hidden; }
    }
}

// ── Utility ────────────────────────────────────────────────────────────────
fn tint_star(star_color: Color, brightness: f32) -> Color {
    let s = star_color.to_srgba();
    Color::srgba((s.red * brightness).clamp(0.0, 1.0), (s.green * brightness).clamp(0.0, 1.0), (s.blue * brightness).clamp(0.0, 1.0), 0.45 + brightness * 0.5)
}

fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    let mut d = target - current;
    while d > std::f32::consts::PI { d -= std::f32::consts::TAU; }
    while d < -std::f32::consts::PI { d += std::f32::consts::TAU; }
    d
}

fn station_uniform_size(image_handle: &Handle<Image>, images: &Assets<Image>) -> Option<Vec2> {
    if let Some(image) = images.get(image_handle) {
        let size = image.size();
        let w = size.x as f32; let h = size.y as f32;
        if w > 0.0 && h > 0.0 { let s = STATION_UNIFORM_SIZE / w.max(h); return Some(Vec2::new(w * s, h * s)); }
    }
    Some(Vec2::splat(STATION_UNIFORM_SIZE))
}

fn buy_commodity(galaxy: &mut GalaxyMap, services: &mut StationServices, commodity: CommodityId) {
    let Some(station) = galaxy.systems[galaxy.current_system].stations.first_mut() else { return; };
    let Some(offer) = station.market.iter_mut().find(|offer| offer.commodity == commodity) else { return; };
    if offer.stock <= 0 || services.cargo >= PLAYER_CARGO_CAPACITY || services.credits < offer.price { return; }
    offer.stock -= 1;
    services.credits -= offer.price;
    services.cargo += 1;
}

fn format_credits(amount: i32) -> String {
    let digits = amount.abs().to_string();
    let mut grouped = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(' ');
        }
        grouped.push(ch);
    }
    let mut result: String = grouped.chars().rev().collect();
    if amount < 0 {
        result.insert(0, '-');
    }
    result.push_str(" C");
    result
}

fn format_trend(trend: i32) -> (String, Color) {
    if trend > 0 {
        (format!("▲ +{}%", trend), C_SUCCESS)
    } else if trend < 0 {
        (format!("▼ {}%", trend), C_DANGER)
    } else {
        ("─ 0%".to_string(), C_MUTED)
    }
}

fn station_market(ore: (i32, i32, i32), fuel: (i32, i32, i32), tech: (i32, i32, i32), food: (i32, i32, i32)) -> Vec<MarketOffer> {
    vec![
        MarketOffer { commodity: CommodityId::Ore, price: ore.0, stock: ore.1, trend: ore.2 },
        MarketOffer { commodity: CommodityId::Fuel, price: fuel.0, stock: fuel.1, trend: fuel.2 },
        MarketOffer { commodity: CommodityId::Tech, price: tech.0, stock: tech.1, trend: tech.2 },
        MarketOffer { commodity: CommodityId::Food, price: food.0, stock: food.1, trend: food.2 },
    ]
}

fn warp_arrival_point(system: &SystemDefinition, system_index: usize) -> Vec2 {
    let sp = system.stations.first().map(|s| system.planet_position + s.offset).unwrap_or(system.planet_position + Vec2::new(220.0, 0.0));
    sp + Vec2::from_angle(0.85 + system_index as f32 * 0.9) * 520.0
}

fn apply_system_palette(system: &SystemDefinition, clear_color: &mut ClearColor, star_query: &mut Query<(&StarfieldStar, &mut Sprite), Without<MapPlanetPreview>>) {
    clear_color.0 = system.sky_color;
    for (star, mut sprite) in star_query.iter_mut() {
        sprite.color = tint_star(system.star_color, star.brightness);
        sprite.custom_size = Some(Vec2::splat(star.size));
    }
}

// ── Galaxy data ────────────────────────────────────────────────────────────
fn authored_galaxy() -> GalaxyMap {
    GalaxyMap {
        systems: vec![
            SystemDefinition { name: "Hearthlight", faction: Faction::Helios, map_position: Vec2::new(-250.0, 120.0), star_color: Color::srgb(1.0, 0.82, 0.52), sky_color: Color::srgb(0.02, 0.02, 0.05), planet_sprite: "planet/spr_planet02.png", planet_name: "Hearth", planet_class: "Class M", security: "High", planet_position: Vec2::new(760.0, 240.0), planet_scale: 1.9, stations: vec![StationDefinition { name: "Hearth Port", owner: Faction::Helios, sprite: "stations/SS1.png", offset: Vec2::new(150.0, -24.0), market: station_market((120, 480, 3), (85, 220, -1), (340, 60, 8), (55, 900, 0)) }] },
            SystemDefinition { name: "Lumen Crossing", faction: Faction::Helios, map_position: Vec2::new(-90.0, -20.0), star_color: Color::srgb(1.0, 0.92, 0.68), sky_color: Color::srgb(0.02, 0.025, 0.055), planet_sprite: "planet/planet18.png", planet_name: "Lumen", planet_class: "Class K", security: "High", planet_position: Vec2::new(840.0, -120.0), planet_scale: 2.2, stations: vec![StationDefinition { name: "Iris Anchor", owner: Faction::Helios, sprite: "stations/WB_baseu2_d0.png", offset: Vec2::new(170.0, 42.0), market: station_market((112, 390, 1), (78, 260, 4), (365, 48, 10), (62, 780, -2)) }] },
            SystemDefinition { name: "Gray Expanse", faction: Faction::Helios, map_position: Vec2::new(-20.0, 205.0), star_color: Color::srgb(0.86, 0.93, 1.0), sky_color: Color::srgb(0.015, 0.02, 0.045), planet_sprite: "planet/spr_planet05.png", planet_name: "Gray", planet_class: "Class N", security: "Medium", planet_position: Vec2::new(700.0, 310.0), planet_scale: 1.7, stations: vec![StationDefinition { name: "Pillar Rest", owner: Faction::Helios, sprite: "stations/WB_base_d0.png", offset: Vec2::new(135.0, 82.0), market: station_market((134, 320, 6), (94, 180, -3), (298, 94, 2), (52, 1020, 1)) }] },
            SystemDefinition { name: "Brimhold", faction: Faction::Vanta, map_position: Vec2::new(120.0, 140.0), star_color: Color::srgb(0.74, 0.86, 1.0), sky_color: Color::srgb(0.015, 0.03, 0.055), planet_sprite: "planet/planet24.png", planet_name: "Brim", planet_class: "Class D", security: "Medium", planet_position: Vec2::new(880.0, 190.0), planet_scale: 2.0, stations: vec![StationDefinition { name: "Black Quarry", owner: Faction::Vanta, sprite: "stations/starbase-tex.png", offset: Vec2::new(155.0, -58.0), market: station_market((104, 620, -4), (98, 140, 7), (392, 44, 12), (48, 760, -1)) }] },
            SystemDefinition { name: "Dusk Meridian", faction: Faction::Vanta, map_position: Vec2::new(210.0, -95.0), star_color: Color::srgb(0.62, 0.78, 1.0), sky_color: Color::srgb(0.012, 0.022, 0.05), planet_sprite: "planet/planet31.png", planet_name: "Dusk", planet_class: "Class M", security: "Low", planet_position: Vec2::new(820.0, -250.0), planet_scale: 2.3, stations: vec![StationDefinition { name: "Ashen Ring", owner: Faction::Vanta, sprite: "stations/WB_baseu2_d0.png", offset: Vec2::new(185.0, 32.0), market: station_market((127, 350, 5), (73, 300, -6), (358, 58, 9), (59, 640, 3)) }] },
        ],
        links: vec![(0, 1), (1, 2), (1, 3), (3, 4)],
        current_system: 0, selected_system: 0, map_open: false,
    }
}
