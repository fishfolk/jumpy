use std::{cell::RefCell, rc::Rc};

use super::MapConstructor;
use crate::{
    editor::MapManager,
    input::{ElementLayer, TileLayer},
    metadata::ElementMeta,
    physics::TileCollisionKind,
};
use bones_lib::prelude::Handle;
use glam::{UVec2, Vec2};
use shiftnanigans::pixel_board::{pixel_board_randomizer::PixelBoardRandomizer, Pixel, PixelBoard};

pub struct ShiftnanigansMapConstructor {
    pixel_board_randomizer: PixelBoardRandomizer<PixelType>,
    compressed_top_height: usize,
    compressed_left_width: usize,
    tile_size: Vec2,
}

impl ShiftnanigansMapConstructor {
    pub fn new(
        map_size: UVec2,
        tile_size: Vec2,
        tile_layers: &[TileLayer],
        element_layers: &[ElementLayer],
    ) -> ShiftnanigansMapConstructor {
        // . map tiles and elements into pixel types
        // . run pixel randomizer
        // . transform tiles and elements based on where they exist in the randomized map

        let mut ungrouped_pixel_per_y_per_x: Vec<Vec<Option<UngroupedPixel>>> = Vec::new();
        for x in 0..map_size.x {
            let mut ungrouped_pixel_per_y: Vec<Option<UngroupedPixel>> = Vec::new();
            for y in 0..map_size.y {
                let ungrouped_pixel_option =
                    UngroupedPixel::from_layers_and_location(tile_layers, element_layers, x, y);
                ungrouped_pixel_per_y.push(ungrouped_pixel_option);
            }
            ungrouped_pixel_per_y_per_x.push(ungrouped_pixel_per_y);
        }

        let top_height;
        'get_top_height: {
            for y in 0..map_size.y as usize {
                // if any column is missing a pixel, then the ceiling wall is as thick as the current value of 'y'
                for ungrouped_pixel_per_y in ungrouped_pixel_per_y_per_x.iter() {
                    if ungrouped_pixel_per_y[y].is_none() {
                        top_height = y;
                        break 'get_top_height;
                    }
                }
            }
            top_height = map_size.y as usize;
        }

