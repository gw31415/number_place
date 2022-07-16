use super::entropy::*;
use super::place::*;
use std::collections::HashSet;

/// 数独の問題を解く構造体です。
#[derive(Default)]
pub struct Processor([[Entropy; 9]; 9]);

impl Processor {
    /// 現在確認できたエントロピーの総量を返します。
    pub fn entropy_amount(&self) -> f64 {
        let mut count = 1f64;
        for y in 0..9 {
            for x in 0..9 {
                count *= self.0[x][y].len() as f64
            }
        }
        count
    }
    /// 現在の条件で、位置に対してどのような値が入る可能性があるかを返します。
    pub fn get_atlas(&self) -> &[[Entropy; 9]; 9] {
        &self.0
    }
    /// 指定されたセルのエントロピーを収束させます。
    pub fn input(&mut self, value: Value, place: Place) -> Result<(), EntropyConflictError> {
        let mut remaining_sets = HashSet::new();
        remaining_sets.insert((value, place));
        while !remaining_sets.is_empty() {
            let iter = remaining_sets;
            remaining_sets = Default::default();
            for (value, place) in iter {
                for remaining_set in self.inner_input(value, place)? {
                    remaining_sets.insert(remaining_set);
                }
            }
        }
        Ok(())
    }

    /// 指定されたセルをその値に収束させます。
    /// 新たに必要になった収束先と値のセットを返します。
    /// この実装になったのはスタックオーバーフロー対策の為。
    fn inner_input(
        &mut self,
        value: Value,
        place: Place,
    ) -> Result<HashSet<(Value, Place)>, EntropyConflictError> {
        let mut remaining_sets = HashSet::new();
        macro_rules! entropy {
            ($place: expr) => {
                self.0[$place.x][$place.y]
            };
        }
        /// セルの値を1つ否定する度に呼ぶ。
        /// そのセルに関係するセルが唯一の値になる可能性があるので、
        /// その値について唯一の可能性の位置となったセルを収束させる。
        macro_rules! search_uniqueness_around {
            ($disabled_value: expr, $changing_place: expr) => {{
                let disabled_value: &Value = $disabled_value;
                let changing_place: &Place = $changing_place;
                let dependencies = changing_place.dependencies();
                for block in [
                    dependencies.x_line(),
                    dependencies.y_line(),
                    dependencies.square(),
                ] {
                    let mut first: Option<&Place> = None;
                    for affected_place in block {
                        // 与えられた一列(y_line)、一行(x_line)、一区画(square)
                        // (:block)のうちで、与えられた$valueが唯一のものを探す。
                        if entropy!(affected_place).is_possible(disabled_value) {
                            match first {
                                Some(_) => {
                                    // まだ複数のセルで可能性がある。
                                    break;
                                },
                                None => {
                                    // 可能性のある最初のセル
                                    first = Some(affected_place);
                                }
                            }
                        }
                    }
                    match first {
                        Some(unique_place) => {
                            remaining_sets.insert((disabled_value.to_owned(), unique_place.to_owned()));
                        },
                        None => {
                            unreachable!();
                        }
                    }
                }
            }};
        }
        // 指定されたセルのエントロピーを収束させる。
        let disabled_values = entropy!(&place).try_converge(&value)?;
        for disabled_value in disabled_values {
            // 削除された可能性について探索
            search_uniqueness_around!(&disabled_value, &place);
        }
        // 直接関係のあるセルから可能性を削除していく。
        for related_place in place.dependencies().into_all() {
            if entropy!(related_place).disable(&value)? {
                search_uniqueness_around!(&value, &related_place);
            }
        }
        Ok(remaining_sets)
    }
}
