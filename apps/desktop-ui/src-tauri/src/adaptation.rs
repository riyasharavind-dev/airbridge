#[derive(Default)]
pub struct Adaptation {
    level: usize,
    pressure: u32,
    stable: u32,
}
impl Adaptation {
    pub fn percent(&self) -> u32 {
        [100, 85, 70, 55, 40][self.level]
    }
    pub fn observe(&mut self, overloaded: bool) -> bool {
        let before = self.level;
        if overloaded {
            self.stable = 0;
            self.pressure += 1;
            if self.pressure >= 4 {
                self.level = (self.level + 1).min(4);
                self.pressure = 0;
            }
        } else {
            self.pressure = 0;
            self.stable += 1;
            if self.stable >= 180 {
                self.level = self.level.saturating_sub(1);
                self.stable = 0;
            }
        }
        before != self.level
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reduces_under_sustained_pressure_and_recovers_slowly() {
        let mut a = Adaptation::default();
        for _ in 0..3 {
            assert!(!a.observe(true));
        }
        assert!(a.observe(true));
        assert_eq!(a.percent(), 85);
        for _ in 0..179 {
            assert!(!a.observe(false));
        }
        assert!(a.observe(false));
        assert_eq!(a.percent(), 100);
    }
    #[test]
    fn bounded_and_resists_isolated_slow_frames() {
        let mut a = Adaptation::default();
        for _ in 0..10 {
            a.observe(true);
            a.observe(false);
        }
        assert_eq!(a.percent(), 100);
        for _ in 0..100 {
            a.observe(true);
        }
        assert_eq!(a.percent(), 40);
    }
}
