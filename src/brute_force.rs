use super::*;
use entropy::ValueIter;
use entropy_field::*;

/// あるEntropyFieldが与えられた時、一番前方にある収束していないエントロピーの
/// 位置とエントロピーの値を返します。
fn first_entropy(field: &EntropyField) -> (Place, Entropy) {
    for i in 0..CELLS_COUNT {
        let place = unsafe { Place::new_from_raw_unchecked(i) };
        if field.entropy_at(&place).len() > 1 {
            let entropy = field.entropy_at(&place).to_owned();
            return (place, entropy);
        }
    }
    unreachable!();
}

/// 総当たりで探索を行う構造体です。
/// 1つの解答が見つかった場合も複数解答の可能性を考慮し終了はしません。
pub struct Attacker(Vec<(EntropyField, Place, ValueIter)>);
impl Attacker {
    pub fn new(field: EntropyField) -> Self {
        let (place, entropy) = first_entropy(&field);
        Attacker(vec![(field, place, entropy.into_iter())])
    }
}
impl From<EntropyField> for Attacker {
    fn from(field: EntropyField) -> Self {
        Attacker::new(field)
    }
}

impl Iterator for Attacker {
    type Item = Report;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((field, place, mut iter)) = self.0.pop() {
            if let Some(value) = iter.next() {
                let mut next_field = field.clone();
                self.0.push((field, place.clone(), iter));
                match next_field.insert(place.clone(), value.clone()) {
                    Ok(_) => {
                        if next_field.len() == 1. {
                            Some(Report::Found(next_field))
                        } else {
                            let res = Some(Report::Try {
                                value,
                                place,
                                result: Ok(next_field.clone()),
                            });
                            let (place, entropy) = first_entropy(&next_field);
                            self.0.push((next_field, place, entropy.into_iter()));
                            res
                        }
                    }
                    Err(e) => Some(Report::Try {
                        value,
                        place,
                        result: Err(e),
                    }),
                }
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

/// Attackerによるステップのレポートを返します。
pub enum Report {
    /// 一つの解答が見つかったことを示します。
    Found(EntropyField),
    /// 1つの試行の結果、解答は導けず、
    /// その試行の結果を示します。
    Try {
        /// 仮定された値
        value: Value,
        /// 仮定された場所
        place: Place,
        /// Ok(_)の場合は仮定した結果のEntropyFieldが返されます。
        /// Err(_)の場合は仮定した結果エントロピーの競合が発生したため
        /// その競合のエラーが返されます。
        result: Result<EntropyField, RuleViolationError>,
    },
}
