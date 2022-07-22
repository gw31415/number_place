use super::entropy::*;
use super::place::*;

/// セルの個数
pub const CELLS_COUNT: usize = 81;

/// 数独の表上で演繹的にエントロピーの重ねあわせを計算する構造体です。
pub struct EntropyField([Entropy; CELLS_COUNT]);

impl Default for EntropyField {
    fn default() -> Self {
        EntropyField::new()
    }
}

impl std::fmt::Display for EntropyField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..81 {
            let entropy = &self.0[i];
            if let Ok(value) = entropy.to_owned().try_into() {
                let value: Value = value;
                write!(f, " {} ", value)?;
            } else {
                write!(f, "[{}]", entropy.len())?;
            }
            if i % 9 == 8 && i != 80 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl EntropyField {
    /// 新しいEntropyFieldを返します。
    pub const fn new() -> Self {
        const INITIAL_ENTROPY: Entropy = Entropy::new();
        EntropyField([INITIAL_ENTROPY; CELLS_COUNT])
    }
    /// 現在確認できたエントロピーの総量を返します。
    pub fn len(&self) -> f64 {
        let mut count = 1f64;
        for i in 0..81 {
            count *= self.0[i].len() as f64
        }
        count
    }

    /// 指定された位置のセルにエントロピーを適用します。
    pub fn insert(
        &mut self,
        place: Place,
        into_entropy: impl Into<Entropy>,
    ) -> Result<(), RuleViolationError> {
        let mut remaining_sets: Vec<(Entropy, Place)> = Vec::new();
        remaining_sets.push((into_entropy.into(), place));
        while !remaining_sets.is_empty() {
            let iter = remaining_sets;
            remaining_sets = Default::default();
            for (value, place) in iter {
                for remaining_set in self.inner_insert(value, place)? {
                    remaining_sets.push((remaining_set.0.into(), remaining_set.1));
                }
            }
        }
        Ok(())
    }
    /// 指定されたセルにエントロピーを適用します。
    /// 新たに必要になった収束先と値のセットを返します。
    /// この実装になったのはスタックオーバーフロー対策の為。
    fn inner_insert(
        &mut self,
        into_entropy: impl Into<Entropy>,
        place: Place,
    ) -> Result<Vec<(Value, Place)>, RuleViolationError> {
        let entropy = into_entropy.into();
        let mut remaining_sets = Vec::new();
        macro_rules! entropy {
            ($place: expr) => {
                self.0[$place.raw().to_owned()]
            };
        }
        /// セルの値を1つ否定する度に呼ぶ。
        /// そのセルに関係するセルが唯一の値になる可能性があるので、
        /// その値について唯一の可能性の位置となったセルを収束させる。
        macro_rules! search_uniqueness_around {
            ($disabled_value: expr, $changing_place: expr) => {{
                let disabled_value: &Value = $disabled_value;
                let changing_place: &Place = $changing_place;
                for block in changing_place.dependencies().into_iter() {
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
                .superimpose(entropy)
                .map_err(|err| RuleViolationError {
                    conflict: err,
                    place: place.to_owned(),
                })?;
        for disabled_value in disabled_values {
            // 削除された可能性について探索
            search_uniqueness_around!(&disabled_value, &place);
        }
        // 仮に指定されたエントロピーが収束する場合。
        // (値が一つのとき)
        if let Ok(value) = entropy!(place).to_owned().try_into() {
            for related_block in place.dependencies().into_iter() {
                for related_place in related_block.into_iter() {
                    if related_place != place {
                        if entropy!(related_place).disable(&value).map_err(|err| {
                            RuleViolationError {
                                conflict: err,
                                place: place.to_owned(),
                            }
                        })? {
                            search_uniqueness_around!(&value, &related_place);

                            // 仮にこの削除によって関係するセルの可能性の数が1つになった場合
                            if let Ok(value) = entropy!(&related_place).to_owned().try_into() {
                                remaining_sets.push((value, related_place));
                            }
                        }
                    }
                }
            }
        }
        // 直接関係のあるセルから可能性を削除していく。
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
