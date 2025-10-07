#![cfg(feature = "gui")]
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues as VAV;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::{PrimitiveTopology, RenderAssetUsages};
use map::{Map, Hex, Terrain, Biome, WaterDepth};

pub fn run(map: Map) {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.03)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "Civorum".into(), ..default() }),
            ..default()
        }))
        .insert_resource(WorldMap(map))
        .add_systems(Startup, (setup_camera_light, spawn_tiles))
        .run();
}

#[derive(Resource)]
struct WorldMap(Map);

fn setup_camera_light(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20.0, 40.0, 60.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { illuminance: 25_000.0, shadows_enabled: false, ..default() },
        transform: Transform::from_xyz(20.0, 50.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn spawn_tiles(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, world: Res<WorldMap>) {
    let size = 1.0f32; // hex radius
    let hmesh = hex_mesh(size);
    let mesh_handle = meshes.add(hmesh);
    let mut mat_cache: std::collections::BTreeMap<TileKey, Handle<StandardMaterial>> = Default::default();

    for (h, t) in world.0.iter() {
        let (x, z) = axial_to_world(h, size);
        let y = t.elevation * 0.05; // vertical exaggeration
        let color = tile_color(t);
        let key = TileKey::from_tile(*t);
        let mat = mat_cache.entry(key).or_insert_with(|| materials.add(StandardMaterial { base_color: color, unlit: true, ..default() }));
        commands.spawn(PbrBundle {
            mesh: mesh_handle.clone(),
            material: mat.clone(),
            transform: Transform::from_xyz(x, y, z),
            ..default()
        });
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum TileKey { Land(Biome), Water(WaterDepth) }
impl TileKey { fn from_tile(t: map::Tile) -> Self { match t.terrain { Terrain::Land => TileKey::Land(t.biome.unwrap()), Terrain::Water => TileKey::Water(t.water.unwrap()) } } }

fn axial_to_world(h: Hex, size: f32) -> (f32, f32) {
    let x = size * (3f32.sqrt()) * (h.q as f32 + h.r as f32 * 0.5);
    let z = size * (1.5) * (h.r as f32);
    (x, z)
}

fn hex_mesh(size: f32) -> Mesh {
    // Flat, unlit hex; triangles fan around center.
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(7);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(7);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(7);
    let mut indices: Vec<u32> = Vec::with_capacity(6 * 3);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..6 {
        let ang = std::f32::consts::PI / 180.0 * (60.0 * i as f32 - 30.0);
        let x = size * ang.cos();
        let z = size * ang.sin();
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + x / (2.0 * size), 0.5 + z / (2.0 * size)]);
    }
    for i in 1..6 { indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]); }
    indices.extend_from_slice(&[0, 6, 1]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VAV::Float32x3(positions));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VAV::Float32x3(normals));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VAV::Float32x2(uvs));
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn tile_color(t: &map::Tile) -> Color {
    match t.terrain {
        Terrain::Water => match t.water.unwrap() {
            WaterDepth::DeepOcean => Color::srgb(0.02, 0.08, 0.25),
            WaterDepth::Ocean => Color::srgb(0.05, 0.15, 0.35),
            WaterDepth::Shallow => Color::srgb(0.10, 0.35, 0.55),
        },
        Terrain::Land => match t.biome.unwrap() {
            Biome::TropicalForest => Color::srgb(0.0, 0.45, 0.1),
            Biome::Savanna => Color::srgb(0.65, 0.65, 0.2),
            Biome::Desert => Color::srgb(0.85, 0.75, 0.4),
            Biome::TemperateForest => Color::srgb(0.05, 0.6, 0.15),
            Biome::Temperate => Color::srgb(0.35, 0.6, 0.25),
            Biome::Prairie => Color::srgb(0.65, 0.8, 0.4),
            Biome::Grassland => Color::srgb(0.5, 0.75, 0.3),
            Biome::Taiga => Color::srgb(0.2, 0.55, 0.35),
            Biome::Tundra => Color::srgb(0.7, 0.8, 0.8),
            Biome::Snow => Color::srgb(0.95, 0.97, 0.99),
        },
    }
}
