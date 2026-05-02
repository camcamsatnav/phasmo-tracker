use image::RgbaImage;

use crate::config::EvidenceConfig;

#[derive(Debug, Clone, Copy)]
struct Region {
    x_pct: f64,
    y_pct: f64,
    w_pct: f64,
    h_pct: f64,
}

pub fn evidence_page_visible(image: &RgbaImage, evidence: &[EvidenceConfig]) -> bool {
    paper_visible(image)
        && checkbox_count(image, evidence) >= required_checkbox_count(evidence)
        && ghost_grid_visible(image)
}

fn paper_visible(image: &RgbaImage) -> bool {
    let regions = [
        Region {
            x_pct: 0.30,
            y_pct: 0.31,
            w_pct: 0.06,
            h_pct: 0.03,
        },
        Region {
            x_pct: 0.33,
            y_pct: 0.52,
            w_pct: 0.06,
            h_pct: 0.03,
        },
        Region {
            x_pct: 0.72,
            y_pct: 0.52,
            w_pct: 0.08,
            h_pct: 0.03,
        },
    ];

    regions
        .iter()
        .filter(|region| light_paper_ratio(image, region) >= 0.55)
        .count()
        >= 2
}

fn checkbox_count(image: &RgbaImage, evidence: &[EvidenceConfig]) -> usize {
    evidence
        .iter()
        .filter(|item| {
            dark_ratio(image, &left_checkbox_border(item)) >= 0.10
                || dark_ratio(image, &top_checkbox_border(item)) >= 0.10
        })
        .count()
}

fn required_checkbox_count(evidence: &[EvidenceConfig]) -> usize {
    match evidence.len() {
        0 => usize::MAX,
        1..=5 => evidence.len(),
        _ => 6,
    }
}

fn ghost_grid_visible(image: &RgbaImage) -> bool {
    let columns = [0.61, 0.74, 0.87];
    let rows = [0.28, 0.36, 0.44, 0.52, 0.60, 0.68, 0.76, 0.84];
    let mut visible_cells = 0;

    for y_pct in rows {
        for x_pct in columns {
            let region = Region {
                x_pct,
                y_pct,
                w_pct: 0.075,
                h_pct: 0.032,
            };

            if ghost_name_ink_ratio(image, &region) >= 0.025 {
                visible_cells += 1;
            }
        }
    }

    visible_cells >= 10
}

fn left_checkbox_border(item: &EvidenceConfig) -> Region {
    Region {
        x_pct: (item.selected.x_pct - item.selected.w_pct * 0.42).max(0.0),
        y_pct: (item.selected.y_pct - item.selected.h_pct * 0.30).max(0.0),
        w_pct: item.selected.w_pct * 0.40,
        h_pct: item.selected.h_pct * 1.65,
    }
}

fn top_checkbox_border(item: &EvidenceConfig) -> Region {
    Region {
        x_pct: (item.selected.x_pct - item.selected.w_pct * 0.42).max(0.0),
        y_pct: (item.selected.y_pct - item.selected.h_pct * 0.30).max(0.0),
        w_pct: item.selected.w_pct * 1.55,
        h_pct: item.selected.h_pct * 0.32,
    }
}

fn light_paper_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        pixel[0] >= 145 && pixel[1] >= 135 && pixel[2] >= 95
    })
}

fn dark_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        pixel[0] <= 80 && pixel[1] <= 80 && pixel[2] <= 80
    })
}

fn ghost_name_ink_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        pixel[0] <= 175 && pixel[1] <= 170 && pixel[2] <= 145
    })
}

