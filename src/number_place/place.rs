/// 数独上の位置を表します。
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Place(usize);

impl Place {
    pub fn x(&self) -> usize {
        self.0 % 9
    }
    pub fn y(&self) -> usize {
        self.0 / 9
    }
    /// 新しいPlaceを返します。
    /// 引数の値が0以上9未満でない場合はNoneが返ります。
    pub const fn new(x: usize, y: usize) -> Option<Place> {
        if x < 9 && y < 9 {
            Some(unsafe { Place::new_unchecked(x, y) })
        } else {
            None
        }
    }
    /// 新しいPlaceを返します。
    pub const unsafe fn new_unchecked(x: usize, y: usize) -> Place {
        Place(y * 9 + x)
    }
    /// そのPlaceに直接的に影響のあるPlaceを返します。
    pub fn dependencies(&self) -> Dependencies {
        Dependencies {
            x: { Block(self.0 / 9 * 9, BlockContext::XLine) },
            y: { Block(self.0 % 9, BlockContext::YLine) },
            s: {
                let (x, y) = (self.x(), self.y());
                Block((y / 3) * 27 + ((x / 3) * 3), BlockContext::Square)
            },
        }
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

pub struct Dependencies {
    /// 3x3の領域の依存セルを返します。
    pub s: Block,
    /// 横の1行のラインの依存セルを返します。
    pub x: Block,
    /// 横の1行のラインの依存セルを返します。
    pub y: Block,
}

impl Iterator for Dependencies {
    type Item = Place;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.s.next() {
            Some(p)
        } else if let Some(p) = self.x.next() {
            Some(p)
        } else {
            self.y.next()
        }
    }
}

enum BlockContext {
    Finished,
    XLine,
    YLine,
    Square,
}

/// 互いに関係のあるPlaceのイテレータ
pub struct Block(usize, BlockContext);

impl Iterator for Block {
    type Item = Place;
    fn next(&mut self) -> Option<Self::Item> {
        use BlockContext::*;
        let mut next_value = self.0;
        Some(Place(std::mem::replace(
            &mut self.0,
            match &self.1 {
                Finished => {
                    return None;
                }
                Square => {
                    next_value += if next_value % 3 == 2 {
                        if (next_value / 9) % 3 == 2 {
                            // 今の値が最後の値の時
                            self.1 = Finished;
                        }
                        7
                    } else {
                        1
                    };
                    next_value
                }
                XLine => {
                    if next_value % 9 == 8 {
                        // 今の値が最後の値の時
                        self.1 = Finished;
                    }
                    next_value += 1;
                    next_value
                }
                YLine => {
                    next_value += 9;
                    if next_value >= 81 {
                        // 次の値が範囲外の値の時
                        self.1 = Finished;
                    }
                    next_value
                }
            },
        )))
    }
}
