use crate::window_state::WindowGeometry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Physical screen coordinates; corners take precedence over the narrow edge band.
pub fn resize_edge(
    frame: WindowGeometry,
    screen_x: i32,
    screen_y: i32,
    scale: f32,
) -> Option<FrameEdge> {
    if !scale.is_finite() || scale <= 0. || frame.width < 2 || frame.height < 2 {
        return None;
    }
    let x = (i64::from(screen_x) - i64::from(frame.x)) as f64;
    let y = (i64::from(screen_y) - i64::from(frame.y)) as f64;
    let width = f64::from(frame.width);
    let height = f64::from(frame.height);
    if x < 0. || y < 0. || x >= width || y >= height {
        return None;
    }
    let edge = (8. * f64::from(scale)).max(1.).min(width.min(height) / 2.);
    let corner = (14. * f64::from(scale))
        .max(edge)
        .min(width.min(height) / 2.);
    match (
        x < corner,
        x >= width - corner,
        y < corner,
        y >= height - corner,
    ) {
        (true, _, true, _) => return Some(FrameEdge::TopLeft),
        (_, true, true, _) => return Some(FrameEdge::TopRight),
        (true, _, _, true) => return Some(FrameEdge::BottomLeft),
        (_, true, _, true) => return Some(FrameEdge::BottomRight),
        _ => {}
    }
    if x < edge {
        Some(FrameEdge::Left)
    } else if x >= width - edge {
        Some(FrameEdge::Right)
    } else if y < edge {
        Some(FrameEdge::Top)
    } else if y >= height - edge {
        Some(FrameEdge::Bottom)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_eight_grips_work_at_supported_scales_and_negative_origins() {
        for scale in [1., 1.25, 1.5, 2.] {
            let frame = WindowGeometry {
                x: -1920,
                y: -1080,
                width: (420. * scale) as u32,
                height: (600. * scale) as u32,
            };
            let w = frame.width as i32;
            let h = frame.height as i32;
            for (x, y, expected) in [
                (1, 1, FrameEdge::TopLeft),
                (w - 2, 1, FrameEdge::TopRight),
                (1, h - 2, FrameEdge::BottomLeft),
                (w - 2, h - 2, FrameEdge::BottomRight),
                (1, h / 2, FrameEdge::Left),
                (w - 2, h / 2, FrameEdge::Right),
                (w / 2, 1, FrameEdge::Top),
                (w / 2, h - 2, FrameEdge::Bottom),
            ] {
                assert_eq!(
                    resize_edge(frame, frame.x + x, frame.y + y, scale),
                    Some(expected)
                );
            }
            assert_eq!(
                resize_edge(frame, frame.x + w / 2, frame.y + h / 2, scale),
                None
            );
        }
    }
    #[test]
    fn corners_extend_inside_the_rounded_client_area() {
        let frame = WindowGeometry {
            x: 0,
            y: 0,
            width: 420,
            height: 600,
        };
        assert_eq!(resize_edge(frame, 10, 10, 1.), Some(FrameEdge::TopLeft));
        assert_eq!(
            resize_edge(frame, 410, 590, 1.),
            Some(FrameEdge::BottomRight)
        );
        assert_eq!(resize_edge(frame, 20, 20, 1.), None);
    }
    #[test]
    fn outside_points_and_invalid_geometry_do_not_claim_controls() {
        let frame = WindowGeometry {
            x: -10,
            y: -10,
            width: 420,
            height: 600,
        };
        for (x, y) in [(-11, 0), (0, -11), (410, 0), (0, 590)] {
            assert_eq!(resize_edge(frame, x, y, 1.), None);
        }
        for scale in [0., -1., f32::NAN, f32::INFINITY] {
            assert_eq!(resize_edge(frame, 0, 0, scale), None);
        }
        assert_eq!(
            resize_edge(WindowGeometry { width: 0, ..frame }, 0, 0, 1.),
            None
        );
    }
}
