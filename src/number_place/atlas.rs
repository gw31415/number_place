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
        /// 与えられた一列(y_line)、一行(x_line)、一区画(square)
        /// (:$one_depend_set)のうちで、与えられた$valueが唯一のものかどうか。
        macro_rules! is_only {
            ($value :expr, $one_depend_set: expr) => {{
                let one_depend_set: &HashSet<Place> = $one_depend_set;
                let value: &Value = $value;
                let mut is_only = true;
                for place in one_depend_set {
                    let entropy = &entropy!(&place);
                    if entropy.is_possible(value) {
                        is_only = false;
                        break;
                    }
                }
                is_only
            }};
        }
        macro_rules! search_uniqueness {
            ($value: expr, $place: expr) => {{
                let disabled_value: &Value = $value;
                let changing_place: &Place = $place;
                for affected_place in changing_place.dependencies().into_all() {
                    let dependencies = affected_place.dependencies();
                    let (x_line, y_line, square) = (
                        dependencies.x_line(),
                        dependencies.y_line(),
                        dependencies.square(),
                    );
                    if is_only!(disabled_value, x_line)
                        || is_only!(disabled_value, y_line)
                        || is_only!(disabled_value, square)
                    {
                        remaining_sets.insert((disabled_value.clone(), affected_place.clone()));
                    }
                }
            }};
        }
        // 指定されたセルのエントロピーを収束させる。
        let disabled_values = entropy!(&place).try_converge(&value)?;
        for disabled_value in disabled_values {
            let changing_place = &place;
            search_uniqueness!(&disabled_value, changing_place);
        }
        // 直接関係のあるセルから可能性を削除していく。
        for place_related_1 in place.dependencies().into_all() {
            if !entropy!(place_related_1).is_converged() {
                if entropy!(place_related_1).disable(&value)? {
                    // 可能性削除ができたということは、アトラスに変化があったということ。
                    // その結果、削除したセル(place_related_1)に関係するセル
                    // (つまり入力を受けたセルから見て2次的に関係のあるセル: place_related_2)
                    // の可能性(possible_value)のいずれかが、
                    // place_related_2のある列・行・区画のいずれかの中で唯一のものになった場合、
                    // その可能性はその値に収束するべきものであると見做す
                    // (同じ行・列・区画で、他のセルに入らないがこのセルにのみ入る場合はその値が入ると見做す)。
                    let changing_place = &place_related_1;
                    let disabled_value = &value;
                    search_uniqueness!(disabled_value, changing_place);
                }

                // 仮に今回の入力によって関係するセルの可能性が収束した場合
                if let Some(value) = entropy!(&place_related_1).check_convergence() {
                    remaining_sets.insert((value, place_related_1));
                }
            }
        }
        Ok(remaining_sets)
    }
}
