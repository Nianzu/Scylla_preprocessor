use rschess::{Board, Color, PieceType};
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process::exit;

static MIN_ELO: u16 = 2_000;
static MAX_GAMES: usize = 20_000;

struct BitBoards {
    pawns: Vec<i8>,
    bishops: Vec<i8>,
    knights: Vec<i8>,
    rooks: Vec<i8>,
    kings: Vec<i8>,
    queens: Vec<i8>,
    piece_selected: Vec<i8>,
    destination: Vec<i8>,
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
            destination: vec![0; 64],
        }
    }

    fn export_board(board: Vec<i8>, f: &mut File) {
        let mut output = "".to_owned();
        for i in 0..64 {
            output += &format!("{:2}", board[i]);
            output += ",";
        }
        f.write_all(output.as_bytes())
            .expect("Issue writing to file");
    }
    fn export_boards(&self, f: &mut File, destination: bool) {
        BitBoards::export_board(self.pawns.clone(), f);
        BitBoards::export_board(self.bishops.clone(), f);
        BitBoards::export_board(self.knights.clone(), f);
        BitBoards::export_board(self.rooks.clone(), f);
        BitBoards::export_board(self.queens.clone(), f);
        BitBoards::export_board(self.kings.clone(), f);
        f.write_all("\n".as_bytes()).expect("Issue writing to file");
        if destination {
            BitBoards::export_board(self.destination.clone(), f);
        } else {
            BitBoards::export_board(self.piece_selected.clone(), f);
        }
        f.write_all("\n".as_bytes()).expect("Issue writing to file");
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
        output += "Queens\n";
        output += &BitBoards::print_board(self.queens.clone());
        output += "Kings\n";
        output += &BitBoards::print_board(self.kings.clone());
        output += "Piece Selected\n";
        output += &BitBoards::print_board(self.piece_selected.clone());
        output += "Destination\n";
        output += &BitBoards::print_board(self.destination.clone());
        output
    }
}

struct Game {
    white_elo: Option<u16>,
    black_elo: Option<u16>,
    pgn: String,
    moves: Vec<String>,
}

impl Game {
    fn new() -> Game {
        Game {
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
        let beginning = Game::remove_brackets(self.pgn.clone())
            .replace("?", "")
            .replace("!", "");
        let mut intermediate: Vec<&str> = beginning.split(" ").collect();
        intermediate.retain(|value| !value.contains(".") && !value.is_empty());
        let end: Vec<String> = intermediate
            .into_iter()
            .map(|value| value.to_owned())
            .collect();
        self.moves = end.split_last().unwrap().1.to_vec();
        self
    }

    fn process_moves(
        &self,
        piece_selector_file: &mut File,
        pawn_file: &mut File,
        bishop_file: &mut File,
        knight_file: &mut File,
        rook_file: &mut File,
        queen_file: &mut File,
        king_file: &mut File,
    ) {
        let mut board = Board::default();
        for arithmetic_move in self.moves.clone() {
            let side_to_move = board.side_to_move();
            let move_being_made = board.san_to_move(&arithmetic_move).expect("ERRORRRRRRRRR");
            // println!("Side to move: {}", side_to_move);
            // println!("Move being made: {:?}", move_being_made.from_square());
            if side_to_move == Color::White {
                let mut pieces = BitBoards::new();

                let src_i = move_being_made.from_square().0 as usize - 97;
                let src_j = move_being_made.from_square().1 as usize - 49;
                pieces.piece_selected[src_i + ((7 - src_j) * 8)] = 1;

                let dest_i = move_being_made.to_square().0 as usize - 97;
                let dest_j = move_being_made.to_square().1 as usize - 49;
                pieces.destination[dest_i + ((7 - dest_j) * 8)] = 1;

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
                // println!("{}", pieces.print_boards());
                pieces.export_boards(piece_selector_file, false);
                match board
                    .occupant_of_square(
                        move_being_made.from_square().0,
                        move_being_made.from_square().1,
                    )
                    .unwrap()
                    .unwrap()
                    .piece_type()
                {
                    PieceType::P => pieces.export_boards(pawn_file, true),
                    PieceType::B => pieces.export_boards(bishop_file, true),
                    PieceType::N => pieces.export_boards(knight_file, true),
                    PieceType::R => pieces.export_boards(rook_file, true),
                    PieceType::K => pieces.export_boards(king_file, true),
                    PieceType::Q => pieces.export_boards(queen_file, true),
                }
            }

            board.make_move_san(&arithmetic_move).expect("ERRORRRR");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please include a path to the pgn you want to use");
        return Ok(());
    }

    let mut piece_selector_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/piece_selector.csv")
        .expect("Unable to create file");
    let mut pawn_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/pawn.csv")
        .expect("Unable to create file");
    let mut bishop_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/bishop.csv")
        .expect("Unable to create file");
    let mut knight_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/knight.csv")
        .expect("Unable to create file");
    let mut rook_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/rook.csv")
        .expect("Unable to create file");
    let mut queen_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/queen.csv")
        .expect("Unable to create file");
    let mut king_file = OpenOptions::new()
        .read(true)
        .write(true) // <--------- this
        .create(true)
        .truncate(true)
        .open("../Scylla/datasets/chess_2000/king.csv")
        .expect("Unable to create file");

    let mut last_line_type = "moves";
    let file = File::open(&args[1]).expect("Error opening file");

    let reader = BufReader::new(file);
    let mut current_game = Game::new();
    let mut num_games = 0;

    for line_obj in reader.lines() {
        let line = line_obj?;
        let mut current_line_type = "moves";
        if line.is_empty() {
            continue;
        }
        if line.chars().next().expect("Issue getting first char") == '[' {
            current_line_type = "tags";
            let (name, content) = Game::get_tag(&line);
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
                current_game.process_moves(
                    &mut piece_selector_file,
                    &mut pawn_file,
                    &mut bishop_file,
                    &mut knight_file,
                    &mut rook_file,
                    &mut queen_file,
                    &mut king_file,
                );
                num_games += 1;
                if num_games % 100 == 0 {
                    println!("{}", num_games);
                }

                // Debug code
                if num_games >= MAX_GAMES {
                    break;
                }
            }

            current_game = Game::new();
        }
        last_line_type = current_line_type;
    }

    println!("Games: {}", num_games);
    Ok(())
}
