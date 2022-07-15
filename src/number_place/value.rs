/// 数独の各セルに入っている値の型です。
#[derive(PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
#[derive(Debug)]
pub struct Value(u8);
impl Value {
    pub const ONE: Value = Value(1);
    pub const TWO: Value = Value(2);
    pub const THREE: Value = Value(3);
    pub const FOUR: Value = Value(4);
    pub const FIVE: Value = Value(5);
    pub const SIX: Value = Value(6);
    pub const SEVEN: Value = Value(7);
    pub const EIGHT: Value = Value(8);
    pub const NINE: Value = Value(9);
    pub fn new(value: u8) -> Option<Self> {
        if value > 0 && value <= 9 {
            Some(Value(value))
        } else {
            None
        }
    }
    pub unsafe fn new_unchecked(value: u8) -> Self {
        Value(value)
    }
}

impl Into<u8> for Value {
    fn into(self) -> u8 {
        let Self(value) = self;
        value
    }
}

impl TryFrom<u8> for Value {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value != 0 && value <= 9 {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
