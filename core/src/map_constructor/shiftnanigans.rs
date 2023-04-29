use std::{cell::RefCell, rc::Rc};

use crate::prelude::{LoadedMap, SpawnedMapLayerMeta};
use super::MapConstructor;
use bones_lib::prelude::{CompMut, Transform};
use glam::{UVec2, Vec2};
use shiftnanigans::pixel_board::{
    pixel_board_randomizer::PixelBoardRandomizer,
    Pixel, PixelBoard
};

pub struct ShiftnanigansMapConstructor {
    pixel_board_randomizer: PixelBoardRandomizer<PixelType>,
    original_width: u32,
    original_height: u32
}

impl ShiftnanigansMapConstructor {
    fn from_map(&self, map: &LoadedMap) -> ShiftnanigansMapConstructor {

        // . map tiles and elements into pixel types
        // . run pixel randomizer
        // . transform tiles and elements based on where they exist in the randomized map

        let mut ungrouped_pixels_per_y_per_x: Vec<Vec<Option<UngroupedPixel>>> = Vec::new();
        for y in 0..map.grid_size.y {
            let mut ungrouped_pixels_per_y: Vec<Option<UngroupedPixel>> = Vec::new();
            for x in 0..map.grid_size.x {
                let composite_layer_pixel_option = UngroupedPixel::from_loaded_map_location(map, x, y);
                ungrouped_pixels_per_y.push(composite_layer_pixel_option);
            }
            ungrouped_pixels_per_y_per_x.push(ungrouped_pixels_per_y);
        }

        let mut top_height;
        'get_top_height: {
            for y in 0..map.grid_size.y as usize {
                // if any column is missing a pixel, then the ceiling wall is as thick as the current value of 'y'
                for x in 0..map.grid_size.x as usize {
                    if ungrouped_pixels_per_y_per_x[x][y].is_none() {
                        top_height = y;
                        break 'get_top_height;
                    }
                }
            }
            top_height = map.grid_size.y as usize;
        }

