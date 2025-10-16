use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::prelude::EulerRot;
use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin};
use bevy::render::{
    mesh::Indices,
    render_asset::RenderAssetUsages,
    render_resource::{PrimitiveTopology, WgpuFeatures},
    settings::{RenderCreation, WgpuSettings},
    RenderPlugin,
};
use hexx::{
    shapes, Hex, HexLayout, HexOrientation, MeshInfo, PlaneMeshBuilder, Vec2 as HVec2,
    Vec3 as HVec3,
};
use map::{MapSize, SIZE};

const WINDOW_WIDTH: f32 = 1400.0;
const WINDOW_HEIGHT: f32 = 900.0;
const WINDOW_MARGIN: f32 = 0.9;

#[derive(Resource, Clone, Copy)]
struct GridDims {
    width: u32,
    height: u32,
}

#[derive(Component)]
struct OrbitCamera {
    target: Vec3,
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    yaw: f32,
    pitch: f32,
}

pub fn run_gui(size: MapSize) {
    let (width, height) = size.dimensions();
    let title = format!("Civorum – {} map", size);

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
        .insert_resource(GridDims { width, height })
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dims: Res<GridDims>,
) {
    // Build a layout for flat-top hexes. We'll scale and center later.
    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        origin: HVec2::ZERO,
        scale: HVec2::splat(SIZE as f32),
        ..Default::default()
    };

    // Generate axial coordinates in an even-q rectangle
    let tiles: Vec<Hex> = shapes::flat_rectangle([
        0,
        dims.width as i32 - 1,
        0,
        dims.height as i32 - 1,
    ])
    .collect();

    // Compute world centers, bounding box and fit scale
    let centers: Vec<HVec2> = tiles
        .iter()
        .map(|&h| layout.hex_to_world_pos(h))
        .collect();

    let Some((min, max)) = bounding_box(&centers) else {
        commands.spawn((Camera3d::default(), Msaa::Sample4, Transform::default()));
        return;
    };
    let center = (min + max) * 0.5;
    let span = max - min;
    let scale = compute_fit_scale(span, layout.rect_size());

    let material = materials.add(StandardMaterial::from(Color::srgb(0.3, 0.65, 0.55)));

    // Precompute center offset vector in 3D
    let center3 = HVec3::new(center.x * scale, 0.0, center.y * scale);

    for (i, hex) in tiles.into_iter().enumerate() {
        let pos = layout.hex_to_world_pos(hex);
        let pos3 = HVec3::new(pos.x, 0.0, pos.y);
        // Offsetting formula to scale both geometry and positions around center
        let custom_offset = pos3 * (scale - 1.0) - center3;

        // Build a hex plane mesh at `hex`, scaled and offset to center the grid
        let info: MeshInfo = PlaneMeshBuilder::new(&layout)
            .at(hex)
            .with_scale(HVec3::splat(scale))
            .with_offset(custom_offset)
            .build();
        let mesh = mesh_from_info(info);
        let handle = meshes.add(mesh);

        commands.spawn((
            Mesh3d(handle),
            MeshMaterial3d(material.clone()),
            // Each plane already sits flat; rotate so Y is up in Bevy
            Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            Wireframe, // Edge overlay for boundaries
            Name::new(format!("hex-{i}")),
        ));
    }

    // Light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: 12_000.0,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, FRAC_PI_4, FRAC_PI_4, 0.0)),
    ));

    let max_extent = span.length().max((SIZE as f32) * 4.0) * scale;
    let distance = (max_extent * 0.75).max((SIZE as f32) * 10.0);

    let camera = OrbitCamera {
        target: Vec3::ZERO,
        distance,
        min_distance: (SIZE as f32) * scale,
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
            camera.pitch = camera.pitch.clamp(-FRAC_PI_2 * 3.0, FRAC_PI_2 * 3.0);
        }

        if scroll_delta.abs() > f32::EPSILON {
            let s = 1.0 - scroll_delta * 0.1;
            camera.distance = (camera.distance * s).clamp(camera.min_distance, camera.max_distance);
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
            let movement = rotation * pan_input.normalize() * camera.distance * 0.5 * time.delta_secs();
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

fn compute_fit_scale(span: HVec2, rect: HVec2) -> f32 {
    let base_width = span.x.abs() + rect.x;
    let base_height = span.y.abs() + rect.y;
    let target_width = WINDOW_WIDTH * WINDOW_MARGIN;
    let target_height = WINDOW_HEIGHT * WINDOW_MARGIN;
    let scale_w = target_width / base_width.max(1.0);
    let scale_h = target_height / base_height.max(1.0);
    scale_w.min(scale_h)
}

fn mesh_from_info(info: MeshInfo) -> Mesh {
    let positions: Vec<[f32; 3]> = info.vertices.into_iter().map(|v| [v.x, v.y, v.z]).collect();
    let normals: Vec<[f32; 3]> = info.normals.into_iter().map(|n| [n.x, n.y, n.z]).collect();
    let uvs: Vec<[f32; 2]> = info.uvs.into_iter().map(|uv| [uv.x, uv.y]).collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U16(info.indices))
}
