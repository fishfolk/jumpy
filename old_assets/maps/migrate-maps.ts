/**
 * This is a deno script that converts the old Jumpy maps to the new map format.
 */

import { assert } from "https://deno.land/std@0.158.0/testing/asserts.ts";
import { bufferToHex } from "https://deno.land/x/hextools@v1.0.0/mod.ts";

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

  // newMap.background_color = bufferToHex(
  //   new Uint8Array([
  //     255 * map.background_color.red,
  //     255 * map.background_color.green,
  //     255 * map.background_color.blue,
  //     255 * map.background_color.alpha,
  //   ])
  // );

  // This is the background color all maps should have but only some of them actually had it set, so
  // we set it manually instead of pulling it from the map JSON like the commented code above does.
  newMap.background_color = "7fb2b2ff"; 
  newMap.grid_size = [map.grid_size.x, map.grid_size.y];
  newMap.tile_size = [map.tile_size.x, map.tile_size.y];
  newMap.layers = [];

  newMap.background_layers = [
    {
      image: "./resources/background_04.png",
      speed: 0,
      tile_size: [896, 480],
      z: -110,
      position: [0, 360],
      scale: 2.0,
    },
    {
      image: "./resources/background_03.png",
      speed: 0.74,
      tile_size: [896, 480],
      z: -109,
      position: [0, 360],
      scale: 2.0,
    },
    {
      image: "./resources/background_02.png",
      speed: 0.82,
      tile_size: [896, 480],
      z: -108,
      position: [0, 360],
      scale: 2.0,
    },
    {
      image: "./resources/background_01.png",
      speed: 100,
      tile_size: [896, 480],
      z: -107,
      position: [0, 360],
      scale: 2.0,
    },
  ];

  for (const layer of map.layers) {
    const newLayer = {} as any;

    newLayer.id = layer.id;

    if (layer.tiles && layer.tiles.length > 0) {
      assert(layer.tiles.length == map.grid_size.x * map.grid_size.y);

      newLayer.kind = { tile: {} };
      newLayer.kind.tile.has_collision = layer.has_collision;
      newLayer.kind.tile.tiles = [];
      newLayer.kind.tile.tilemap = "./resources/default_tileset.png";

      for (let i = 0; i < layer.tiles!.length; i++) {
        const tile = layer.tiles![i];
        if (tile == 0) continue;
        const posX = i % map.grid_size.x;
        const posY = map.grid_size.y - 1 - Math.floor(i / map.grid_size.x);

        assert(posX < map.grid_size.x);
        assert(
          posY < map.grid_size.y,
          `posY ( ${posY} ) isn't less than map.grid_size.y ( ${map.grid_size.y} ) for tile index ${i}`
        );

        newLayer.kind.tile.tiles.push({
          pos: [posX, posY],
          idx: tile - 1,
        });
      }
    } else if (layer.objects && layer.objects.length > 0) {
      newLayer.kind = { element: { elements: [] } };
      for (const object of layer.objects) {
        const element = {} as any;
        element.pos = [object.position.x, object.position.y];
        element.name = object.id;
        element.element = `./elements/${object.id}.item.yaml`;
        element.category = object.kind;
        newLayer.kind.element.elements.push(element);
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
