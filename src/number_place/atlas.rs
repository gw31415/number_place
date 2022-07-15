use super::entropy::*;
use super::place::*;
use super::value::*;
use std::collections::HashSet;

/// 数独の問題を解く構造体です。
#[derive(Default)]
pub struct Processor([[Entropy; 9]; 9]);

impl Processor {
    fn get(&self, place: &Place) -> &Entropy {
        &self.0[place.x][place.y]
    }
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
        // 指定されたセルのエントロピーを収束させる。
        self.0[place.x][place.y].try_converge(&value)?;
        // ↑の収束時に削除された可能性についてdisable_valueを行なうこと。
        /*
        macro_rules! disable_value {
            ($place: expr, $value: expr) => {{}};
        }
        */
        let mut remaining_sets = HashSet::new();

        // 直接関係のあるセルから可能性を削除していく。
        for place_related_1 in place.dependencies().into_all() {
            macro_rules! entropy_at_place_related_1 {
                () => {
                    self.0[place_related_1.x][place_related_1.y]
                };
            }
            if !entropy_at_place_related_1!().is_converged() {
                if entropy_at_place_related_1!().disable(&value)? {
                    // 可能性削除ができたということは、アトラスに変化があったということ。
                    // その結果、削除したセル(place_related_1)に関係するセル
                    // (つまり入力を受けたセルから見て2次的に関係のあるセル: place_related_2)
                    // の可能性(possible_value)のいずれかが、
                    // place_related_2のある列・行・区画のいずれかの中で唯一のものになった場合、
                    // その可能性はその値に収束するべきものであると見做す
                    // (同じ行・列・区画で、他のセルに入らないがこのセルにのみ入る場合はその値が入ると見做す)。
                    for place_related_2 in place_related_1.dependencies().into_all() {
                        let dependencies = place_related_2.dependencies();
                        let (x_line, y_line, square) = (
                            dependencies.x_line(),
                            dependencies.y_line(),
                            dependencies.square(),
                        );
                        /// 与えられた一列(y_line)、一行(x_line)、一区画(square)
                        /// (:$one_depend_set)のうちで、与えられた$valueが唯一のものかどうか。
                        macro_rules! is_only {
                            ($value :expr, $one_depend_set: expr) => {{
                                let one_depend_set: &HashSet<Place> = $one_depend_set;
                                let value: &Value = $value;
                                let mut is_only = true;
                                for place in one_depend_set {
                                    let entropy = self.get(place);
                                    if entropy.is_possible(value) {
                                        is_only = false;
                                        break;
                                    }
                                }
                                is_only
                            }};
                        }
                        for possible_values in self.get(&place_related_2).to_owned() {
                            if is_only!(&possible_values, x_line)
                                || is_only!(&possible_values, y_line)
                                || is_only!(&possible_values, square)
                            {
                                remaining_sets
                                    .insert((possible_values.clone(), place_related_2.clone()));
                                break;
                            }
                        }
                    }
                }

                // 仮に今回の入力によって関係するセルの可能性が収束した場合
                if let Some(value) = self.get(&place_related_1).check_convergence() {
                    remaining_sets.insert((value, place_related_1));
                }
            }
        }
        Ok(remaining_sets)
    }
}
