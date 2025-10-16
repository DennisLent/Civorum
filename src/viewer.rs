use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

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
use bevy::render::camera::{PerspectiveProjection, Projection};
use hexx::{MeshInfo, PlaneMeshBuilder, Vec2 as HVec2};
use map::Map;

const WINDOW_WIDTH: f32 = 1400.0;
const WINDOW_HEIGHT: f32 = 900.0;

#[derive(Resource, Clone)]
struct MapRes(Map);

pub fn run_gui(map: Map) {
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
        .add_systems(Startup, setup)
        .add_systems(Update, camera::orbit_camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapRes>,
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

    // Material: disable culling so tiles are visible even if the camera goes below the plane
    let mut mat = StandardMaterial::from(Color::srgb(0.3, 0.65, 0.55));
    mat.cull_mode = None;
    let material = materials.add(mat);

    for (i, hex) in tiles.into_iter().enumerate() {
        let pos = layout.hex_to_world_pos(hex);
        // Build a hex plane mesh at `hex`. Layout handles size and position.
        let info: MeshInfo = PlaneMeshBuilder::new(&layout).at(hex).build();
        if i < 5 {
            println!(
                "viewer: tile[{i}] hex=({},{}) world=({:.2},{:.2}) verts={} tris={}",
                hex.x(),
                hex.y(),
                pos.x,
                pos.y,
                info.vertices.len(),
                info.indices.len() / 3
            );
        }
        let mesh = mesh_from_info(info);
        let handle = meshes.add(mesh);

        let entity = commands
            .spawn((
                Mesh3d(handle.clone()),
                MeshMaterial3d(material.clone()),
                // Plane faces +Y; no rotation needed
                Transform::default(),
                Wireframe, // Edge overlay for boundaries
                Name::new(format!("hex-{i}")),
            ))
            .id();
        if i < 5 {
            println!("viewer: spawned entity {:?} with mesh handle {:?}", entity, handle);
        }
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

fn mesh_from_info(info: MeshInfo) -> Mesh {
    let positions: Vec<[f32; 3]> = info.vertices.into_iter().map(|v| [v.x, v.y, v.z]).collect();
    let normals: Vec<[f32; 3]> = info.normals.into_iter().map(|n| [n.x, n.y, n.z]).collect();
    let uvs: Vec<[f32; 2]> = info.uvs.into_iter().map(|uv| [uv.x, uv.y]).collect();

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U16(info.indices))
}
