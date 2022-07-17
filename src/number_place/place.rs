use std::collections::HashSet;
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
        let mut depends = Dependencies::new();
        let (x, y) = (self.x(), self.y());
        let keystone = ((x / 3) * 3, (y / 3) * 3);
        for i in 0..9 {
            depends.x.insert(unsafe { Place::new_unchecked(x, i) });
            depends.y.insert(unsafe { Place::new_unchecked(i, y) });
            depends
                .s
                .insert(unsafe { Place::new_unchecked(i / 3 + keystone.0, i % 3 + keystone.1) });
        }
        // 自身は削除する
        depends.s.remove(self);
        depends.x.remove(self);
        depends.y.remove(self);
        depends
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
                        if (next_value / 3) % 3 == 2 {
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

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

pub struct Dependencies {
    s: HashSet<Place>,
    x: HashSet<Place>,
    y: HashSet<Place>,
}

impl Dependencies {
    fn new() -> Self {
        Dependencies {
            s: Default::default(),
            x: Default::default(),
            y: Default::default(),
        }
    }

    /// 3x3の領域の依存セルを返します。
    pub fn square(&self) -> &HashSet<Place> {
        &self.s
    }

    /// 横の1行のラインの依存セルを返します。
    pub fn x_line(&self) -> &HashSet<Place> {
        &self.x
    }

    /// 横の1行のラインの依存セルを返します。
    pub fn y_line(&self) -> &HashSet<Place> {
        &self.y
    }

    /// 全ての依存セルを統合して返します。
    pub fn into_all(self) -> HashSet<Place> {
        let Self { s: mut res, x, y } = self;
        for i in x {
            res.insert(i);
        }
        for i in y {
            res.insert(i);
        }
        res
    }
}
