#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiScope {
    Text,
    Vision,
    Input,
}
impl AiScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Vision => "vision",
            Self::Input => "input",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenshotAction {
    Copy,
    Extract,
    Translate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId(u64);
#[derive(Default)]
pub struct Session {
    generation: u64,
    pending: bool,
}
impl Session {
    pub fn begin(&mut self) -> RequestId {
        self.generation = self.generation.wrapping_add(1);
        self.pending = true;
        RequestId(self.generation)
    }
    pub fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = false;
    }
    pub fn accepts(&self, id: RequestId) -> bool {
        self.pending && self.generation == id.0
    }
    pub fn finish(&mut self, id: RequestId) -> bool {
        if self.accepts(id) {
            self.pending = false;
            true
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn late_and_duplicate_results_are_rejected() {
        let mut s = Session::default();
        let a = s.begin();
        s.cancel();
        assert!(!s.accepts(a));
        let b = s.begin();
        let c = s.begin();
        assert!(!s.finish(b));
        assert!(s.finish(c));
        assert!(!s.finish(c));
    }
}
