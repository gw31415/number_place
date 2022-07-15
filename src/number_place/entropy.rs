use super::value::*;
use std::collections::HashSet;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_len_disable() {
        let mut entropy = Entropy::new();
        assert_eq!(entropy.len(), 9);
        entropy.disable(&Value::ONE).unwrap();
        assert_eq!(entropy.len(), 8);
        entropy.disable(&Value::ONE).unwrap();
        assert_eq!(entropy.len(), 8);
    }
    #[test]
    fn new_converged() {
        let entropy = Entropy::new_converged(Value::new(8).unwrap());
        assert_eq!(entropy.len(), 1);
    }
    #[test]
    fn new_try_converge() {
        let mut entropy = Entropy::new();
        entropy.try_converge(&Value::ONE).unwrap();
        assert_eq!(entropy.len(), 1);
        let mut entropy = Entropy::new();
        entropy.disable(&Value::ONE).unwrap();
        entropy.try_converge(&Value::ONE).unwrap_err();
        entropy.try_converge(&Value::TWO).unwrap();
        assert_eq!(entropy.len(), 1);
    }
}

/// そのセルでの値の可能性を表します。
#[derive(Clone, Debug)]
pub struct Entropy(Option<HashSet<Value>>);
impl Entropy {
    /// エントロピーの大きさを返します。
    pub fn len(&self) -> usize {
        match &self.0 {
            Some(s) => s.len(),
            None => 9,
        }
    }
    /// 全く収束していない新しいエントロピーを返します。
    pub fn new() -> Self {
        Default::default()
    }

    /// 新しい収束済みのエントロピーを返します。
    pub fn new_converged(value: Value) -> Self {
        Entropy(Some(HashSet::from([value])))
    }

    /// 指定された値に収束させます。
    pub fn try_converge(&mut self, value: &Value) -> Result<HashSet<Value>, EntropyConflictError> {
        if !self.is_possible(value) {
            Err(EntropyConflictError(Some((value.to_owned(), self.clone()))))
        } else {
            let mut others = HashSet::with_capacity(self.len() - 1);
            match &self.0 {
                Some(s) => {
                    for added_value in s.iter() {
                        if added_value != value {
                            others.insert(added_value.to_owned());
                        }
                    }
                }
                None => {
                    for value_index in 1u8..=9 {
                        let added_value = unsafe { Value::new_unchecked(value_index) };
                        if &added_value != value {
                            others.insert(added_value.to_owned());
                        }
                    }
                }
            }
            *self = Entropy::new_converged(value.clone());
            Ok(others)
        }
    }

    /// その値になる可能性があるかどうかを返します。
    pub fn is_possible(&self, value: &Value) -> bool {
        match &self.0 {
            Some(s) => s.contains(value),
            None => true,
        }
    }

    /// 確率が収束しているかどうかを返します。
    pub fn is_converged(&self) -> bool {
        match &self.0 {
            Some(s) => s.len() == 1,
            None => false,
        }
    }

    /// 確率が収束している場合、その値を返します。
    pub fn check_convergence(&self) -> Option<Value> {
        if self.is_converged() {
            let hash_set = unsafe {
                self.0
                    .as_ref()
                    .unwrap_unchecked()
                    .iter()
                    .next()
                    .unwrap_unchecked()
                    .to_owned()
            };
            Some(hash_set)
        } else {
            None
        }
    }

    /// 指定された値の可能性を否定します。
    pub fn disable(&mut self, value: &Value) -> Result<bool, EntropyConflictError> {
        match &self.0 {
            Some(_) => {
                if self.len() == 1 {
                    if self.is_possible(value) {
                        return Err(EntropyConflictError(None));
                    }
                    return Ok(false);
                }
                Ok(unsafe { self.0.as_mut().unwrap_unchecked() }.remove(value))
            }
            None => {
                let mut base = HashSet::with_capacity(8);
                for i in 1..=9 {
                    if i != value.clone().into() {
                        base.insert(unsafe { Value::new_unchecked(i) });
                    }
                }
                self.0 = Some(base);
                Ok(true)
            }
        }
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Entropy(None)
    }
}

impl std::fmt::Display for Entropy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.to_owned() {
            Some(sorted_values) => {
                write!(f, "[")?;
                let mut sorted_values = sorted_values.iter().collect::<Vec<_>>();
                sorted_values.sort();
                for c in sorted_values {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")
            }
            None => {
                write!(f, "[123456789]")
            }
        }
    }
}

impl IntoIterator for Entropy {
    type Item = Value;
    type IntoIter = std::collections::hash_set::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(s) => s.into_iter(),
            None => {
                let mut hash_set = HashSet::with_capacity(9);
                for i in 1..=9 {
                    hash_set.insert(unsafe { Value::new_unchecked(i) });
                }
                hash_set.into_iter()
            }
        }
    }
}

/// 値とエントロピーが競合した際のエラーです。
// 競合した値とエントロピーのペアが存在する場合はSome(Value, Entropy)の形式で指定。
// disableの結果、存在するペアがなくなってしまった場合はNoneで指定。
#[derive(Debug)]
pub struct EntropyConflictError(Option<(Value, Entropy)>);

impl std::fmt::Display for EntropyConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some((value, entropy)) => write!(f, "{} <- {}", entropy, value),
            None => write!(f, "[]"),
        }
    }
}
