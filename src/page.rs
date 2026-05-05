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
    journal_spread_visible(image)
        && evidence_page_markers_visible(image)
        && checkbox_count(image, evidence) >= required_checkbox_count(evidence)
        && evidence_label_count(image, evidence) >= required_evidence_label_count(evidence)
        && ghost_grid_visible(image)
}

fn journal_spread_visible(image: &RgbaImage) -> bool {
    paper_visible(image) && yellow_paper_coverage_visible(image)
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

fn yellow_paper_coverage_visible(image: &RgbaImage) -> bool {
    let x_positions = [0.12, 0.35, 0.55, 0.83];
    let y_positions = [0.22, 0.50, 0.78];
    let mut yellow_regions = 0;
    let mut total_regions = 0;

    for y_pct in y_positions {
        for x_pct in x_positions {
            total_regions += 1;
            let region = Region {
                x_pct,
                y_pct,
                w_pct: 0.035,
                h_pct: 0.040,
            };

            if yellow_paper_ratio(image, &region) >= 0.45 {
                yellow_regions += 1;
            }
        }
    }

    yellow_regions >= total_regions - 2
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

fn evidence_label_count(image: &RgbaImage, evidence: &[EvidenceConfig]) -> usize {
    evidence
        .iter()
        .filter(|item| evidence_label_ink_ratio(image, &evidence_label_region(item)) >= 0.020)
        .count()
}

fn required_checkbox_count(evidence: &[EvidenceConfig]) -> usize {
    match evidence.len() {
        0 => usize::MAX,
        1..=5 => evidence.len(),
        _ => 6,
    }
}

fn required_evidence_label_count(evidence: &[EvidenceConfig]) -> usize {
    match evidence.len() {
        0 => usize::MAX,
        1..=5 => evidence.len(),
        _ => 6,
    }
}

fn ghost_grid_visible(image: &RgbaImage) -> bool {
    let columns = [0.55, 0.68, 0.80];
    let rows = [
        0.285, 0.352, 0.419, 0.486, 0.553, 0.620, 0.687, 0.754, 0.821,
    ];
    let mut visible_cells = 0;
    let mut visible_rows = 0;
    let mut visible_columns = [0usize; 3];

    for y_pct in rows {
        let mut row_cells = 0;
        for (column_index, x_pct) in columns.iter().copied().enumerate() {
            let region = Region {
                x_pct,
                y_pct,
                w_pct: 0.075,
                h_pct: 0.032,
            };

            if ghost_name_ink_ratio(image, &region) >= 0.025 {
                visible_cells += 1;
                row_cells += 1;
                visible_columns[column_index] += 1;
            }
        }

        if row_cells >= 2 {
            visible_rows += 1;
        }
    }

    visible_cells >= 6
        && visible_rows >= 3
        && visible_columns
            .iter()
            .filter(|visible_in_column| **visible_in_column >= 1)
            .count()
            >= 2
}

fn evidence_page_markers_visible(image: &RgbaImage) -> bool {
    evidence_words_visible(image) && right_page_prompt_visible(image)
}

fn evidence_words_visible(image: &RgbaImage) -> bool {
    let evidence_tab = Region {
        x_pct: 0.510,
        y_pct: 0.038,
        w_pct: 0.075,
        h_pct: 0.040,
    };
    let left_title = Region {
        x_pct: 0.105,
        y_pct: 0.122,
        w_pct: 0.145,
        h_pct: 0.055,
    };
    let right_title = Region {
        x_pct: 0.515,
        y_pct: 0.122,
        w_pct: 0.145,
        h_pct: 0.055,
    };
    let left_rule = Region {
        x_pct: 0.110,
        y_pct: 0.176,
        w_pct: 0.350,
        h_pct: 0.008,
    };
    let right_rule = Region {
        x_pct: 0.520,
        y_pct: 0.172,
        w_pct: 0.365,
        h_pct: 0.008,
    };

    let evidence_word_count = [evidence_tab, left_title, right_title]
        .iter()
        .filter(|region| title_ink_ratio(image, region) >= 0.030)
        .count();

    evidence_word_count >= 3
        && dark_ratio(image, &left_rule) >= 0.30
        && dark_ratio(image, &right_rule) >= 0.30
}

fn right_page_prompt_visible(image: &RgbaImage) -> bool {
    let prompt = Region {
        x_pct: 0.540,
        y_pct: 0.195,
        w_pct: 0.335,
        h_pct: 0.060,
    };

    title_ink_ratio(image, &prompt) >= 0.015
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

fn evidence_label_region(item: &EvidenceConfig) -> Region {
    Region {
        x_pct: item.selected.x_pct + item.selected.w_pct * 3.0,
        y_pct: (item.selected.y_pct - item.selected.h_pct * 0.25).max(0.0),
        w_pct: 0.195,
        h_pct: item.selected.h_pct * 2.0,
    }
}

fn light_paper_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        pixel[0] >= 145 && pixel[1] >= 135 && pixel[2] >= 95
    })
}

