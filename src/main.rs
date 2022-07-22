use number_place::*;
fn main() {
    // 入力した文字数
    let mut char_count = 0;
    let mut field = EntropyField::default();
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
            if let Some(value) = Value::new((c - b'0') as BITS) {
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

}