fn ratio(image: &RgbaImage, region: &Region, matches: impl Fn(&image::Rgba<u8>) -> bool) -> f64 {
    let Some((x, y, width, height)) = resolve_region(image, region) else {
        return 0.0;
    };

    let mut total = 0u32;
    let mut matched = 0u32;

    for py in y..y.saturating_add(height) {
        for px in x..x.saturating_add(width) {
            total += 1;
            if matches(image.get_pixel(px, py)) {
                matched += 1;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        matched as f64 / total as f64
    }
}

fn resolve_region(image: &RgbaImage, region: &Region) -> Option<(u32, u32, u32, u32)> {
    if image.width() == 0 || image.height() == 0 || region.w_pct <= 0.0 || region.h_pct <= 0.0 {
        return None;
    }

    let x = (region.x_pct * image.width() as f64).round() as u32;
    let y = (region.y_pct * image.height() as f64).round() as u32;
    if x >= image.width() || y >= image.height() {
        return None;
    }

    let width = ((region.w_pct * image.width() as f64).round() as u32).max(1);
    let height = ((region.h_pct * image.height() as f64).round() as u32).max(1);
    Some((
        x,
        y,
        width.min(image.width() - x),
        height.min(image.height() - y),
    ))
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;
    use crate::config::{ColorMatcher, EvidenceConfig, RegionMatcher};

    #[test]
    fn rejects_non_journal_frame() {
        let image = RgbaImage::from_pixel(1000, 1000, Rgba([15, 15, 15, 255]));
        assert!(!evidence_page_visible(&image, &evidence_items()));
    }

    #[test]
    fn rejects_paper_without_evidence_layout() {
        let image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        assert!(!evidence_page_visible(&image, &evidence_items()));
    }

    #[test]
    fn rejects_journal_page_with_checkboxes_but_without_ghost_grid() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        for item in &evidence {
            draw_region(
                &mut image,
                &left_checkbox_border(item),
                Rgba([5, 5, 5, 255]),
            );
            draw_region(&mut image, &top_checkbox_border(item), Rgba([5, 5, 5, 255]));
        }

        assert!(!evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn accepts_journal_evidence_page_shape() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_ghost_grid(&mut image);

        assert!(evidence_page_visible(&image, &evidence));
    }

    fn draw_evidence_checkboxes(image: &mut RgbaImage, evidence: &[EvidenceConfig]) {
        for item in evidence {
            draw_region(image, &left_checkbox_border(item), Rgba([5, 5, 5, 255]));
            draw_region(image, &top_checkbox_border(item), Rgba([5, 5, 5, 255]));
        }
    }

    fn draw_ghost_grid(image: &mut RgbaImage) {
        let columns = [0.61, 0.74, 0.87];
        let rows = [0.28, 0.36, 0.44, 0.52];

        for y_pct in rows {
            for x_pct in columns {
                draw_region(
                    image,
                    &Region {
                        x_pct,
                        y_pct,
                        w_pct: 0.075,
                        h_pct: 0.032,
                    },
                    Rgba([105, 100, 80, 255]),
                );
            }
        }
    }

    fn evidence_items() -> Vec<EvidenceConfig> {
        (0..7)
            .map(|index| EvidenceConfig {
                name: format!("Evidence {index}"),
                selected: region_matcher(0.231, 0.235 + index as f64 * 0.098, 0.008, 0.017),
                rejected: region_matcher(0.244, 0.240 + index as f64 * 0.098, 0.006, 0.008),
            })
            .collect()
    }

    fn region_matcher(x_pct: f64, y_pct: f64, w_pct: f64, h_pct: f64) -> RegionMatcher {
        RegionMatcher {
            x_pct,
            y_pct,
            w_pct,
            h_pct,
            color: ColorMatcher {
                r: 10,
                g: 10,
                b: 10,
                tolerance: 55,
                min_ratio: 0.08,
            },
        }
    }

    fn draw_region(image: &mut RgbaImage, region: &Region, color: Rgba<u8>) {
        let Some((x, y, width, height)) = resolve_region(image, region) else {
            return;
        };

        for py in y..y.saturating_add(height) {
            for px in x..x.saturating_add(width) {
                image.put_pixel(px, py, color);
            }
        }
    }
}
