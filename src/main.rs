use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::prelude::EulerRot;
use bevy::math::primitives::{Extrusion, RegularPolygon};
use bevy::prelude::*;
use map::{Hex, Map, MapSize, SIZE};

const WINDOW_WIDTH: f32 = 1400.0;
const WINDOW_HEIGHT: f32 = 900.0;
const WINDOW_MARGIN: f32 = 0.9;
// Nudge to reduce subpixel cracks between separate hex meshes during rasterization.
const TILE_PACKING: f32 = 0.9985;

struct CliOptions {
    gui: bool,
    size: MapSize,
}

#[derive(Resource, Clone)]
struct MapResource(Map);

#[derive(Component)]
struct OrbitCamera {
    target: Vec3,
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    yaw: f32,
    pitch: f32,
}

fn main() {
    let options = parse_cli().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });

    let map = Map::new(options.size);

    print_map(&map);

    if options.gui {
        run_gui(map);
    }
}

fn parse_cli() -> Result<CliOptions, String> {
    let mut gui = false;
    let mut size: Option<MapSize> = None;

    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--gui" | "-g" => gui = true,
            "--size" | "-s" => {
                let value = args.next().ok_or_else(|| {
                    format!(
                        "Expected a map size after '{}'. Available options: {}.",
                        arg,
                        MapSize::NAMES.join(", ")
                    )
                })?;
                size = Some(parse_size(&value)?);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                if let Some(value) = arg.strip_prefix("--size=") {
                    size = Some(parse_size(value)?);
                } else {
                    return Err(format!(
                        "Unknown argument '{}'. Use --help to see supported options.",
                        arg
                    ));
                }
            }
        }
    }

    Ok(CliOptions {
        gui,
        size: size.unwrap_or(MapSize::Standard),
    })
}

fn parse_size(value: &str) -> Result<MapSize, String> {
    value.parse::<MapSize>().map_err(|_| {
        format!(
            "Unknown map size '{}'. Available options: {}.",
            value,
            MapSize::NAMES.join(", ")
        )
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run [--gui] [--size <{}>]",
        MapSize::NAMES.join("|")
    );
    println!("\nExamples:");
    println!("  cargo run -- --size standard");
    println!("  cargo run -- --gui --size huge");
}

fn print_map(map: &Map) {
    println!(
        "Map size: {} ({} × {} tiles) – {} total tiles",
        map.size(),
        map.width(),
        map.height(),
        map.tiles().len()
    );

    for (index, hex) in map.tiles().iter().enumerate() {
        println!(
            "idx={:>4} q={:>4}, r={:>4}, s={:>4}",
            index,
            hex.q(),
            hex.r(),
            hex.s()
        );
    }
}

fn run_gui(map: Map) {
    let size = map.size();
    let title = format!("Civorum – {} map", size);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title,
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.06, 0.08, 0.12)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.9, 0.95, 1.0),
            brightness: 250.0,
            affects_lightmapped_meshes: false,
        })
        .insert_resource(MapResource(map))
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_res: Res<MapResource>,
) {
    let map = &map_res.0;
    let positions: Vec<Vec2> = map.tiles().iter().copied().map(axial_to_world).collect();

    if positions.is_empty() {
        commands.spawn((Camera3d::default(), Msaa::Sample4, Transform::default()));
        return;
    }

    let (min, max) = bounding_box(&positions).expect("positions not empty");
    let center = (min + max) * 0.5;
    let span = max - min;
    let scale = compute_fit_scale(span);

    let base_hex = RegularPolygon::new(hex_radius(), 6);
    let hex_mesh = meshes.add(Extrusion::new(base_hex, 1.0));
    let hex_material = materials.add(StandardMaterial::from(Color::srgb(0.3, 0.65, 0.55)));

    for position in positions {
        let world = Vec3::new(
            (position.x - center.x) * scale,
            0.0,
            (position.y - center.y) * scale,
        );

        let mut transform =
            Transform::from_translation(world).with_rotation(Quat::from_rotation_x(-FRAC_PI_2));
        transform.scale = Vec3::new(scale, scale, scale * 0.1);

        commands.spawn((
            Mesh3d(hex_mesh.clone()),
            MeshMaterial3d(hex_material.clone()),
            transform,
        ));
    }

    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: 12_000.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, FRAC_PI_4, FRAC_PI_4, 0.0)),
    ));

    let max_extent = span.length().max(hex_radius() * 4.0) * scale;
    let distance = (max_extent * 0.75).max(hex_radius() * 10.0);

    let camera = OrbitCamera {
        target: Vec3::ZERO,
        distance,
        min_distance: hex_radius() * scale,
        max_distance: distance * 4.0,
        yaw: -FRAC_PI_4,
        pitch: FRAC_PI_4,
    };

    let transform = orbit_to_transform(&camera);

    commands.spawn((Camera3d::default(), Msaa::Sample4, transform, camera));
}