        let mut left_width;
        'get_left_width: {
            for x in 0..map.grid_size.x as usize {
                for y in 0..map.grid_size.y as usize {
                    if ungrouped_pixels_per_y_per_x[x][y].is_none() {
                        left_width = x;
                        break 'get_left_width;
                    }
                }
            }
            left_width = map.grid_size.x as usize;
        }

        let mut bottom_height;
        'get_bottom_height: {
            for y in (0..map.grid_size.y as usize).rev() {
                for x in 0..map.grid_size.x as usize {
                    if ungrouped_pixels_per_y_per_x[x][y].is_none() {
                        bottom_height = map.grid_size.y as usize - y - 1;
                        break 'get_bottom_height;
                    }
                }
            }
            bottom_height = 0;  // pretend the bottom is missing since the top_height must also be the entire height
        }

        let mut right_width;
        'get_right_width: {
            for x in (0..map.grid_size.x as usize).rev() {
                for y in 0..map.grid_size.y as usize {
                    if ungrouped_pixels_per_y_per_x[x][y].is_none() {
                        right_width = map.grid_size.x as usize - x - 1;
                        break 'get_right_width;
                    }
                }
            }
            right_width = 0;  // pretend the right is missing since the left_width must also be the entire width
        }

        let composite_map_width = map.grid_size.x as usize - std::cmp::max(left_width, 1) - std::cmp::max(right_width, 1) + 2;
        let composite_map_height = map.grid_size.y as usize - std::cmp::max(top_height, 1) - std::cmp::max(bottom_height, 1) + 2;

        let mut grouped_pixels_per_y_per_x: Vec<Vec<Vec<GroupedPixel>>> = Vec::new();
        for x in 0..composite_map_width {
            let mut grouped_pixels_per_y: Vec<Vec<GroupedPixel>> = Vec::new();
            for y in 0..composite_map_height {
                grouped_pixels_per_y.push(Vec::new());
            }
            grouped_pixels_per_y_per_x.push(grouped_pixels_per_y);
        }

        for x in 0..map.grid_size.x as usize {
            for y in 0..map.grid_size.y as usize {

                // the pixel board coordinates based on the map grid coordinates 'x' and 'y'
                let pixel_board_x;
                let pixel_board_y;

                if x < left_width {
                    pixel_board_x = 0;
                }
                else if x >= map.grid_size.x as usize - right_width {
                    pixel_board_x = composite_map_width - 1;
                }
                else {
                    if left_width == 0 {
                        pixel_board_x = 0;
                    }
                    else {
                        pixel_board_x = x - left_width + 1;
                    }
                }

                if y < top_height {
                    pixel_board_y = 0;
                }
                else if y >= map.grid_size.y as usize - bottom_height {
                    pixel_board_y = composite_map_height - 1;
                }
                else {
                    if top_height == 0 {
                        pixel_board_y = y;
                    }
                    else {
                        pixel_board_y = y - top_height + 1
                    }
                }

                // add the existing pixel to the grouped pixels (changing from None to an instance if applicable)
                // at this point we know that (x, y) in ungrouped pixels equates to (pixel_board_x, pixel_board_y) in grouped pixels

                if let Some(ungrouped_pixel) = ungrouped_pixels_per_y_per_x[x][y] {
                    grouped_pixels_per_y_per_x[pixel_board_x][pixel_board_y].push(GroupedPixel { ungrouped_pixel, ungrouped_pixel_location: UVec2 { x: x as u32, y: y as u32 } });
                }
            }
        }

        // at this point all of the grouped pixels are known and so the pixel types can now be added to the pixel board

        let mut pixel_board: PixelBoard<PixelType> = PixelBoard::new(composite_map_width, composite_map_height);

        for y in 0..composite_map_height {
            for x in 0..composite_map_width {
                if grouped_pixels_per_y_per_x[x][y].len() != 0 {
                    let pixel_type = PixelType {
                        grouped_pixels: grouped_pixels_per_y_per_x[x][y]
                    };
                    pixel_board.set(x, y, Rc::new(RefCell::new(pixel_type)));
                }
            }
        }

        ShiftnanigansMapConstructor {
            pixel_board_randomizer: PixelBoardRandomizer::new(pixel_board),
            original_width: map.grid_size.x,
            original_height: map.grid_size.y
        }
    }
}

impl MapConstructor for ShiftnanigansMapConstructor {
    fn construct_map(&self, mut spawned_map_layer_metas: CompMut<SpawnedMapLayerMeta>, mut transforms: CompMut<Transform>) -> LoadedMap {
        let random_pixel_board = self.pixel_board_randomizer.get_random_pixel_board();


        todo!()
    }
}

/// The tile as it exists in the map
struct Tile {
    idx: u32
}

/// The type of the element
enum ElementType {
    Crab,
    Crate,
    Decoration,
    FishSchool,
    Grenade,
    KickBomb,
    Mine,
    Musket,
    PlayerSpawner,
    Slippery,
    SlipperySeaweed,
    Sproinger,
    StompBoots,
    Sword,
    Urchin,
    Unknown
}

/// An element at a location within the map
struct Element {
    element_type: ElementType,
    offset_location: Vec2
}

/// The entity as it exists in the map
enum LayerPixelEntityType {
    Tile(Tile),
    Element(Element)
}

/// This contains the layer pixels
struct UngroupedPixel {
    layer_pixel_entity_types: Vec<LayerPixelEntityType>
}

