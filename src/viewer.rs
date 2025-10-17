use std::f32::consts::{FRAC_PI_4, FRAC_PI_6};

use bevy::math::prelude::EulerRot;
use bevy::prelude::*;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::render::{render_resource::{WgpuFeatures}, settings::{RenderCreation, WgpuSettings}, RenderPlugin};
use bevy::render::camera::{PerspectiveProjection, Projection};
use hexx::{Vec2 as HVec2, conversions::OffsetHexMode, HexOrientation};
use map::{Map, Terrain};

const WINDOW_WIDTH: f32 = 1400.0;
const WINDOW_HEIGHT: f32 = 900.0;
const MODEL_DIAMETER_M: f32 = 1.1547; // measured vertex-to-vertex diameter

#[derive(Resource, Clone)]
struct MapRes(Map);

#[derive(Resource, Clone, Copy)]
struct TerrainSeed(pub u64);


pub fn run_gui(map: Map, seed: u64) {
    let title = format!("Civorum – {} map", map.size());

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title,
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(RenderPlugin {
                    // Enable wireframe rendering (native only; requires POLYGON_MODE_LINE)
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            WireframePlugin::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.06, 0.08, 0.12)))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::BLACK.into(),
        })
        .insert_resource(MapRes(map))
        .insert_resource(TerrainSeed(seed))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera::orbit_camera_controls, toggle_wireframe, update_hover_ui))
        .run();
}

fn setup(
    mut commands: Commands,
    // 3D assets are glTF scenes; no procedural meshes/materials needed here
    asset_server: Res<AssetServer>,
    map: Res<MapRes>,
    seed: Res<TerrainSeed>,
) {
    let map = &map.0;
    let layout = map.layout();
    let tiles = map.tiles().to_vec();
    // Compute world centers and bounds
    let centers: Vec<HVec2> = tiles.iter().map(|&h| layout.hex_to_world_pos(h)).collect();
    let Some((min, max)) = bounding_box(&centers) else {
        commands.spawn((Camera3d::default(), Msaa::Sample4, Transform::default()));
        return;
    };
    let center = (min + max) * 0.5;
    let span = max - min;
    let rect = layout.rect_size();
    println!(
        "viewer: tiles={} rect=({:.2},{:.2}) bounds min=({:.2},{:.2}) max=({:.2},{:.2}) center=({:.2},{:.2}) span=({:.2},{:.2})",
        tiles.len(), rect.x, rect.y, min.x, min.y, max.x, max.y, center.x, center.y, span.x, span.y
    );

    // Load terrain models (glb scenes) and compute scale to fit hex diameter
    let models = TerrainModels {
        water: asset_server.load("models/water.glb#Scene0"),
        water_deep: asset_server.load("models/deep-water.glb#Scene0"),
        stone_mountain: asset_server.load("models/stone-mountain.glb#Scene0"),
        sand_desert: asset_server.load("models/sand-desert.glb#Scene0"),
        sand: asset_server.load("models/sand.glb#Scene0"),
        grass_hill: asset_server.load("models/grass-hill.glb#Scene0"),
        grass_forest: asset_server.load("models/grass-forest.glb#Scene0"),
        grass: asset_server.load("models/grass.glb#Scene0"),
        snow: asset_server.load("models/snow.glb#Scene0"),
    };
    let scale = map.scale_for_model_diameter(MODEL_DIAMETER_M);

    let seed = seed.0;
    for (i, cell) in map.cells().iter().enumerate() {
        let hex = *cell.hex();
        let pos = layout.hex_to_world_pos(hex);
        let scene = handle_for_terrain(&models, map, cell.terrain(), hex, seed).clone();
        // Rotate pointy-top assets by 30° around Y to match our flat-top layout
        let transform = Transform::from_xyz(pos.x, 0.0, pos.y)
            .with_rotation(Quat::from_rotation_y(FRAC_PI_6))
            .with_scale(Vec3::splat(scale));

        let entity = commands
            .spawn((SceneRoot(scene), transform, Name::new(format!("hex-{i}"))))
            .id();
        if i < 5 {
            println!(
                "viewer: tile[{i}] hex=({},{}) world=({:.2},{:.2}) terrain={:?} entity={:?}",
                hex.x(), hex.y(), pos.x, pos.y, cell.terrain(), entity
            );
        }
    }

    // Hover UI text
    let ui_entity = commands
        .spawn((
            Text::new("Hover a tile..."),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..Default::default()
            },
            HoverUi,
        ))
        .id();
    commands.insert_resource(HoverText(ui_entity));

    // Light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: 12_000.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, FRAC_PI_4, FRAC_PI_4, 0.0)),
    ));

    let base = rect.x.max(rect.y);
    let max_extent = span.length().max(base * 4.0);
    let distance = (max_extent * 1.2).max(base * 10.0);

    let camera = camera::OrbitCamera {
        target: Vec3::new(center.x, 0.0, center.y),
        distance,
        min_distance: base,
        max_distance: distance * 4.0,
        yaw: 0.0,
        // Negative pitch puts the camera above the plane looking down
        pitch: -FRAC_PI_4,
    };

    let transform = camera::transform_from(&camera);

    // Increase far plane to ensure distant grids are visible
    let projection = Projection::Perspective(PerspectiveProjection { far: 10_000.0, ..Default::default() });
    println!(
        "viewer: camera target=({:.2},{:.2},{:.2}) distance={:.2} yaw={:.2} pitch={:.2} far={:.1} pos=({:.2},{:.2},{:.2})",
        camera.target.x,
        camera.target.y,
        camera.target.z,
        camera.distance,
        camera.yaw,
        camera.pitch,
        10_000.0,
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
    );

    let cam_ent = commands
        .spawn((Camera3d::default(), Msaa::Sample4, transform, camera, projection))
        .id();
    println!("viewer: spawned camera entity {:?}", cam_ent);
}