fn yellow_paper_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        let red = pixel[0] as i16;
        let green = pixel[1] as i16;
        let blue = pixel[2] as i16;

        red >= 135
            && green >= 125
            && blue >= 80
            && red <= 245
            && green <= 235
            && blue <= 205
            && red >= blue + 25
            && green >= blue + 15
            && (red - green).abs() <= 45
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

fn title_ink_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        pixel[0] <= 95 && pixel[1] <= 90 && pixel[2] <= 75
    })
}

fn evidence_label_ink_ratio(image: &RgbaImage, region: &Region) -> f64 {
    ratio(image, region, |pixel| {
        pixel[0] <= 115 && pixel[1] <= 110 && pixel[2] <= 95
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
    fn rejects_evidence_layout_without_yellow_journal_spread() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 190, 190, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_evidence_labels(&mut image, &evidence);
        draw_evidence_page_markers(&mut image);
        draw_ghost_grid(&mut image);

        assert!(!evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn rejects_journal_page_with_checkboxes_but_without_evidence_markers() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_evidence_labels(&mut image, &evidence);
        draw_ghost_grid(&mut image);

        assert!(!evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn rejects_journal_page_with_evidence_markers_but_without_ghost_grid() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_evidence_labels(&mut image, &evidence);
        draw_evidence_page_markers(&mut image);

        assert!(!evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn rejects_journal_page_without_evidence_tab_marker() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_evidence_labels(&mut image, &evidence);
        draw_evidence_page_body_markers(&mut image);
        draw_ghost_grid(&mut image);

        assert!(!evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn rejects_journal_page_without_evidence_label_rows() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_evidence_page_markers(&mut image);
        draw_ghost_grid(&mut image);

        assert!(!evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn accepts_journal_evidence_page_shape() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        draw_evidence_checkboxes(&mut image, &evidence);
        draw_evidence_labels(&mut image, &evidence);
        draw_evidence_page_markers(&mut image);
        draw_ghost_grid(&mut image);

        assert!(evidence_page_visible(&image, &evidence));
    }

    #[test]
    fn rejects_compact_layout_with_current_evidence_map() {
        let mut image = RgbaImage::from_pixel(1000, 1000, Rgba([190, 180, 130, 255]));
        let evidence = evidence_items();

        let old_evidence = old_evidence_items();
        draw_old_evidence_checkboxes(&mut image, &old_evidence);
        draw_old_evidence_labels(&mut image, &old_evidence);
        draw_old_evidence_page_markers(&mut image);
        draw_old_ghost_grid(&mut image);

        assert!(!evidence_page_visible(&image, &evidence));
    }

    fn draw_evidence_checkboxes(image: &mut RgbaImage, evidence: &[EvidenceConfig]) {
        for item in evidence {
            draw_region(image, &left_checkbox_border(item), Rgba([5, 5, 5, 255]));
            draw_region(image, &top_checkbox_border(item), Rgba([5, 5, 5, 255]));
        }
    }

    fn draw_evidence_labels(image: &mut RgbaImage, evidence: &[EvidenceConfig]) {
        for item in evidence {
            let label = evidence_label_region(item);
            draw_region(
                image,
                &Region {
                    x_pct: label.x_pct,
                    y_pct: label.y_pct + label.h_pct * 0.35,
                    w_pct: label.w_pct * 0.45,
                    h_pct: label.h_pct * 0.12,
                },
                Rgba([5, 5, 5, 255]),
            );
        }
    }

    fn draw_evidence_page_markers(image: &mut RgbaImage) {
        draw_region(
            image,
            &Region {
                x_pct: 0.510,
                y_pct: 0.038,
                w_pct: 0.075,
                h_pct: 0.040,
            },
            Rgba([5, 5, 5, 255]),
        );
        draw_evidence_page_body_markers(image);
    }

    fn draw_evidence_page_body_markers(image: &mut RgbaImage) {
        let regions = [
            Region {
                x_pct: 0.105,
                y_pct: 0.122,
                w_pct: 0.145,
                h_pct: 0.055,
            },
            Region {
                x_pct: 0.515,
                y_pct: 0.122,
                w_pct: 0.145,
                h_pct: 0.055,
            },
            Region {
                x_pct: 0.110,
                y_pct: 0.176,
                w_pct: 0.350,
                h_pct: 0.008,
            },
            Region {
                x_pct: 0.520,
                y_pct: 0.172,
                w_pct: 0.365,
                h_pct: 0.008,
            },
            Region {
                x_pct: 0.540,
                y_pct: 0.195,
                w_pct: 0.335,
                h_pct: 0.060,
            },
        ];

        for region in regions {
            draw_region(image, &region, Rgba([5, 5, 5, 255]));
        }
    }

    fn draw_ghost_grid(image: &mut RgbaImage) {
        let columns = [0.55, 0.68, 0.80];
        let rows = [0.285, 0.352, 0.419, 0.486, 0.553, 0.620];

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
                selected: region_matcher(0.122, 0.210 + index as f64 * 0.0952, 0.012, 0.023),
                rejected: region_matcher(0.120, 0.223 + index as f64 * 0.0952, 0.270, 0.006),
            })
            .collect()
    }

    fn old_evidence_items() -> Vec<EvidenceConfig> {
        (0..7)
            .map(|index| EvidenceConfig {
                name: format!("Evidence {index}"),
                selected: region_matcher(0.231, 0.235 + index as f64 * 0.098, 0.008, 0.017),
                rejected: region_matcher(0.244, 0.240 + index as f64 * 0.098, 0.006, 0.008),
            })
            .collect()
    }

    fn draw_old_evidence_checkboxes(image: &mut RgbaImage, evidence: &[EvidenceConfig]) {
        for item in evidence {
            draw_region(image, &left_checkbox_border(item), Rgba([5, 5, 5, 255]));
            draw_region(image, &top_checkbox_border(item), Rgba([5, 5, 5, 255]));
        }
    }

    fn draw_old_evidence_labels(image: &mut RgbaImage, evidence: &[EvidenceConfig]) {
        for item in evidence {
            let label = evidence_label_region(item);
            draw_region(
                image,
                &Region {
                    x_pct: label.x_pct,
                    y_pct: label.y_pct + label.h_pct * 0.35,
                    w_pct: label.w_pct * 0.45,
                    h_pct: label.h_pct * 0.12,
                },
                Rgba([5, 5, 5, 255]),
            );
        }
    }

    fn draw_old_evidence_page_markers(image: &mut RgbaImage) {
        let regions = [
            Region {
                x_pct: 0.515,
                y_pct: 0.060,
                w_pct: 0.070,
                h_pct: 0.040,
            },
            Region {
                x_pct: 0.220,
                y_pct: 0.145,
                w_pct: 0.110,
                h_pct: 0.055,
            },
            Region {
                x_pct: 0.530,
                y_pct: 0.150,
                w_pct: 0.110,
                h_pct: 0.055,
            },
            Region {
                x_pct: 0.225,
                y_pct: 0.197,
                w_pct: 0.260,
                h_pct: 0.008,
            },
            Region {
                x_pct: 0.533,
                y_pct: 0.197,
                w_pct: 0.260,
                h_pct: 0.008,
            },
            Region {
                x_pct: 0.540,
                y_pct: 0.220,
                w_pct: 0.260,
                h_pct: 0.075,
            },
        ];

        for region in regions {
            draw_region(image, &region, Rgba([5, 5, 5, 255]));
        }
    }

    fn draw_old_ghost_grid(image: &mut RgbaImage) {
        let columns = [0.61, 0.74, 0.87];
        let rows = [0.28, 0.36, 0.44, 0.52, 0.60, 0.68];

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
