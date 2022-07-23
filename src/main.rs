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
        println!();
        println!("======BRUTE-FORCE======");
        for report in brute_force::Attacker::new(field) {
            use brute_force::Report::*;
            match report {
                Found(field) => {
                    println!("{field}");
                    return;
                }
                Try {
                    value,
                    place,
                    result,
                } => {
                    print!("ASSUME: {value}@{place} -> ");
                    println!(
                        "LEN: {}",
                        match result {
                            Ok(field) => field.len(),
                            Err(_) => 0.,
                        }
                    );
                }
            }
        }
    }
}
