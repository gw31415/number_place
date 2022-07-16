pub mod number_place;
fn main() {
    use number_place::*;
    // 入力した文字数
    let mut char_count = 0;
    let mut processor = Processor::default();
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
            if let Some(value) = Value::new((c - b'0') as u32) {
                let place = Place::new(x, y).unwrap();
                if let Err(error) = processor.input(value, place.clone()) {
                    eprintln!("{error} @{place}");
                    panic!("ルール違反が検出されました。");
                }
                println!("STEP {:2}: {}", char_count, processor.entropy_amount());
            } else {
                panic!("入力形式が正しくありません。");
            }
        }
    }

    let atlas = processor.get_atlas();
    for y in 0..9 {
        for x in 0..9 {
            let entropy = &atlas[x][y];
            if let Some(value) = entropy.check_convergence() {
                print!(" {} ", value);
            } else {
                print!("[{}]", entropy.len());
            }
        }
        println!();
    }
}
