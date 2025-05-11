/// 数独上の位置を表します。
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Place(usize);

impl Place {
    /// PlaceのX座標を返します。
    pub fn x(&self) -> usize {
        self.0 % 9
    }
    /// PlaceのY座標を返します。
    pub fn y(&self) -> usize {
        self.0 / 9
    }
    /// 先頭から数えたインデックスをそのまま返します。
    pub fn raw(&self) -> &usize {
        &self.0
    }
    /// 新しいPlaceを返します。
    /// 引数の値が0以上9未満でない場合はNoneが返ります。
    pub const fn new(x: usize, y: usize) -> Option<Place> {
        if x < 9 && y < 9 {
            Some(Place(y * 9 + x))
        } else {
            None
        }
    }
    /// 新しいPlaceを返します。
    /// そのX、Yの値が範囲内にあるかどうかの確認をしません。
    pub const unsafe fn new_unchecked(x: usize, y: usize) -> Place {
        Place(y * 9 + x)
    }
    /// 新しいPlaceを返します。
    /// そのインデックスの値が範囲外であればNoneが返ります。。
    pub const fn new_from_raw(i: usize) -> Option<Place> {
        if i < 81 { Some(Place(i)) } else { None }
    }
    /// 新しいPlaceを返します。
    /// そのインデックスの値が範囲内にあるかどうかの確認をしません。
    pub const unsafe fn new_from_raw_unchecked(i: usize) -> Place {
        Place(i)
    }
    /// そのPlaceに直接的に影響のあるPlaceを返します。
    pub fn dependencies(&self) -> Dependencies<'_> {
        Dependencies(self)
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

/// 関係するブロックを表します。
#[derive(Debug, Clone)]
pub struct Dependencies<'a>(&'a Place);

impl Dependencies<'_> {
    /// 何のPlaceに関するDependenciesかを返します。
    pub fn about(&self) -> &Place {
        self.0
    }
    /// 横の1行のラインの依存セルを返します。
    pub fn x_line(&self) -> block::Block {
        unsafe { block::Block::new_unchecked(self.0.0 / 9 * 9, block::BlockType::XLine) }
    }
    /// 縦の1列のラインの依存セルを返します。
    pub fn y_line(&self) -> block::Block {
        unsafe { block::Block::new_unchecked(self.0.0 % 9, block::BlockType::YLine) }
    }
    /// 3x3の領域の依存セルを返します。
    pub fn square(&self) -> block::Block {
        let (x, y) = (self.0.x(), self.0.y());
        unsafe {
            block::Block::new_unchecked((y / 3) * 27 + ((x / 3) * 3), block::BlockType::Square)
        }
    }
}

impl<'a> IntoIterator for Dependencies<'a> {
    type Item = block::Block;
    type IntoIter = BlockIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        BlockIter(self, 0)
    }
}

/// ある1つのセルに関係する3つのブロックのイテレータです。
pub struct BlockIter<'a>(Dependencies<'a>, usize);

impl Iterator for BlockIter<'_> {
    type Item = block::Block;
    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.1 {
            0 => Some(self.0.x_line()),
            1 => Some(self.0.y_line()),
            2 => Some(self.0.square()),
            _ => None,
        };
        self.1 += 1;
        res
    }
}

mod block {
    use super::*;
    /// 互いに関係のあるPlaceの9セル1組がいずれのタイプかを表します。
    pub enum BlockType {
        /// 横一行の9セル
        XLine,
        /// 縦一列の9セル
        YLine,
        /// 3x3の正方形9セル
        Square,
    }

    /// 互いに関係のあるPlaceの9セル1組を表します。
    // Block.0は最初に指し示すPlaceの値
    pub struct Block(usize, BlockType);

    impl Block {
        /// 直接ブロックを構築して返します。
        pub unsafe fn new_unchecked(i: usize, blocktype: BlockType) -> Block {
            Block(i, blocktype)
        }
    }

    impl IntoIterator for Block {
        type Item = Place;
        type IntoIter = PlaceIter;
        fn into_iter(self) -> Self::IntoIter {
            use BlockType::*;
            match self.1 {
                XLine => PlaceIter::x_line(self.0),
                YLine => PlaceIter::y_line(self.0),
                Square => PlaceIter::square(self.0),
            }
        }
    }
    /// Placeを返すイテレータです。
    /// 互いに関係のある9セル内でイテレートします。
    pub struct PlaceIter {
        place: usize,
        sneak: fn(usize) -> usize,
        len: usize,
    }

    impl PlaceIter {
        fn x_line(place: usize) -> PlaceIter {
            fn sneak(index: usize) -> usize {
                index + 1
            }
            PlaceIter {
                place,
                sneak,
                len: 9,
            }
        }
        fn y_line(place: usize) -> PlaceIter {
            fn sneak(index: usize) -> usize {
                index + 9
            }
            PlaceIter {
                place,
                sneak,
                len: 9,
            }
        }
        fn square(place: usize) -> PlaceIter {
            fn sneak(index: usize) -> usize {
                index + if index % 3 == 2 { 7 } else { 1 }
            }
            PlaceIter {
                place,
                sneak,
                len: 9,
            }
        }
    }

    impl Iterator for PlaceIter {
        type Item = Place;
        fn next(&mut self) -> Option<Self::Item> {
            if 0 != self.len {
                let res = Some(Place(self.place));
                self.place = (self.sneak)(self.place);
                self.len -= 1;
                res
            } else {
                None
            }
        }
    }
}

pub use block::*;