        let left_width;
        'get_left_width: {
            for (x, ungrouped_pixel_per_y) in ungrouped_pixel_per_y_per_x.iter().enumerate() {
                for ungrouped_pixel in ungrouped_pixel_per_y.iter() {
                    if ungrouped_pixel.is_none() {
                        left_width = x;
                        break 'get_left_width;
                    }
                }
            }
            left_width = map_size.x as usize;
        }

        let bottom_height;
        'get_bottom_height: {
            for y in (0..map_size.y as usize).rev() {
                for ungrouped_pixel_per_y in ungrouped_pixel_per_y_per_x.iter() {
                    if ungrouped_pixel_per_y[y].is_none() {
                        bottom_height = map_size.y as usize - y - 1;
                        break 'get_bottom_height;
                    }
                }
            }
            bottom_height = 0; // pretend the bottom is missing since the top_height must also be the entire height
        }

        let right_width;
        'get_right_width: {
            for (x, ungrouped_pixel_per_y) in ungrouped_pixel_per_y_per_x.iter().enumerate().rev() {
                for ungrouped_pixel in ungrouped_pixel_per_y.iter() {
                    if ungrouped_pixel.is_none() {
                        right_width = map_size.x as usize - x - 1;
                        break 'get_right_width;
                    }
                }
            }
            right_width = 0; // pretend the right is missing since the left_width must also be the entire width
        }

        let composite_map_width =
            map_size.x as usize - std::cmp::max(left_width, 1) - std::cmp::max(right_width, 1) + 2;
        let composite_map_height =
            map_size.y as usize - std::cmp::max(top_height, 1) - std::cmp::max(bottom_height, 1)
                + 2;

        let mut grouped_pixels_per_y_per_x: Vec<Vec<Vec<GroupedPixel>>> =
            Vec::with_capacity(composite_map_width);
        for _ in 0..composite_map_width {
            let mut grouped_pixels_per_y: Vec<Vec<GroupedPixel>> =
                Vec::with_capacity(composite_map_height);
            for _ in 0..composite_map_height {
                grouped_pixels_per_y.push(Vec::new());
            }
            grouped_pixels_per_y_per_x.push(grouped_pixels_per_y);
        }

        ungrouped_pixel_per_y_per_x
            .into_iter()
            .enumerate()
            .for_each(|(x, ungrouped_pixel_per_y)| {
                ungrouped_pixel_per_y
                    .into_iter()
                    .enumerate()
                    .for_each(|(y, ungrouped_pixel)| {
                        // the pixel board coordinates based on the map grid coordinates 'x' and 'y'
                        let pixel_board_x;
                        let pixel_board_y;

                        if x < left_width {
                            pixel_board_x = 0;
                        } else if x >= map_size.x as usize - right_width {
                            pixel_board_x = composite_map_width - 1;
                        } else if left_width == 0 {
                            pixel_board_x = x;
                        } else {
                            pixel_board_x = x - left_width + 1;
                        }

                        if y < top_height {
                            pixel_board_y = 0;
                        } else if y >= map_size.y as usize - bottom_height {
                            pixel_board_y = composite_map_height - 1;
                        } else if top_height == 0 {
                            pixel_board_y = y;
                        } else {
                            pixel_board_y = y - top_height + 1
                        }

                        // add the existing pixel to the grouped pixels (changing from None to an instance if applicable)
                        // at this point we know that (x, y) in ungrouped pixels equates to (pixel_board_x, pixel_board_y) in grouped pixels

                        if let Some(ungrouped_pixel) = ungrouped_pixel {
                            grouped_pixels_per_y_per_x[pixel_board_x][pixel_board_y].push(
                                GroupedPixel {
                                    ungrouped_pixel,
                                    ungrouped_pixel_location: UVec2 {
                                        x: x as u32,
                                        y: y as u32,
                                    },
                                },
                            );
                        }
                    });
            });

        // at this point all of the grouped pixels are known and so the pixel types can now be added to the pixel board

        let mut pixel_board: PixelBoard<PixelType> =
            PixelBoard::new(composite_map_width, composite_map_height);

        grouped_pixels_per_y_per_x
            .into_iter()
            .enumerate()
            .for_each(|(x, grouped_pixels_per_y)| {
                grouped_pixels_per_y
                    .into_iter()
                    .enumerate()
                    .for_each(|(y, grouped_pixels)| {
                        if !grouped_pixels.is_empty() {
                            let pixel_type = PixelType { grouped_pixels };
                            pixel_board.set(x, y, Rc::new(RefCell::new(pixel_type)));
                        }
                    });
            });

        ShiftnanigansMapConstructor {
            pixel_board_randomizer: PixelBoardRandomizer::new(pixel_board),
            compressed_top_height: top_height,
            compressed_left_width: left_width,
            tile_size,
        }
    }
}

impl MapConstructor for ShiftnanigansMapConstructor {
    fn construct_map(&self, map_manager: &mut MapManager) {
        let random_pixel_board = self.pixel_board_randomizer.get_random_pixel_board();

        // remove all tiles
        map_manager.clear_tiles();

        // remove all elements
        map_manager.clear_elements();

        // place all tiles and elements
        for y in 0..random_pixel_board.get_height() {
            for x in 0..random_pixel_board.get_width() {
                if random_pixel_board.exists(x, y) {
                    let wrapped_pixel = random_pixel_board.get(x, y).unwrap();
                    let borrowed_pixel: &PixelType = &wrapped_pixel.borrow();
                    let top_left_position: UVec2 = borrowed_pixel
                        .grouped_pixels
                        .first()
                        .unwrap()
                        .ungrouped_pixel_location;

                    // calculate the x and y that the grouped pixels uncompress to
                    let uncompressed_y: usize = if y == 0 {
                        0
                    } else {
                        let top_offset: usize = if self.compressed_top_height == 0 {
                            0
                        } else {
                            self.compressed_top_height - 1
                        };
                        y + top_offset
                    };
                    let uncompressed_x: usize = if x == 0 {
                        0
                    } else {
                        let left_offset: usize = if self.compressed_left_width == 0 {
                            0
                        } else {
                            self.compressed_left_width - 1
                        };
                        x + left_offset
                    };

                    borrowed_pixel.grouped_pixels.iter().for_each(|gp| {
                        gp.ungrouped_pixel
                            .layer_pixel_entity_types
                            .iter()
                            .for_each(|lpet| match lpet {
                                LayerPixelEntityType::Tile(tile) => {
                                    let position = UVec2 {
                                        x: uncompressed_x as u32 + gp.ungrouped_pixel_location.x
                                            - top_left_position.x,
                                        y: uncompressed_y as u32 + gp.ungrouped_pixel_location.y
                                            - top_left_position.y,
                                    };
                                    map_manager.set_tile(
                                        tile.layer_index,
                                        position,
                                        &Some(tile.tilemap_tile_index as usize),
                                        tile.tile_collision_kind,
                                    );
                                }
                                LayerPixelEntityType::Element(element) => {
                                    let position = Vec2 {
                                        x: (uncompressed_x as f32
                                            + gp.ungrouped_pixel_location.x as f32
                                            - top_left_position.x as f32
                                            + element.position.x)
                                            * self.tile_size.x,
                                        y: (uncompressed_y as f32
                                            + gp.ungrouped_pixel_location.y as f32
                                            - top_left_position.y as f32
                                            + element.position.y)
                                            * self.tile_size.y,
                                    };
                                    map_manager.create_element(
                                        &element.element_meta_handle,
                                        &position,
                                        element.layer_index,
                                    );
                                }
                            });
                    });
                }
            }
        }
    }
}

