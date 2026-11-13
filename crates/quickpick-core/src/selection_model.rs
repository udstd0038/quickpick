use crate::region::{MonitorPx, PointDip, RectDip, RegionRectPx};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}
impl Handle {
    pub const ALL: [Self; 8] = [
        Self::Nw,
        Self::N,
        Self::Ne,
        Self::E,
        Self::Se,
        Self::S,
        Self::Sw,
        Self::W,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Handle(Handle),
    Move,
    Empty,
}
#[derive(Clone, Copy)]
struct Drag {
    start: PointDip,
    before: Option<RectDip>,
    hit: Hit,
}
pub struct SelectionModel {
    pub selection: Option<RectDip>,
    width: f64,
    height: f64,
    drag: Option<Drag>,
    submitted: bool,
}
impl SelectionModel {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            selection: None,
            width,
            height,
            drag: None,
            submitted: false,
        }
    }
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }
    fn clamp(&self, p: PointDip) -> PointDip {
        PointDip {
            x: p.x.clamp(0., self.width),
            y: p.y.clamp(0., self.height),
        }
    }
    pub fn handles(&self) -> Vec<(Handle, PointDip)> {
        let Some(r) = self.selection else {
            return vec![];
        };
        let mx = (r.left + r.right) / 2.;
        let my = (r.top + r.bottom) / 2.;
        [
            (Handle::Nw, r.left, r.top),
            (Handle::N, mx, r.top),
            (Handle::Ne, r.right, r.top),
            (Handle::E, r.right, my),
            (Handle::Se, r.right, r.bottom),
            (Handle::S, mx, r.bottom),
            (Handle::Sw, r.left, r.bottom),
            (Handle::W, r.left, my),
        ]
        .into_iter()
        .filter(|(h, _, _)| {
            matches!(h, Handle::Nw | Handle::Ne | Handle::Se | Handle::Sw)
                || (r.right - r.left >= 32. && r.bottom - r.top >= 32.)
        })
        .map(|(h, x, y)| (h, self.clamp(PointDip { x, y })))
        .collect()
    }
    pub fn hit(&self, p: PointDip) -> Hit {
        if let Some((handle, _)) = self
            .handles()
            .into_iter()
            .filter(|(_, h)| (p.x - h.x).hypot(p.y - h.y) <= 8.)
            .min_by(|(_, a), (_, b)| {
                (p.x - a.x)
                    .hypot(p.y - a.y)
                    .total_cmp(&(p.x - b.x).hypot(p.y - b.y))
            })
        {
            return Hit::Handle(handle);
        }
        let Some(r) = self.selection else {
            return Hit::Empty;
        };
        if p.x >= r.left - 5. && p.x <= r.right + 5. && (p.y - r.top).abs() <= 5. {
            return Hit::Handle(Handle::N);
        }
        if p.x >= r.left - 5. && p.x <= r.right + 5. && (p.y - r.bottom).abs() <= 5. {
            return Hit::Handle(Handle::S);
        }
        if p.y >= r.top - 5. && p.y <= r.bottom + 5. && (p.x - r.left).abs() <= 5. {
            return Hit::Handle(Handle::W);
        }
        if p.y >= r.top - 5. && p.y <= r.bottom + 5. && (p.x - r.right).abs() <= 5. {
            return Hit::Handle(Handle::E);
        }
        if p.x >= r.left && p.x <= r.right && p.y >= r.top && p.y <= r.bottom {
            Hit::Move
        } else {
            Hit::Empty
        }
    }
    pub fn down(&mut self, p: PointDip) {
        if !self.submitted {
            self.drag = Some(Drag {
                start: self.clamp(p),
                before: self.selection,
                hit: self.hit(p),
            });
        }
    }
    pub fn move_to(&mut self, p: PointDip) {
        let Some(d) = self.drag else { return };
        let p = self.clamp(p);
        if (p.x - d.start.x).hypot(p.y - d.start.y) <= 3. {
            self.selection = d.before;
            return;
        }
        self.selection = match (d.hit, d.before) {
            (Hit::Move, Some(r)) => {
                let dx = (p.x - d.start.x).clamp(-r.left, self.width - r.right);
                let dy = (p.y - d.start.y).clamp(-r.top, self.height - r.bottom);
                Some(RectDip {
                    left: r.left + dx,
                    right: r.right + dx,
                    top: r.top + dy,
                    bottom: r.bottom + dy,
                })
            }
            (Hit::Handle(h), Some(r)) => {
                let (mut a, mut b) = (
                    PointDip {
                        x: r.left,
                        y: r.top,
                    },
                    PointDip {
                        x: r.right,
                        y: r.bottom,
                    },
                );
                if matches!(h, Handle::Nw | Handle::W | Handle::Sw) {
                    a.x = p.x;
                }
                if matches!(h, Handle::Ne | Handle::E | Handle::Se) {
                    b.x = p.x;
                }
                if matches!(h, Handle::Nw | Handle::N | Handle::Ne) {
                    a.y = p.y;
                }
                if matches!(h, Handle::Sw | Handle::S | Handle::Se) {
                    b.y = p.y;
                }
                RectDip::between(a, b)
            }
            _ => RectDip::between(d.start, p),
        };
    }
    pub fn up(&mut self, p: PointDip, monitor: MonitorPx, scale: f64) {
        self.move_to(p);
        if let Some(d) = self.drag.take()
            && self
                .selection
                .and_then(|r| r.to_physical(monitor, scale))
                .is_none()
        {
            self.selection = d.before;
        }
    }
    pub fn lost_capture(&mut self) {
        if let Some(d) = self.drag.take() {
            self.selection = d.before;
        }
    }
    pub fn submit(&mut self, monitor: MonitorPx, scale: f64) -> Option<RegionRectPx> {
        if self.dragging() || self.submitted {
            return None;
        }
        let region = self.selection?.to_physical(monitor, scale)?;
        self.submitted = true;
        Some(region)
    }
    pub fn cancel(&mut self) {
        self.selection = None;
        self.drag = None;
        self.submitted = true;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const MON: MonitorPx = MonitorPx {
        x: -100,
        y: 0,
        width: 1000,
        height: 1000,
    };
    fn model() -> SelectionModel {
        let mut m = SelectionModel::new(1000., 1000.);
        m.selection = Some(RectDip {
            left: 100.,
            top: 100.,
            right: 300.,
            bottom: 300.,
        });
        m
    }
    #[test]
    fn all_handles_resize_and_cross_anchors() {
        for h in Handle::ALL {
            let mut m = model();
            let p = m.handles().into_iter().find(|(v, _)| *v == h).unwrap().1;
            m.down(p);
            m.move_to(PointDip { x: 400., y: 400. });
            let r = m.selection.unwrap();
            assert!(r.left <= r.right && r.top <= r.bottom);
            m.lost_capture();
            assert_eq!(m.selection, model().selection);
        }
    }
    #[test]
    fn movement_preserves_size_at_boundaries() {
        for p in [
            PointDip { x: -50., y: -30. },
            PointDip { x: 2000., y: 2000. },
        ] {
            let mut m = model();
            m.down(PointDip { x: 200., y: 200. });
            m.move_to(p);
            let r = m.selection.unwrap();
            assert_eq!(r.right - r.left, 200.);
            assert_eq!(r.bottom - r.top, 200.);
            assert!(r.left >= 0. && r.top >= 0. && r.right <= 1000. && r.bottom <= 1000.);
        }
    }
    #[test]
    fn single_submission_and_cancellation() {
        let mut m = model();
        assert!(m.submit(MON, 1.).is_some());
        assert!(m.submit(MON, 1.).is_none());
        m.cancel();
        assert!(m.selection.is_none());
    }
    #[test]
    fn zero_click_does_not_create_a_screen_selection() {
        let mut m = SelectionModel::new(1000., 1000.);
        let p = PointDip { x: 10., y: 10. };
        m.down(p);
        m.up(p, MON, 1.);
        assert!(m.submit(MON, 1.).is_none());
    }
    #[test]
    fn corners_win_over_overlapping_edges() {
        let m = model();
        assert_eq!(
            m.hit(PointDip { x: 101., y: 101. }),
            Hit::Handle(Handle::Nw)
        );
    }
}
