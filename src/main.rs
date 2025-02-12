use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

static MIN_ELO: u16 = 2000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please include a path to the pgn you want to use");
        return Ok(());
    }
    let mut game_num = 0;
    let mut last_line_type = "moves";
    let file = File::open(&args[1]).expect("Error opening file");
    let reader = BufReader::new(file);
    let mut white_elo = 0;
    let mut black_elo = 0;
    for line_obj in reader.lines() {
        let line = line_obj?;
        let mut current_line_type = "moves";
        if line.is_empty() {
            continue;
        }
        if line.chars().next().expect("Issue getting first char") == '[' {
            current_line_type = "tags";
            if line.split_once(" ").map(|(a,b)| a.contains("WhiteElo")).unwrap() {
                white_elo = line.split_once(" ").map(|(a,b)| b.replace('\"',"").replace(']',"").parse::<u16>().unwrap()).unwrap();
            }
            if line.split_once(" ").map(|(a,b)| a.contains("BlackElo")).unwrap() {
                black_elo = line.split_once(" ").map(|(a,b)| b.replace('\"',"").replace(']',"").parse::<u16>().unwrap()).unwrap();
            }
        }
        if current_line_type == "moves" && last_line_type == "tags" {
            if white_elo >= MIN_ELO && black_elo >= MIN_ELO {
                game_num += 1;
            }
            white_elo = 0;
            black_elo = 0;

        }
        last_line_type = current_line_type;
    }
    println!("Games: {game_num}");
    Ok(())
}
