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
    fn value_into_is_possible() {
        for i in 1..=9 {
            let value = &Value::new(i).unwrap();
            let entropy: Entropy = value.clone().into();
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
            let rest = entropy.superimpose(test_value.clone()).unwrap();
            assert_eq!(entropy.len(), 1);
            assert_eq!(rest.len(), 8);
            for rest_value in rest {
                assert_ne!(rest_value, test_value);
            }
        }
        let mut entropy = Entropy::new();
        entropy.disable(&Value::ONE).unwrap();
        entropy.superimpose(Value::ONE).unwrap_err();
        entropy.superimpose(Value::TWO).unwrap();
        assert_eq!(entropy.len(), 1);
    }
}

/// ValueやEntropyが内部的に用いている型です。
/// コンストラクト時にはこの型で入力を行います。
pub type BITS = u32;

const MASK: BITS = 0b1111111110;

/// 数独の各セルに入っている値の型です。
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Value(BITS);
impl Value {
    pub const ONE: Value = unsafe { Value::new_unchecked(1) };
    pub const TWO: Value = unsafe { Value::new_unchecked(2) };
    pub const THREE: Value = unsafe { Value::new_unchecked(3) };
    pub const FOUR: Value = unsafe { Value::new_unchecked(4) };
    pub const FIVE: Value = unsafe { Value::new_unchecked(5) };
    pub const SIX: Value = unsafe { Value::new_unchecked(6) };
    pub const SEVEN: Value = unsafe { Value::new_unchecked(7) };
    pub const EIGHT: Value = unsafe { Value::new_unchecked(8) };
    pub const NINE: Value = unsafe { Value::new_unchecked(9) };
    pub const fn new(value: BITS) -> Option<Self> {
        if value > 0 && value <= 9 {
            Some(unsafe { Self::new_unchecked(value) })
        } else {
            None
        }
    }
    pub const unsafe fn new_unchecked(value: BITS) -> Self {
        Value(1 << value)
    }
}

impl Into<u32> for Value {
    fn into(self) -> u32 {
        self.0.trailing_zeros()
    }
}

impl Into<Entropy> for Value {
    fn into(self) -> Entropy {
        Entropy(self.0)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.trailing_zeros())
    }
}

/// Valueの重複のないイテレータです。
#[derive(Debug, Clone)]
pub struct ValueIter(BITS);

impl ValueIter {
    pub fn len(&self) -> BITS {
        self.0.count_ones()
    }
}

impl Iterator for ValueIter {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        match Value::new(self.0.trailing_zeros()) {
            Some(v) => {
                self.0 -= v.0;
                Some(v)
            }
            None => None,
        }
    }
}

/// そのセルでの値の可能性を表します。
// 初期状態はMASK、
// 否定された可能性がnの場合、(n+1)桁目が0となる。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entropy(BITS);
impl Entropy {
    /// 全く収束していない新しいエントロピーを返します。
    pub const fn new() -> Self {
        Entropy(MASK)
    }

    /// エントロピーの大きさを返します。
    pub fn len(&self) -> BITS {
        self.0.count_ones()
    }

    /// その値になる可能性があるかどうかを返します。
    pub fn is_possible(&self, value: &Value) -> bool {
        self.0 & value.0 != 0
    }

    /// 指定された値の可能性を手動で否定します。
    /// 可能性を削除した場合はOk(true)を返します。
    /// 既にその値になる可能性がなかった場合はOk(false)を返します。
    /// 可能性を否定した結果不能となった場合はErr(EntropyConflictError)を返します。
    pub fn disable(&mut self, value: &Value) -> Result<bool, EntropyConflictError> {
        if self.is_possible(value) {
            if self.len() == 1 {
                Err(EntropyConflictError {
                    main_entropy: self.to_owned(),
                    conflicting_entropy: value.to_owned().into(),
                })
            } else {
                self.0 -= value.0;
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// 他のEntropy、またはValueと重ねあわせます。
    /// 重ねあわせが出来た場合は否定された可能性のリストを返します。
    pub fn superimpose<T>(&mut self, into_entropy: T) -> Result<ValueIter, EntropyConflictError>
    where
        T: Into<Entropy>,
    {
        let entropy: Entropy = into_entropy.into();
        // selfから削除される予定の可能性のリスト
        let res = ValueIter(!entropy.0 & self.0);
        if res.0 != self.0 {
            // 全ては削除されない場合
            self.0 &= entropy.0;
            Ok(res)
        } else {
            // 仮に可能性削除すると全て消えてしまう場合。
            Err(EntropyConflictError {
                main_entropy: self.to_owned(),
                conflicting_entropy: entropy.to_owned(),
            })
        }
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Entropy::new()
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

impl TryInto<Value> for Entropy {
    type Error = ();
    fn try_into(self) -> Result<Value, Self::Error> {
        if self.len() == 1 {
            Ok(Value(self.0))
        } else {
            Err(())
        }
    }
}

impl IntoIterator for Entropy {
    type Item = Value;
    type IntoIter = ValueIter;
    fn into_iter(self) -> Self::IntoIter {
        ValueIter(self.0)
    }
}

/// エントロピーが競合した際のエラーです。
#[derive(Debug)]
pub struct EntropyConflictError {
    conflicting_entropy: Entropy,
    main_entropy: Entropy,
}

impl std::fmt::Display for EntropyConflictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} x {}", self.main_entropy, self.conflicting_entropy)
    }
}
