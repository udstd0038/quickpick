/// 显示器上的逻辑坐标；不能与全局物理坐标混用。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointDip {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectDip {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// 经验证的全局物理矩形。字段只读，避免构造未验证的捕获区域。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionRectPx {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl RegionRectPx {
    pub fn x(self) -> i32 {
        self.x
    }
    pub fn y(self) -> i32 {
        self.y
    }
    pub fn width(self) -> u32 {
        self.width
    }
    pub fn height(self) -> u32 {
        self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorPx {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl RectDip {
    pub fn between(a: PointDip, b: PointDip) -> Option<Self> {
        [a.x, a.y, b.x, b.y]
            .into_iter()
            .all(f64::is_finite)
            .then_some(Self {
                left: a.x.min(b.x),
                top: a.y.min(b.y),
                right: a.x.max(b.x),
                bottom: a.y.max(b.y),
            })
    }

    /// 在提交边界统一取整并限制到显示器内；负全局原点保留符号。
    pub fn to_physical(self, monitor: MonitorPx, scale: f64) -> Option<RegionRectPx> {
        if !scale.is_finite()
            || scale <= 0.0
            || monitor.width == 0
            || monitor.height == 0
            || ![self.left, self.top, self.right, self.bottom]
                .into_iter()
                .all(f64::is_finite)
            || self.right < self.left
            || self.bottom < self.top
        {
            return None;
        }
        let left = (self.left * scale)
            .floor()
            .clamp(0.0, f64::from(monitor.width)) as u32;
        let top = (self.top * scale)
            .floor()
            .clamp(0.0, f64::from(monitor.height)) as u32;
        let right = (self.right * scale)
            .ceil()
            .clamp(0.0, f64::from(monitor.width)) as u32;
        let bottom = (self.bottom * scale)
            .ceil()
            .clamp(0.0, f64::from(monitor.height)) as u32;
        let width = right.checked_sub(left)?;
        let height = bottom.checked_sub(top)?;
        if width < 8 || height < 8 {
            return None;
        }
        let x = i32::try_from(i64::from(monitor.x) + i64::from(left)).ok()?;
        let y = i32::try_from(i64::from(monitor.y) + i64::from(top)).ok()?;
        i32::try_from(i64::from(x) + i64::from(width)).ok()?;
        i32::try_from(i64::from(y) + i64::from(height)).ok()?;
        Some(RegionRectPx {
            x,
            y,
            width,
            height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MONITOR: MonitorPx = MonitorPx {
        x: -1920,
        y: -200,
        width: 1920,
        height: 1080,
    };

    #[test]
    fn all_drawing_directions_normalize() {
        for (a, b) in [
            (PointDip { x: 1.0, y: 2.0 }, PointDip { x: 20.0, y: 30.0 }),
            (PointDip { x: 20.0, y: 2.0 }, PointDip { x: 1.0, y: 30.0 }),
        ] {
            for (a, b) in [(a, b), (b, a)] {
                assert_eq!(
                    RectDip::between(a, b),
                    Some(RectDip {
                        left: 1.0,
                        top: 2.0,
                        right: 20.0,
                        bottom: 30.0
                    })
                );
            }
        }
    }

    #[test]
    fn fractional_dpi_rounds_outwards_with_negative_origin() {
        let rect = RectDip {
            left: 1.1,
            top: 2.1,
            right: 10.1,
            bottom: 12.1,
        };
        assert_eq!(
            rect.to_physical(MONITOR, 1.25),
            Some(RegionRectPx {
                x: -1919,
                y: -198,
                width: 12,
                height: 14
            })
        );
    }

    #[test]
    fn clips_and_rejects_small_or_invalid_regions() {
        let rect = RectDip {
            left: -100.0,
            top: -100.0,
            right: 4000.0,
            bottom: 4000.0,
        };
        assert_eq!(
            rect.to_physical(MONITOR, 2.0),
            Some(RegionRectPx {
                x: -1920,
                y: -200,
                width: 1920,
                height: 1080
            })
        );
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(rect.to_physical(MONITOR, scale).is_none());
        }
        for edge in [0.0, 7.0, f64::NAN, f64::INFINITY] {
            assert!(
                RectDip {
                    left: 0.0,
                    top: 0.0,
                    right: edge,
                    bottom: edge
                }
                .to_physical(MONITOR, 1.0)
                .is_none()
            );
        }
    }

    #[test]
    fn valid_regions_stay_inside_monitor() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            for x in [-3000.1, -1.1, 0.0, 7.1, 100.3, 1900.5, 4000.0] {
                for y in [-2000.0, 0.0, 12.3, 1000.1, 3000.0] {
                    let rect = RectDip::between(
                        PointDip { x, y },
                        PointDip {
                            x: x + 41.7,
                            y: y + 29.9,
                        },
                    )
                    .unwrap();
                    if let Some(r) = rect.to_physical(MONITOR, scale) {
                        assert!(r.x >= MONITOR.x && r.y >= MONITOR.y);
                        assert!(
                            i64::from(r.x) + i64::from(r.width)
                                <= i64::from(MONITOR.x) + i64::from(MONITOR.width)
                        );
                        assert!(
                            i64::from(r.y) + i64::from(r.height)
                                <= i64::from(MONITOR.y) + i64::from(MONITOR.height)
                        );
                        assert!(r.width >= 8 && r.height >= 8);
                    }
                }
            }
        }
    }
}
