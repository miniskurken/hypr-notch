// filepath: src/layout.rs
use crate::config::{ModuleStateConfig, NotchConfig};
use crate::module::{Module, Rect};
use std::collections::HashMap;

/// Represents the calculated area for each module
#[derive(Debug, Clone)]
pub struct ModuleLayout {
    pub areas: HashMap<String, Rect>,
}

pub fn calculate_module_layout(
    config: &NotchConfig,
    modules: &[Box<dyn Module>],
    expanded: bool,
) -> ModuleLayout {
    let layout_state = if expanded {
        &config.layout.expanded
    } else {
        &config.layout.collapsed
    };

    let style = config.style_for(expanded);

    // Notch avoidance: use [notch] section if present, fallback to main style
    let notch_x =
        config.main.width.unwrap_or(style.width) / 2 - config.main.width.unwrap_or(style.width) / 6; // Example: notch at center
    let notch_width = config.main.width.unwrap_or(style.width) / 3; // Example: notch width
    let notch_rect = Rect {
        x: notch_x as i32,
        y: 0,
        width: notch_width,
        height: config.main.height.unwrap_or(style.height),
    };

    let mut areas = HashMap::new();
    let mut y_offset = 0i32;
    let row_spacing = layout_state.row_spacing.unwrap_or(8);

    let total_width = style.width as i32;

    for row in &layout_state.rows {
        let default_cfg = ModuleStateConfig::default();

        // Group modules by per-module alignment
        let mut left_modules = Vec::new();
        let mut center_modules = Vec::new();
        let mut right_modules = Vec::new();

        for id in &row.modules {
            let state_cfg = config
                .modules
                .state
                .get(id)
                .map(|s| if expanded { &s.expanded } else { &s.collapsed })
                .unwrap_or(&default_cfg);
            let visible = state_cfg.visible.unwrap_or(true);
            let alignment = state_cfg.alignment.as_deref().unwrap_or("center");
            if !visible {
                continue;
            }
            if let Some(m) = modules.iter().find(|m| m.id() == id) {
                match alignment {
                    "left" => left_modules.push(m),
                    "right" => right_modules.push(m),
                    _ => center_modules.push(m),
                }
            }
        }

        // Place left modules from left to right
        let mut x_offset = 0;
        for m in &left_modules {
            let (w, h) = m.preferred_size();
            let area = Rect {
                x: x_offset,
                y: y_offset,
                width: w,
                height: h,
            };
            println!(
                "Placing module '{}' at {:?} (alignment: left)",
                m.id(),
                area
            );
            areas.insert(m.id().to_string(), area);
            x_offset += w as i32 + 8;
        }

        // Place right modules from right to left
        let mut x_offset = total_width;
        for m in right_modules.iter().rev() {
            let (w, h) = m.preferred_size();
            x_offset -= w as i32;
            let area = Rect {
                x: x_offset,
                y: y_offset,
                width: w,
                height: h,
            };
            println!(
                "Placing module '{}' at {:?} (alignment: right)",
                m.id(),
                area
            );
            areas.insert(m.id().to_string(), area);
            x_offset -= 8;
        }

        // Place center modules centered in remaining space
        let center_total_width: u32 = center_modules.iter().map(|m| m.preferred_size().0).sum();
        let center_total_spacing = if center_modules.len() > 1 {
            (center_modules.len() as u32 - 1) * 8
        } else {
            0
        };
        let center_row_width = center_total_width + center_total_spacing;
        let mut x_offset = ((total_width - center_row_width as i32) / 2).max(0);
        for m in &center_modules {
            let (w, h) = m.preferred_size();
            let area = Rect {
                x: x_offset,
                y: y_offset,
                width: w,
                height: h,
            };
            println!(
                "Placing module '{}' at {:?} (alignment: center)",
                m.id(),
                area
            );
            areas.insert(m.id().to_string(), area);
            x_offset += w as i32 + 8;
        }

        y_offset += left_modules
            .iter()
            .chain(center_modules.iter())
            .chain(right_modules.iter())
            .map(|m| m.preferred_size().1)
            .max()
            .unwrap_or(0) as i32
            + row_spacing as i32;
    }

    ModuleLayout { areas }
}
