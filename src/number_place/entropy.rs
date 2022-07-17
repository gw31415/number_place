#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn len_disable() {
        let mut len = 9;
        let mut entropy = Entropy::new();
        assert_eq!(entropy.len(), len);
        for i in 1..9 {
            len -= 1;
            entropy.disable(&Value::new(i).unwrap()).unwrap();
            assert_eq!(entropy.len(), len);
            entropy.disable(&Value::new(i).unwrap()).unwrap();
            assert_eq!(entropy.len(), len);
        }
        entropy.disable(&Value::NINE).unwrap_err();
    }
    #[test]
    fn new_converged_is_converged_is_possible() {
        for i in 1..=9 {
            let value = &Value::new(i).unwrap();
            let entropy = Entropy::new_converged(value.clone());
            assert!(entropy.is_converged());
            assert_eq!(entropy.len(), 1);
            assert!(entropy.is_possible(value));
            for j in 1..=9 {
                if j != i {
                    assert!(!entropy.is_possible(&Value::new(j).unwrap()));
                }
            }
        }
    }
    #[test]
    fn new_try_converge() {
        for i in 1..=9 {
            let test_value = Value::new(i).unwrap();
            let mut entropy = Entropy::new();
            let rest = entropy.try_converge(&test_value).unwrap();
            assert_eq!(entropy.len(), 1);
            assert_eq!(rest.len(), 8);
            for rest_value in rest {
                assert_ne!(rest_value, test_value);
            }
        }
        let mut entropy = Entropy::new();
        entropy.disable(&Value::ONE).unwrap();
        entropy.try_converge(&Value::ONE).unwrap_err();
        entropy.try_converge(&Value::TWO).unwrap();
        assert_eq!(entropy.len(), 1);
    }
}

pub type BITS = u32;

const MASK: BITS = 0b1111111110;

/// 数独の各セルに入っている値の型です。
#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Debug)]
pub struct Value(BITS);
impl Value {
    pub const ONE: Value = Value(0b0000000010);
    pub const TWO: Value = Value(0b0000000100);
    pub const THREE: Value = Value(0b0000001000);
    pub const FOUR: Value = Value(0b0000010000);
    pub const FIVE: Value = Value(0b0000100000);
    pub const SIX: Value = Value(0b0001000000);
    pub const SEVEN: Value = Value(0b0010000000);
    pub const EIGHT: Value = Value(0b0100000000);
    pub const NINE: Value = Value(0b1000000000);
    pub fn new(value: BITS) -> Option<Self> {
        if value > 0 && value <= 9 {
            Some(unsafe { Self::new_unchecked(value) })
        } else {
            None
        }
    }
    pub unsafe fn new_unchecked(value: BITS) -> Self {
        Value(1 << value)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.trailing_zeros())
    }
}

/// Valueの重複のない集合です。
#[derive(Debug, Clone)]
pub struct IterValue {
    bits: BITS,
}

impl IterValue {
    pub fn len(&self) -> BITS {
        self.bits.count_ones()
    }
    fn new_raw(bits: BITS) -> IterValue {
        IterValue { bits }
    }
}

impl Iterator for IterValue {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        match Value::new(self.bits.trailing_zeros()) {
            Some(v) => {
                self.bits -= v.0;
                Some(v)
            }
            None => None,
        }
    }
}

/// そのセルでの値の可能性を表します。
// 初期状態はMASK、
// 否定された可能性がnの場合、(n+1)桁目が0となる。
#[derive(Clone, Debug)]
pub struct Entropy(BITS);
impl Entropy {
    const NEVER: Entropy = Entropy(0);
    /// エントロピーの大きさを返します。
    pub fn len(&self) -> BITS {
        self.0.count_ones()
    }
    /// 全く収束していない新しいエントロピーを返します。
    pub fn new() -> Self {
        Default::default()
    }

    /// 新しい収束済みのエントロピーを返します。
    pub fn new_converged(value: Value) -> Self {
        Entropy(value.0)
    }

    /// その値になる可能性があるかどうかを返します。
    pub fn is_possible(&self, value: &Value) -> bool {
        self.0 & value.0 != 0
    }

    /// 指定された値の可能性を否定します。
    /// 既にその値になる可能性がなかった場合はOk(false)を返します。
    /// 可能性を否定した結果不能となった場合はErr(EntropyConflictError)を返します。
    /// 不能となったselfは回復しません。
    pub fn disable(&mut self, value: &Value) -> Result<bool, EntropyConflictError> {
        if self.is_possible(value) {
            self.0 -= value.0;
            if self.0 == 0 {
                Err(EntropyConflictError {
                    entropy: Entropy::NEVER,
                    value: value.to_owned(),
                })
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 確率が収束しているかどうかを返します。
    pub fn is_converged(&self) -> bool {
        self.len() == 1
    }

    /// 確率が収束している場合、その値を返します。
    pub fn check_convergence(&self) -> Option<Value> {
        if self.is_converged() {
            Some(Value(self.0))
        } else {
            None
        }
    }

    /// 指定された値に収束させます。
    /// 収束した場合は否定された可能性のIterValueを返します。
    pub fn try_converge(&mut self, value: &Value) -> Result<IterValue, EntropyConflictError> {
        if self.is_possible(value) {
            let res = Ok(IterValue::new_raw(self.0 - value.0));
            self.0 = value.0;
            res
        } else {
            Err(EntropyConflictError {
                value: value.to_owned(),
                entropy: self.to_owned(),
            })
        }
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Entropy(MASK)
    }
}

impl std::fmt::Display for Entropy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let iter = self.clone().into_iter();
        for v in iter {
            write!(f, "{v}")?;
        }
        write!(f, "]")
    }
}

impl IntoIterator for Entropy {
    type Item = Value;
    type IntoIter = IterValue;
    fn into_iter(self) -> Self::IntoIter {
        IterValue::new_raw(self.0)
    }
}

/// 値とエントロピーが競合した際のエラーです。
// 競合した値とエントロピーのペアが存在する場合はSome(Value, Entropy)の形式で指定。
// disableの結果、存在するペアがなくなってしまった場合はNoneで指定。
#[derive(Debug)]
pub struct EntropyConflictError {
    value: Value,
    entropy: Entropy,
}

impl std::fmt::Display for EntropyConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <- {}", self.entropy, self.value)
    }
}