/// The tile as it exists in the map
struct Tile {
    layer_index: usize,
    tilemap_tile_index: u32,
    tile_collision_kind: TileCollisionKind,
}

/// An element at a location within the map
struct Element {
    layer_index: usize,
    element_meta_handle: Handle<ElementMeta>,
    position: Vec2,
}

/// The entity as it exists in the map
enum LayerPixelEntityType {
    Tile(Tile),
    Element(Element),
}

/// This contains the layer pixels
struct UngroupedPixel {
    layer_pixel_entity_types: Vec<LayerPixelEntityType>,
}

impl UngroupedPixel {
    fn from_layers_and_location(
        tile_layers: &[TileLayer],
        element_layers: &[ElementLayer],
        x: u32,
        y: u32,
    ) -> Option<Self> {
        // iterate over each layer at the provided location, creating a vector of MapEntityType
        let mut entity_types: Vec<LayerPixelEntityType> = Vec::new();
        tile_layers.iter().for_each(|tile_layer| {
            tile_layer.located_tiles.iter().for_each(|tile| {
                if tile.0.x == x && tile.0.y == y {
                    entity_types.push(LayerPixelEntityType::Tile(Tile {
                        layer_index: tile_layer.layer_index,
                        tilemap_tile_index: tile.1,
                        tile_collision_kind: tile.2,
                    }));
                }
            });
        });
        element_layers.iter().for_each(|element_layer| {
            element_layer.located_elements.iter().for_each(|element| {
                // grab elements nearby this location
                if element.0.x as u32 == x && element.0.y as u32 == y {
                    // TODO determine which element this is in 'element' for custom spacing requirements
                    entity_types.push(LayerPixelEntityType::Element(Element {
                        layer_index: element_layer.layer_index,
                        element_meta_handle: element.1.clone(),
                        position: Vec2::new(element.0.x - x as f32, element.0.y - y as f32),
                    }));
                }
            });
        });
        if entity_types.is_empty() {
            None
        } else {
            Some(UngroupedPixel {
                layer_pixel_entity_types: entity_types,
            })
        }
    }
}

/// This can contain one or more ungrouped pixels, allowing for combining adjacent pixels into one grouped pixel
///     This is useful for when thick walls need to be considered as one-pixel-wide walls
struct GroupedPixel {
    ungrouped_pixel: UngroupedPixel,
    ungrouped_pixel_location: UVec2,
}

/// The pixel provided to the pixel board randomizer
struct PixelType {
    grouped_pixels: Vec<GroupedPixel>,
}

impl Pixel for PixelType {
    fn get_invalid_location_offsets_for_other_pixel(&self, _: &Self) -> Vec<(i16, i16)> {
        let invalid_location_offsets: Vec<(i16, i16)> = Vec::new();
        // TODO add invalid location offsets as needed
        self.grouped_pixels.iter().for_each(|gp| {
            gp.ungrouped_pixel
                .layer_pixel_entity_types
                .iter()
                .for_each(|lpet| {
                    match lpet {
                        LayerPixelEntityType::Tile(_) => {
                            // there are no restrictions on where the other pixel can exist
                        }
                        LayerPixelEntityType::Element(_) => {
                            // there may be restrictions in the future (as needed)
                        }
                    }
                });
        });
        invalid_location_offsets
    }
}
