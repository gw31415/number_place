use super::entropy::*;
use super::place::*;

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
    pub fn input(&mut self, value: Value, place: Place) -> Result<(), RuleViolationError> {
        let mut remaining_sets = Vec::new();
        remaining_sets.push((value, place));
        while !remaining_sets.is_empty() {
            let iter = remaining_sets;
            remaining_sets = Default::default();
            for (value, place) in iter {
                for remaining_set in self.inner_input(value, place)? {
                    remaining_sets.push(remaining_set);
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
    ) -> Result<Vec<(Value, Place)>, RuleViolationError> {
        let mut remaining_sets = Vec::new();
        macro_rules! entropy {
            ($place: expr) => {
                self.0[$place.x()][$place.y()]
            };
        }
        /// セルの値を1つ否定する度に呼ぶ。
        /// そのセルに関係するセルが唯一の値になる可能性があるので、
        /// その値について唯一の可能性の位置となったセルを収束させる。
        macro_rules! search_uniqueness_around {
            ($disabled_value: expr, $changing_place: expr) => {{
                let disabled_value: &Value = $disabled_value;
                let changing_place: &Place = $changing_place;
                let Dependencies{x,y,s} = changing_place.dependencies();
                for block in [x,y,s] {
                    let mut first: Option<Place> = None;
                    for affected_place in block {
                        // 与えられた一列(y_line)、一行(x_line)、一区画(square)
                        // (:block)のうちで、与えられた$valueが唯一のものを探す。
                        if &affected_place != changing_place {
                            if entropy!(affected_place).is_possible(disabled_value) {
                                match first {
                                    Some(_) => {
                                        // まだ複数のセルで可能性がある。
                                        first = None;
                                        break;
                                    },
                                    None => {
                                        // 可能性のある最初のセル
                                        first = Some(affected_place);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(unique_place) = first {
                        remaining_sets.push((disabled_value.to_owned(), unique_place));
                    }
                }
            }};
        }
        // 指定されたセルのエントロピーを収束させる。
        let disabled_values =
            entropy!(&place)
                .try_converge(&value)
                .map_err(|err| RuleViolationError {
                    conflict: err,
                    place: place.to_owned(),
                })?;
        for disabled_value in disabled_values {
            // 削除された可能性について探索
            search_uniqueness_around!(&disabled_value, &place);
        }
        // 直接関係のあるセルから可能性を削除していく。
        for related_place in place.dependencies() {
            if related_place != place {
                if entropy!(related_place)
                    .disable(&value)
                    .map_err(|err| RuleViolationError {
                        conflict: err,
                        place: place.to_owned(),
                    })?
                {
                    search_uniqueness_around!(&value, &related_place);

                    // 仮にこの削除によって関係するセルの可能性の数が1つになった場合
                    if let Some(value) = entropy!(&related_place).check_convergence() {
                        remaining_sets.push((value, related_place));
                    }
                }
            }
        }
        Ok(remaining_sets)
    }
}

/// ルール違反が検出されたエラー
pub struct RuleViolationError {
    conflict: EntropyConflictError,
    place: Place,
}

impl std::fmt::Display for RuleViolationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.conflict.fmt(f)?;
        write!(f, " @{}", self.place)
    }
}
