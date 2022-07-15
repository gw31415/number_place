use std::collections::HashSet;
/// 数独上の位置を表します。
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Place {
    pub x: usize,
    pub y: usize,
}

impl Place {
    /// 新しいPlaceを返します。
    /// 引数の値が0以上9未満でない場合はNoneが返ります。
    pub fn new(x: usize, y: usize) -> Option<Self> {
        if x < 9 && y < 9 {
            Some(Self { x, y })
        } else {
            None
        }
    }
    /// そのPlaceに直接的に影響のあるPlaceを返します。
    pub fn dependencies(&self) -> Dependencies {
        let mut depends = Dependencies::new();
        let Place { x, y } = *self;
        let keystone = ((x / 3) * 3, (y / 3) * 3);
        for i in 0..9 {
            depends.x.insert(Place { x: self.x, y: i });
            depends.y.insert(Place { x: i, y: self.y });
            depends.s.insert(Place {
                x: i / 3 + keystone.0,
                y: i % 3 + keystone.1,
            });
        }
        // 自身は削除する
        depends.s.remove(self);
        depends.x.remove(self);
        depends.y.remove(self);
        depends
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
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
