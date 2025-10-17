#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
    Water,
    Mountain,
    Desert,
    Grass,
    Forrest
}

impl Terrain {

    pub fn terrain_to_file(self) -> &'static str {
        match self {
            Terrain::Water => "water.glb",
            Terrain::Desert => "sand.glb",
            Terrain::Forrest => "grass-forest.glb",
            Terrain::Grass => "grass.glb",
            Terrain::Mountain => "stone-mountain.glb"
        }
    }
}