#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/generate_debug_maps.sh [options] <seed_or_seeds...>

Examples:
  scripts/generate_debug_maps.sh 2
  scripts/generate_debug_maps.sh 2 12 42
  scripts/generate_debug_maps.sh --size huge --cell-px 14 --out out/debug_maps 2,12,42
  scripts/generate_debug_maps.sh --size standard --none

Options:
  --size <duel|tiny|small|standard|large|huge>  Map size (default: standard)
  --cell-px <int>                                Hex cell size in pixels (default: 16)
  --out <dir>                                    Output directory (default: out/debug_maps)
  --none                                         Use seed "none" (random/default behavior in map code)
  -h, --help                                     Show this help

Notes:
  - Seeds can be passed as separate args and/or comma-separated values.
  - The script renders all map types for every provided seed.
USAGE
}

SIZE="standard"
CELL_PX="16"
OUT_DIR="out/debug_maps"
SEEDS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --size)
      [[ $# -ge 2 ]] || { echo "Missing value for --size" >&2; exit 1; }
      SIZE="$2"
      shift 2
      ;;
    --cell-px)
      [[ $# -ge 2 ]] || { echo "Missing value for --cell-px" >&2; exit 1; }
      CELL_PX="$2"
      shift 2
      ;;
    --out)
      [[ $# -ge 2 ]] || { echo "Missing value for --out" >&2; exit 1; }
      OUT_DIR="$2"
      shift 2
      ;;
    --none)
      SEEDS+=("none")
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      IFS=',' read -r -a split <<< "$1"
      for seed in "${split[@]}"; do
        [[ -n "$seed" ]] && SEEDS+=("$seed")
      done
      shift
      ;;
  esac
done

if [[ ${#SEEDS[@]} -eq 0 ]]; then
  usage
  exit 1
fi

case "$SIZE" in
  duel|tiny|small|standard|large|huge) ;;
  *)
    echo "Invalid --size '$SIZE'" >&2
    exit 1
    ;;
esac

if ! [[ "$CELL_PX" =~ ^[0-9]+$ ]]; then
  echo "--cell-px must be an integer" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

MAP_TYPES=(
  "continents"
  "small_continents"
  "islands_continents"
  "pangea"
  "mirror"
  "terra"
)

echo "Building debug renderer binary..."
cargo build -p civorum-core --bin render_debug_map >/dev/null

BIN="./target/debug/render_debug_map"
mkdir -p "$OUT_DIR"

for seed in "${SEEDS[@]}"; do
  seed_dir="$OUT_DIR/seed_${seed}"
  mkdir -p "$seed_dir"

  echo "Generating maps for seed '$seed' (size=$SIZE, cell_px=$CELL_PX)..."
  for map_type in "${MAP_TYPES[@]}"; do
    out_file="$seed_dir/${map_type}.png"
    "$BIN" "$SIZE" "$seed" "$map_type" "$CELL_PX" "$out_file"
  done
done

echo "Done. Wrote debug maps to: $OUT_DIR"