use crate::camera;

fn bounding_box(points: &[HVec2]) -> Option<(HVec2, HVec2)> {
    let mut iter = points.iter();
    let first = *iter.next()?;
    let mut min = first;
    let mut max = first;

    for &p in iter {
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
    }
    Some((min, max))
}

// No longer fitting to window; layout controls hex scale. Camera centers on bounds.

fn pick_index(hex: hexx::Hex, seed: u64, count: usize) -> usize {
    let mut z = seed
        ^ ((hex.x() as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        ^ ((hex.y() as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    let v = z ^ (z >> 31);
    (v as usize) % count.max(1)
}

#[derive(Clone)]
struct TerrainModels {
    water: Handle<Scene>,
    water_deep: Handle<Scene>,
    stone_mountain: Handle<Scene>,
    sand_desert: Handle<Scene>,
    sand: Handle<Scene>,
    grass_hill: Handle<Scene>,
    grass_forest: Handle<Scene>,
    grass: Handle<Scene>,
    snow: Handle<Scene>,
}

fn handle_for_terrain<'a>(models: &'a TerrainModels, map: &Map, t: &Terrain, hex: hexx::Hex, seed: u64) -> &'a Handle<Scene> {
    match t {
        Terrain::Water => {
            if is_deep_water(map, hex) { &models.water_deep } else { &models.water }
        }
        Terrain::Mountain => &models.stone_mountain,
        Terrain::Desert => &models.sand,
        Terrain::Forest => &models.grass_forest,
        Terrain::Grass => &models.grass,
        Terrain::Snow => &models.snow,
    }
}

fn is_deep_water(map: &Map, hex: hexx::Hex) -> bool {
    let mut count = 0;
    for n in map.neighbors(hex) {
        count += 1;
        if let Some(idx) = map.axial_to_index(n) {
            if map.cells()[idx].terrain() != &Terrain::Water {
                return false;
            }
        } else {
            return false;
        }
    }
    count == 6
}
#[derive(Resource, Clone, Copy)]
struct HoverText(Entity);

#[derive(Component)]
struct HoverUi;

fn toggle_wireframe(keys: Res<ButtonInput<KeyCode>>, mut cfg: ResMut<WireframeConfig>) {
    if keys.just_pressed(KeyCode::Space) {
        cfg.global = !cfg.global;
    }
}

fn update_hover_ui(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    map: Res<MapRes>,
    mut layout_cache: Local<Option<hexx::HexLayout>>, // cache to avoid recompute
    mut qtext: Query<&mut Text, With<HoverUi>>,
) {
    let window = match windows.get_single() { Ok(w) => w, Err(_) => return };
    let (camera, cam_xform) = match cameras.get_single() { Ok(v) => v, Err(_) => return };
    let cursor = match window.cursor_position() { Some(p) => p, None => return };
    let Ok(ray) = camera.viewport_to_world(cam_xform, cursor) else { return };
    if ray.direction.y.abs() < 1e-5 { return; }
    let t = -ray.origin.y / ray.direction.y;
    if t.is_nan() || t.is_infinite() { return; }
    let world = ray.origin + ray.direction * t;

    let mapref = &map.0;
    let layout = layout_cache.get_or_insert_with(|| mapref.layout());
    let hex = layout.world_pos_to_hex(HVec2::new(world.x, world.z));
    let [col, row] = hex.to_offset_coordinates(OffsetHexMode::Odd, HexOrientation::Flat);
    if let Some(idx) = mapref.axial_to_index(hex) {
        let tile = &mapref.cells()[idx];
        if let Ok(mut text) = qtext.get_single_mut() {
            *text = Text::new(format!(
                "Tile col={}, row={} | elev={:.2} temp={:.2} rain={:.2} | {:?}",
                col, row, tile.elevation(), tile.temperature(), tile.rainfall(), tile.terrain()
            ));
        }
    }
}
