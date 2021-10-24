use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAlignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

pub fn draw_aligned_text(
    text: &str,
    position: Vec2,
    ha: HorizontalAlignment,
    va: VerticalAlignment,
    params: TextParams,
) {
    let measure = measure_text(text, Some(params.font), params.font_size, params.font_scale);

    let x = match ha {
        HorizontalAlignment::Left => position.x,
        _ => {
            if ha == HorizontalAlignment::Center {
                position.x - (measure.width / 2.0)
            } else {
                position.x - measure.width
            }
        }
    };

    let y = match va {
        VerticalAlignment::Top => position.y + measure.height,
        VerticalAlignment::Center => position.y + measure.height / 2.0,
        _ => position.y,
    };

    draw_text_ex(text, x, y, params);
}