impl UngroupedPixel {
    fn from_loaded_map_location(map: &LoadedMap, x: u32, y: u32) -> Option<Self> {
        // iterate over each layer at the provided location, creating a vector of MapEntityType
        let mut entity_types: Vec<LayerPixelEntityType> = Vec::new();
        map.layers
            .iter()
            .for_each(|layer| {
                layer.tiles
                    .iter()
                    .for_each(|tile| {
                        if tile.pos.x == x &&
                            tile.pos.y == y {

                            entity_types.push(LayerPixelEntityType::Tile(Tile {
                                idx: tile.idx
                            }));
                        }
                    });
                layer.elements
                    .iter()
                    .for_each(|element| {
                        // TODO grab elements nearby this location
                        if element.pos.x as u32 == x &&
                            element.pos.y as u32 == y {

                            // TODO determine which element this is in 'element'
                            entity_types.push(LayerPixelEntityType::Element(Element {
                                element_type: ElementType::Unknown,
                                offset_location: Vec2::new(element.pos.x - x as f32, element.pos.y - y as f32)
                            }));
                        }
                    });
            });
        if entity_types.len() == 0 {
            None
        }
        else {
            Some(UngroupedPixel { layer_pixel_entity_types: entity_types })
        }
    }
}

/// This can contain one or more ungrouped pixels, allowing for combining adjacent pixels into one grouped pixel
///     This is useful for when thick walls need to be considered as one-pixel-wide walls
struct GroupedPixel {
    ungrouped_pixel: UngroupedPixel,
    ungrouped_pixel_location: UVec2
}

/// The pixel provided to the pixel board randomizer
struct PixelType {
    grouped_pixels: Vec<GroupedPixel>
}

// TODO pickup here: change the pixel type to potentially contain walls and non-walls in the same pixel
//  also include the entity identifier

impl Pixel for PixelType {
    fn get_invalid_location_offsets_for_other_pixel(&self, other_pixel: &Self) -> Vec<(i16, i16)> {
        let mut invalid_location_offsets: Vec<(i16, i16)> = Vec::new();
        self.grouped_pixels
            .iter()
            .for_each(|gp| {
                gp.ungrouped_pixel.layer_pixel_entity_types
                    .iter()
                    .for_each(|lpet| {
                        match lpet {
                            LayerPixelEntityType::Tile(_) => {
                                // there are no restrictions on where the other pixel can exist
                            },
                            LayerPixelEntityType::Element(element) => {
                                match element.element_type {
                                    ElementType::Crab => {
                                        // the crab does not have any restrictions on the area around it
                                    },
                                    ElementType::Crate => {
                                        // the crate does not have any restrictions on the area around it
                                    },
                                    ElementType::Decoration => {
                                        // the decoration does not have any restrictions on the area around it
                                    },
                                    ElementType::FishSchool => {
                                        // the fish school does not have any restrictions on the area around it
                                    },
                                    ElementType::Grenade => {
                                        // the grenade does not have any restrictions on the area around it
                                    },
                                    ElementType::KickBomb => {
                                        // the kick bomb does not have any restrictions on the area around it
                                    },
                                    ElementType::Mine => {
                                        // the mine does not have any restrictions on the area around it
                                    },
                                    ElementType::Musket => {
                                        // the musket does not have any restrictions on the area around it
                                    },
                                    ElementType::PlayerSpawner => {
                                        // TODO
                                    },
                                    ElementType::Slippery => {
                                        // the slippery does not have any restrictions on the area around it
                                    },
                                    ElementType::SlipperySeaweed => {
                                        // the slippery seaweed does not have any restrictions on the area around it
                                    },
                                    ElementType::Sproinger => {
                                        // TODO
                                    },
                                    ElementType::StompBoots => {
                                        // the stomp boots does not have any restrictions on the area around it
                                    },
                                    ElementType::Sword => {
                                        // the sword does not have any restrictions on the area around it
                                    },
                                    ElementType::Urchin => {
                                        // the urchin does not have any restrictions on the area around it
                                    },
                                    ElementType::Unknown => {
                                        // TODO log error
                                    }
                                }
                            }
                        }
                    });
            });
        invalid_location_offsets
    }
}
