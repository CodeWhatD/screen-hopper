#[derive(Clone, Debug, PartialEq)]
pub struct MonitorGeom {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

/// Promote `target` to (0,0); every other monitor keeps its relative offset.
pub fn compute_layout(monitors: &[MonitorGeom], target: usize) -> Result<Vec<Placement>, String> {
    let t = monitors
        .iter()
        .find(|m| m.index == target)
        .ok_or_else(|| format!("monitor index {target} not found"))?;
    let (dx, dy) = (t.x, t.y);
    Ok(monitors
        .iter()
        .map(|m| Placement {
            index: m.index,
            x: m.x - dx,
            y: m.y - dy,
            is_primary: m.index == target,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geom(index: usize, x: i32, y: i32) -> MonitorGeom {
        MonitorGeom { index, x, y, width: 1920, height: 1080 }
    }

    #[test]
    fn promotes_target_to_origin_and_shifts_others() {
        let mons = vec![geom(0, 0, 0), geom(1, 1920, 0), geom(2, 3840, 0)];
        let out = compute_layout(&mons, 1).unwrap();
        assert_eq!(out, vec![
            Placement { index: 0, x: -1920, y: 0, is_primary: false },
            Placement { index: 1, x: 0,     y: 0, is_primary: true  },
            Placement { index: 2, x: 1920,  y: 0, is_primary: false },
        ]);
    }

    #[test]
    fn target_already_primary_is_unchanged() {
        let mons = vec![geom(0, 0, 0), geom(1, 1920, 0)];
        let out = compute_layout(&mons, 0).unwrap();
        assert_eq!(out[0], Placement { index: 0, x: 0, y: 0, is_primary: true });
        assert_eq!(out[1], Placement { index: 1, x: 1920, y: 0, is_primary: false });
    }

    #[test]
    fn handles_negative_and_vertical_offsets() {
        let mons = vec![geom(0, -1920, -200), geom(1, 0, 0)];
        let out = compute_layout(&mons, 0).unwrap();
        assert_eq!(out, vec![
            Placement { index: 0, x: 0,    y: 0,   is_primary: true  },
            Placement { index: 1, x: 1920, y: 200, is_primary: false },
        ]);
    }

    #[test]
    fn unknown_target_errors() {
        let mons = vec![geom(0, 0, 0)];
        assert!(compute_layout(&mons, 9).is_err());
    }
}