fn orbit_camera_controls(
    time: Res<Time>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        mouse_delta += event.delta;
    }

    let mut scroll_delta = 0.0;
    for event in mouse_wheel_events.read() {
        scroll_delta += event.y;
    }

    for (mut camera, mut transform) in &mut query {
        if buttons.pressed(MouseButton::Right) {
            camera.yaw -= mouse_delta.x * 0.005;
            camera.pitch += mouse_delta.y * 0.005;
            camera.pitch = camera.pitch.clamp(-FRAC_PI_4 * 3.0, FRAC_PI_4 * 3.0);
        }

        if scroll_delta.abs() > f32::EPSILON {
            let scale = 1.0 - scroll_delta * 0.1;
            camera.distance =
                (camera.distance * scale).clamp(camera.min_distance, camera.max_distance);
        }

        let mut pan_input = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            pan_input.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            pan_input.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            pan_input.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            pan_input.x += 1.0;
        }

        if pan_input.length_squared() > 0.0 {
            let rotation = Quat::from_rotation_y(camera.yaw);
            let movement =
                rotation * pan_input.normalize() * camera.distance * 0.5 * time.delta_secs();
            camera.target += Vec3::new(movement.x, 0.0, movement.z);
        }

        *transform = orbit_to_transform(&camera);
    }
}

fn orbit_to_transform(camera: &OrbitCamera) -> Transform {
    let rotation = Quat::from_euler(EulerRot::YXZ, camera.yaw, camera.pitch, 0.0);
    let offset = rotation * Vec3::Z * camera.distance;
    let position = camera.target + offset;

    Transform::from_translation(position).looking_at(camera.target, Vec3::Y)
}

fn hex_radius() -> f32 {
    SIZE as f32
}

fn axial_to_world(hex: Hex) -> Vec2 {
    let q = hex.q() as f32;
    let r = hex.r() as f32;
    let size = hex_radius();
    let x = size * 1.5 * q * TILE_PACKING;
    let y = size * (3.0_f32).sqrt() * (r + q / 2.0) * TILE_PACKING;

    Vec2::new(x, y)
}

fn bounding_box(points: &[Vec2]) -> Option<(Vec2, Vec2)> {
    let mut iter = points.iter();
    let first = *iter.next()?;
    let mut min = first;
    let mut max = first;

    for &point in iter {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }

    Some((min, max))
}

fn compute_fit_scale(span: Vec2) -> f32 {
    let base_width = span.x.abs() + hex_radius() * 2.0;
    let base_height = span.y.abs() + hex_radius() * (3.0_f32).sqrt();

    let target_width = WINDOW_WIDTH * WINDOW_MARGIN;
    let target_height = WINDOW_HEIGHT * WINDOW_MARGIN;

    let scale_w = target_width / base_width.max(1.0);
    let scale_h = target_height / base_height.max(1.0);

    scale_w.min(scale_h)
}
