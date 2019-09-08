use crossterm::{cursor, terminal, ClearType};
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::{error, fs, io, sync::mpsc, thread};

const NUM_QUESTIONS: usize = 15;
const BEFORE_START_SECS: usize = 3;
// const SLEEP_DURATION: Duration = Duration::from_secs(2);
const SLEEP_DURATION: Duration = Duration::from_millis(100);
// const ANSWER_TIME: Duration = Duration::from_secs(60);
const ANSWER_TIME: Duration = Duration::from_secs(5);

macro_rules! draw_center {
    ($($x:tt)*) => {{
        terminal().clear(ClearType::All)?;
        cursor().goto(10, 5)?;
        println!($($x)*);
        cursor().goto(0, 0)?;
    }}
}

// macro_rules! draw_top {
//     ($($x:tt)*) => {{
//         let (x, y) = cursor().pos();
//         cursor().goto(0, 0)?;
//         println!($($x)*);
//         cursor().goto(x, y)?;
//     }}
// }

macro_rules! wait_for {
    ($sec:expr, $($x:tt)*) => {{
        for i in (0..=$sec).rev() {
            if i == 0 {
                draw_center!("!!! 開始 !!!");
            } else {
                draw_center!($($x)*, sec=i);
            }
            thread::sleep(Duration::from_secs(1));
        }
    }}
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let data = fs::read_to_string("data.csv")?;
    let data: Vec<(&str, &str)> = data
        .lines()
        .map(|line| line.split(','))
        .map(|mut splitted| (splitted.next().unwrap(), splitted.next().unwrap()))
        .collect();

    let mut rng = rand::thread_rng();

    let rand_iter = (0..)
        .filter_map(|_| data.choose(&mut rng))
        .scan(HashSet::new(), |used, &(read, writing)| {
            if read == "" {
                Some(None)
            } else if !used.insert(read) {
                Some(None)
            } else {
                Some(Some((read, writing)))
            }
        })
        .filter_map(|x| x);

    wait_for!(BEFORE_START_SECS, "開始まで {sec} 秒");

    let answers: Vec<_> = rand_iter.take(NUM_QUESTIONS).collect();

    for (idx, (_, question)) in answers.iter().enumerate() {
        draw_center!("[{}/{}] {}", idx + 1, NUM_QUESTIONS, question);
        thread::sleep(SLEEP_DURATION);
    }

    terminal().clear(ClearType::All)?;
    wait_for!(
        BEFORE_START_SECS,
        "終了。解答時間は {} 秒です。開始まで {sec} 秒",
        ANSWER_TIME.as_secs()
    );

    terminal().clear(ClearType::All)?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        match tx.send(input.trim().to_string()) {
            Ok(()) => {}
            Err(_) => break,
        }
    });

    let mut responses = HashSet::new();
    let time = Instant::now();
    while time.elapsed() < ANSWER_TIME {
        if let Ok(v) = rx.try_recv() {
            responses.insert(v);
        }
    }
    drop(rx);

    terminal().clear(ClearType::All)?;
    println!("そこまで!");
    println!();
    let mut res = 0;
    for (read, writing) in answers {
        let sign = if responses.contains(read) || responses.contains(writing) {
            res += 1;
            'o'
        } else {
            'x'
        };
        println!("{} {}", sign, writing);
    }
    println!();
    println!("結果: {}/{}", res, NUM_QUESTIONS);

    Ok(())
}
