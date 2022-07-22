use number_place::*;

fn main() {
    // 入力した文字数
    let mut char_count = 0;
    let mut field = EntropyField::new();
    for y in 0..9 {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let line = line.as_bytes();
        if line.len() != 10 {
            panic!("入力形式が正しくありません。");
        }
        for x in 0..9 {
            let c = line[x];
            char_count += 1;
            if c > b'9' || c < b'1' {
                continue;
            }
            use std::io::Write;
            std::io::stdout().flush().unwrap();
            if let Some(value) = Value::new((c - b'0').into()) {
                let place = Place::new(x, y).unwrap();
                if let Err(error) = field.insert(place.clone(), value) {
                    eprintln!("{error}");
                    panic!("ルール違反が検出されました。");
                }
                println!("STEP {:2}: {}", char_count, field.len());
            } else {
                panic!("入力形式が正しくありません。");
            }
        }
    }
    println!("{field}");

    if field.len() > 1. {
        use number_place::{entropy::ValueIter, entropy_field::CELLS_COUNT};
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
        let mut stack: Vec<(EntropyField, Place, ValueIter)> = {
            let (place, entropy) = first_entropy(&field);
            vec![(field, place, entropy.into_iter())]
        };
        println!();
        println!("======BRUTE-FORCE======");
        while let Some((field, place, mut iter)) = stack.pop() {
            if let Some(value) = iter.next() {
                let mut next_field = field.clone();
                stack.push((field, place.clone(), iter));
                print!("ASSUMING: {value}@{place} -> LEN: ");
                if let Ok(_) = next_field.insert(place, value) {
                    println!("{}", next_field.len());
                    if next_field.len() == 1. {
                        println!("{next_field}");
                        return;
                    } else {
                        let (place, entropy) = first_entropy(&next_field);
                        stack.push((next_field, place, entropy.into_iter()))
                    }
                } else {
                    println!("0");
                }
            }
        }
        eprintln!("確定した解が見つからないままスタックがなくなりました。");
    }
}
