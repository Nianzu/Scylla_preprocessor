use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{env, string};

static MIN_ELO: u16 = 2000;

struct game {
    white_elo: Option<u16>,
    black_elo: Option<u16>,
    
}

impl game {
    fn new() -> game {
        game {
            white_elo: None,
            black_elo: None,
        }
    }

    fn get_tag(line: &str) -> (String, String) {
        line.replace(']', "")
            .replace('[', "")
            .replace('\"', "")
            .split_once(" ")
            .map(|(a, b)| (a.to_owned(), b.to_owned()))
            .unwrap()
    }

    fn is_desired(&self) -> bool {
        !self.white_elo.is_none()
            && !self.black_elo.is_none()
            && self.white_elo >= Some(MIN_ELO)
            && self.black_elo >= Some(MIN_ELO)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please include a path to the pgn you want to use");
        return Ok(());
    }
    let mut games: Vec<game> = vec![];
    let mut last_line_type = "moves";
    let file = File::open(&args[1]).expect("Error opening file");
    let reader = BufReader::new(file);
    let mut current_game = game::new();

    for line_obj in reader.lines() {
        let line = line_obj?;
        let mut current_line_type = "moves";
        if line.is_empty() {
            continue;
        }
        if line.chars().next().expect("Issue getting first char") == '[' {
            current_line_type = "tags";
            let (name, content) = game::get_tag(&line);
            match name.as_str() {
                "WhiteElo" => {
                    current_game.white_elo = content.parse::<u16>().ok();
                }
                "BlackElo" => {
                    current_game.black_elo = content.parse::<u16>().ok();
                }
                _ => {}
            }
        }
        if current_line_type == "moves" && last_line_type == "tags" {
            if current_game.is_desired() {
                games.push(current_game);
                current_game = game::new();
            }
        }
        last_line_type = current_line_type;
    }
    println!("Games: {}", games.len());
    Ok(())
}
