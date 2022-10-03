/**
 * This is a deno script that converts the old Jumpy maps to the new map format.
 */

export interface OldMap {
  name: string;
  background_color: BackgroundColor;
  background_layers: BackgroundLayer[];
  world_offset: GridSize;
  grid_size: GridSize;
  tile_size: GridSize;
  layers: Layer[];
  tilesets: Tileset[];
  spawn_points: GridSize[];
}

export interface BackgroundColor {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

export interface BackgroundLayer {
  texture_id: string;
  depth: number;
  offset: GridSize;
}

export interface GridSize {
  x: number;
  y: number;
}

export interface Layer {
  id: string;
  kind: string;
  has_collision: boolean;
  tiles?: number[];
  is_visible: boolean;
  objects?: Object[];
}

export interface Object {
  id: string;
  kind: Kind;
  position: GridSize;
}

export enum Kind {
  Decoration = "decoration",
  Environment = "environment",
  Item = "item",
}

export interface Tileset {
  id: string;
  texture_id: string;
  texture_size: GridSize;
  tile_size: GridSize;
  grid_size: GridSize;
  first_tile_id: number;
  tile_cnt: number;
  tile_subdivisions: GridSize;
  autotile_mask: boolean[];
  tile_attributes?: { [key: string]: string[] };
}

const dir = Deno.readDir(".");
const inMaps: OldMap[] = [];

for await (const item of dir) {
  if (item.isFile && item.name.endsWith("json") && !item.name.includes("map")) {
    const json = JSON.parse(await Deno.readTextFile(`./${item.name}`));
    json.name = item.name.replace(".json", "");
    inMaps.push(json);
  }
}

const newMaps = [] as any;

for (const map of inMaps) {
  const newMap = {} as any;

  newMap.name = map.name;

  newMap.grid_size = [map.grid_size.x, map.grid_size.y];
  newMap.tile_size = [map.tile_size.x, map.tile_size.y];
  newMap.layers = [];

  for (const layer of map.layers) {
    const newLayer = {} as any;

    newLayer.id = layer.id;

    if (layer.tiles && layer.tiles.length > 0) {
      newLayer.kind = { tile: {} };
      newLayer.kind.tile.has_collision = layer.has_collision;
      newLayer.kind.tile.tiles = [];
      newLayer.kind.tile.tilemap = "./resources/default_tileset.png";

      for (let i = 0; i < layer.tiles!.length; i++) {
        const tile = layer.tiles![i];
        if (tile == 0) continue;
        const posX = i % map.grid_size.x;
        const posY = Math.floor(i / map.grid_size.y);

        newLayer.kind.tile.tiles.push({
          pos: [posX, posY],
          idx: tile - 1,
        });
      }
    } else if (layer.objects && layer.objects.length > 0) {
      newLayer.kind = { entity: { entities: [] } };
      for (const object of layer.objects) {
        const entity = {} as any;
        entity.pos = [object.position.x, object.position.y];
        entity.item = `./items/${object.id}.item.yaml`;
        newLayer.kind.entity.entities.push(entity);
      }
    } else {
      continue;
    }

    newMap.layers.push(newLayer);
  }

  newMaps.push(newMap);
}

for (const map of newMaps) {
  const name = map.name;
  const filename = `../../assets/maps/${name}.map.json`;

  await Deno.writeTextFile(filename, JSON.stringify(map, undefined, "  "));
}
