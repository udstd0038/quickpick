pub trait HotkeyBackend<K> {
    fn register(&mut self, key: K) -> Result<(), String>;
    fn unregister(&mut self, key: K) -> Result<(), String>;
}
pub enum SwapError {
    Apply(String),
    Rollback,
}
/// 只改变集合差异；失败时恢复此前注册，无法恢复时明确报告而非声称成功。
pub fn replace<K: Copy + Eq, B: HotkeyBackend<K>>(
    old: &[K],
    new: &[K],
    backend: &mut B,
) -> Result<(), SwapError> {
    let mut removed = vec![];
    let mut added = vec![];
    let applied = (|| {
        for key in old {
            if !new.contains(key) {
                backend.unregister(*key)?;
                removed.push(*key);
            }
        }
        for key in new {
            if !old.contains(key) {
                backend.register(*key)?;
                added.push(*key);
            }
        }
        Ok::<(), String>(())
    })();
    if let Err(error) = applied {
        let mut failed = false;
        for key in added {
            failed |= backend.unregister(key).is_err();
        }
        for key in removed {
            failed |= backend.register(key).is_err();
        }
        return Err(if failed {
            SwapError::Rollback
        } else {
            SwapError::Apply(error)
        });
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    struct Fake {
        keys: BTreeSet<u8>,
        blocked: u8,
        changed: usize,
    }
    impl HotkeyBackend<u8> for Fake {
        fn register(&mut self, key: u8) -> Result<(), String> {
            if key == self.blocked {
                return Err("occupied".into());
            }
            self.keys.insert(key);
            self.changed += 1;
            Ok(())
        }
        fn unregister(&mut self, key: u8) -> Result<(), String> {
            self.keys.remove(&key);
            self.changed += 1;
            Ok(())
        }
    }
    #[test]
    fn occupied_replacement_restores_original_registration() {
        let mut f = Fake {
            keys: [1, 2].into(),
            blocked: 4,
            changed: 0,
        };
        assert!(replace(&[1, 2], &[3, 4], &mut f).is_err());
        assert_eq!(f.keys, [1, 2].into());
    }
    #[test]
    fn unchanged_keys_are_never_unregistered() {
        let mut f = Fake {
            keys: [1, 2].into(),
            blocked: 9,
            changed: 0,
        };
        assert!(replace(&[1, 2], &[1, 3], &mut f).is_ok());
        assert_eq!(f.keys, [1, 3].into());
        assert_eq!(f.changed, 2);
    }
}
