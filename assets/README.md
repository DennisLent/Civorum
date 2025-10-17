Asset layout for Civorum

Overview
- Bevy loads assets from the `assets/` directory at the crate root.
- Prefer glTF 2.0 binary models (`.glb`) for 3D content. Keep meshes, materials, and textures embedded where possible.
- Keep low poly counts for tiles/props to maintain performance across large maps.

Folders
- `assets/models/` – 3D models (preferred: `.glb`)
- `assets/textures/` – standalone textures used by materials or atlases

GLB authoring guidance
- Origin/pivot at the tile center `(0,0,0)`, +Y up.
- Orientation: flat‑top hex geometry (long edge horizontal) to match our layout.
- Scale: if possible, model with circumradius = 1. Code will uniformly scale to `map::SIZE`.
- Materials: PBR (Base Color sRGB, Metallic‑Roughness linear, Normal linear). Keep textures modest (≤1024px) to save memory.

In‑engine loading tips
- Whole scene: `asset_server.load("models/your_hex.glb#Scene0")`.
- Specific mesh/material: `#Mesh0/Primitive0`, `#Material0` selectors on the same `.glb`.
- Reuse handles (clone) across tiles to avoid duplicating GPU resources.

