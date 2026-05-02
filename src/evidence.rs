use std::collections::BTreeMap;
use std::fmt;

use image::RgbaImage;

use crate::config::{EvidenceConfig, RegionMatcher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceState {
    Unknown,
    Clear,
    Selected,
    Rejected,
    Conflict,
}

impl fmt::Display for EvidenceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvidenceState::Unknown => write!(f, "unknown"),
            EvidenceState::Clear => write!(f, "clear"),
            EvidenceState::Selected => write!(f, "selected"),
            EvidenceState::Rejected => write!(f, "rejected"),
            EvidenceState::Conflict => write!(f, "conflict"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub fn evaluate(image: &RgbaImage, evidence: &[EvidenceConfig]) -> BTreeMap<String, EvidenceState> {
    evidence
        .iter()
        .map(|item| {
            let selected = matcher_active(image, &item.selected);
            let rejected = matcher_active(image, &item.rejected);
            (item.name.clone(), classify(selected, rejected))
        })
        .collect()
}

fn classify(selected: Option<bool>, rejected: Option<bool>) -> EvidenceState {
    match (selected, rejected) {
        (None, _) | (_, None) => EvidenceState::Unknown,
        (Some(true), Some(true)) => EvidenceState::Conflict,
        (Some(true), Some(false)) => EvidenceState::Selected,
        (Some(false), Some(true)) => EvidenceState::Rejected,
        (Some(false), Some(false)) => EvidenceState::Clear,
    }
}

fn matcher_active(image: &RgbaImage, matcher: &RegionMatcher) -> Option<bool> {
    let region = resolve_region(image.width(), image.height(), matcher)?;
    let tolerance = matcher.color.tolerance as i16;
    let mut total = 0u32;
    let mut matched = 0u32;

    for y in region.y..region.y.saturating_add(region.height) {
        for x in region.x..region.x.saturating_add(region.width) {
            let pixel = image.get_pixel(x, y);
            total += 1;

            let r_ok = (pixel[0] as i16 - matcher.color.r as i16).abs() <= tolerance;
            let g_ok = (pixel[1] as i16 - matcher.color.g as i16).abs() <= tolerance;
            let b_ok = (pixel[2] as i16 - matcher.color.b as i16).abs() <= tolerance;
            if r_ok && g_ok && b_ok {
                matched += 1;
            }
        }
    }

    if total == 0 {
        return Some(false);
    }

    Some(matched as f64 / total as f64 >= matcher.color.min_ratio)
}

fn resolve_region(width: u32, height: u32, matcher: &RegionMatcher) -> Option<PixelRegion> {
    if width == 0 || height == 0 || matcher.w_pct <= 0.0 || matcher.h_pct <= 0.0 {
        return None;
    }

    let x = (matcher.x_pct * width as f64).round() as u32;
    let y = (matcher.y_pct * height as f64).round() as u32;
    if x >= width || y >= height {
        return None;
    }

    let region_width = ((matcher.w_pct * width as f64).round() as u32).max(1);
    let region_height = ((matcher.h_pct * height as f64).round() as u32).max(1);

    Some(PixelRegion {
        x,
        y,
        width: region_width.min(width - x),
        height: region_height.min(height - y),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ColorMatcher, RegionMatcher};

    fn matcher(x_pct: f64, y_pct: f64, w_pct: f64, h_pct: f64) -> RegionMatcher {
        RegionMatcher {
            x_pct,
            y_pct,
            w_pct,
            h_pct,
            color: ColorMatcher {
                r: 0,
                g: 0,
                b: 0,
                tolerance: 0,
                min_ratio: 0.5,
            },
        }
    }

    #[test]
    fn classifies_evidence_state() {
        assert_eq!(classify(Some(true), Some(false)), EvidenceState::Selected);
        assert_eq!(classify(Some(false), Some(true)), EvidenceState::Rejected);
        assert_eq!(classify(Some(true), Some(true)), EvidenceState::Conflict);
        assert_eq!(classify(Some(false), Some(false)), EvidenceState::Clear);
        assert_eq!(classify(None, Some(false)), EvidenceState::Unknown);
    }

    #[test]
    fn resolves_normalized_region() {
        assert_eq!(
            resolve_region(1000, 500, &matcher(0.1, 0.2, 0.05, 0.1)),
            Some(PixelRegion {
                x: 100,
                y: 100,
                width: 50,
                height: 50,
            })
        );
    }

    #[test]
    fn rejects_empty_region() {
        assert_eq!(resolve_region(100, 100, &matcher(0.0, 0.0, 0.0, 0.1)), None);
    }
}
