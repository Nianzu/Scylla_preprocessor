use core::fmt;
use rschess::{Board, Color, Piece, PieceType};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{env, string};

static MIN_ELO: u16 = 2000;

struct BitBoards {
    pawns: Vec<i8>,
    bishops: Vec<i8>,
    knights: Vec<i8>,
    rooks: Vec<i8>,
    kings: Vec<i8>,
    queens: Vec<i8>,
    piece_selected: Vec<i8>,
}

impl BitBoards {
    fn new() -> BitBoards {
        BitBoards {
            pawns: vec![0; 64],
            bishops: vec![0; 64],
            knights: vec![0; 64],
            rooks: vec![0; 64],
            kings: vec![0; 64],
            queens: vec![0; 64],
            piece_selected: vec![0; 64],
        }
    }

    fn print_board(board: Vec<i8>) -> String {
        let mut output = "".to_owned();
        for i in 0..64 {
            output += &format!("{:2}", board[i]);
            output += " ";
            if (i + 1) % 8 == 0 {
                output += "\n"
            }
        }
        output
    }
    fn print_boards(&self) -> String {
        let mut output = "".to_owned();
        output += "Pawns\n";
        output += &BitBoards::print_board(self.pawns.clone());
        output += "Bishops\n";
        output += &BitBoards::print_board(self.bishops.clone());
        output += "Knights\n";
        output += &BitBoards::print_board(self.knights.clone());
        output += "Rooks\n";
        output += &BitBoards::print_board(self.rooks.clone());
        output += "Kings\n";
        output += &BitBoards::print_board(self.kings.clone());
        output += "Queens\n";
        output += &BitBoards::print_board(self.queens.clone());
        output += "Piece Selected\n";
        output += &BitBoards::print_board(self.piece_selected.clone());
        output
    }
}

struct game {
    white_elo: Option<u16>,
    black_elo: Option<u16>,
    pgn: String,
    moves: Vec<String>,
}

impl game {
    fn new() -> game {
        game {
            white_elo: None,
            black_elo: None,
            pgn: "".to_owned(),
            moves: vec![],
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

    fn remove_brackets(input: String) -> String {
        let mut bracket_depth = 0;
        let mut output: String = "".to_owned();
        for char in input.chars() {
            if char == '{' {
                bracket_depth += 1;
            }
            if bracket_depth == 0 {
                output.push(char);
            }
            if char == '}' {
                bracket_depth -= 1;
            }
        }
        output
    }

    fn parse_moves(mut self) -> Self {
        let beginning = game::remove_brackets(self.pgn.clone());
        let mut intermediate: Vec<&str> = beginning.split(" ").collect();
        intermediate.retain(|value| !value.contains(".") && !value.is_empty());
        let end: Vec<String> = intermediate
            .into_iter()
            .map(|value| value.to_owned())
            .collect();
        self.moves = end.split_last().unwrap().1.to_vec();
        self
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
        } else {
            current_game.pgn = current_game.pgn + &line.to_owned();
        }
        if current_line_type == "moves" && last_line_type == "tags" {
            if current_game.is_desired() {
                current_game = current_game.parse_moves();
                games.push(current_game);

                // Debug code
                if games.len() > 1 {
                    break;
                }
            }

            current_game = game::new();
        }
        last_line_type = current_line_type;
    }

    let mut board = Board::default();
    for arithmetic_move in games[0].moves.clone() {
        let side_to_move = board.side_to_move();
        let move_being_made = board.san_to_move(&arithmetic_move).expect("ERRORRRRRRRRR");
        println!("Side to move: {}", side_to_move);
        println!("Move being made: {:?}", move_being_made.from_square());
        if (side_to_move == Color::White) {
            let mut pieces = BitBoards::new();
            {
                let i = move_being_made.from_square().0 as usize - 97;
                let j = move_being_made.from_square().1 as usize - 49;
                pieces.piece_selected[i + ((7 - j) * 8)] = 1;
            }
            let mut index = 0;
            for j in ('1'..='8').rev() {
                for i in 'a'..='h' {
                    if !board.occupant_of_square(i, j).unwrap().is_none() {
                        let piece_value =
                            if board.occupant_of_square(i, j).unwrap().unwrap().color()
                                == Color::White
                            {
                                1
                            } else {
                                -1
                            };
                        let piece_type = board
                            .occupant_of_square(i, j)
                            .unwrap()
                            .unwrap()
                            .piece_type();
                        match piece_type {
                            PieceType::P => pieces.pawns[index] = piece_value,
                            PieceType::B => pieces.bishops[index] = piece_value,
                            PieceType::N => pieces.knights[index] = piece_value,
                            PieceType::R => pieces.rooks[index] = piece_value,
                            PieceType::K => pieces.kings[index] = piece_value,
                            PieceType::Q => pieces.queens[index] = piece_value,
                        }
                    }
                    index += 1;
                }
            }
            println!("{}", pieces.print_boards());
        }

        board.make_move_san(&arithmetic_move).expect("ERRORRRR");
    }

    println!("Games: {}", games.len());
    println!("{}", games[0].pgn);
    println!("{:?}", games[0].moves);
    Ok(())
}
