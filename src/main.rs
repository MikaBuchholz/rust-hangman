use colored::*;
use reqwest;
use std::io::stdout;
use std::io::{self, Write};
use std::{env, process};

struct Game {
    word_array: Vec<String>,
    word: String,
    word_length: usize,
    guessed_letters: Vec<String>,
    wrong_guesses: Vec<String>,
    correct_guesses: Vec<String>,
    remaining_guesses: usize,
    game_won: bool,
    replay: bool,
    cheat_mode: bool,
}

impl Game {
    fn new(word_array: Vec<String>, word: String, cheat_mode: bool) -> Game {
        let word_length = word_array.len();

        return Game {
            word_array,
            word,
            word_length,
            guessed_letters: Vec::new(),
            wrong_guesses: Vec::new(),
            correct_guesses: vec!["_".to_string(); word_length],
            remaining_guesses: 10,
            game_won: false,
            replay: false,
            cheat_mode,
        };
    }

    pub async fn play(&mut self) {
        self.display_word_length();
        self.display_gusses_left();
        self.display_wrong_guesses();

        while self.remaining_guesses > 0 {
            if self.cheat_mode {
                println!(
                    "{} | {}",
                    format!("CHEAT MODE ON!").bright_magenta(),
                    format!("WORD IS: {}", self.word).bold().bright_cyan()
                )
            }
            if self.replay {
                self.display_word_length();
            }

            if !self.game_won {
                let mut guess = String::new();
                print!("Guess a letter or word: ");

                let _ = stdout().flush();

                io::stdin().read_line(&mut guess).unwrap();

                let guess_length = guess.chars().count() - 2;
                guess = guess.replace("\r\n", "").to_lowercase();

                if (guess_length == self.get_word_length() || guess_length == 1)
                    && !guess.contains(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
                {
                    self.handle_guess(guess).await;

                    if !self.game_won {
                        self.display_gusses_left();
                        self.display_wrong_guesses();
                    }
                } else {
                    println!(
                        "{} {}",
                        format!("Wrong input format:").red(),
                        format!("{guess}").red().underline()
                    );
                }
            }
        }
    }
    pub fn display_word_length(&self) {
        let word_length = self.get_word_length();
        if !self.replay {
            println!(
                "Welcome to Hangman! The word is {} letters long",
                format!("{word_length}").blue()
            );
        } else {
            println!("This time its {} letters!", format!("{word_length}").blue());
        }
    }

    pub fn display_gusses_left(&self) {
        let guesses = self.remaining_guesses;
        print!("{} guesses left", format!("{guesses}").blue());
    }

    pub fn display_wrong_guesses(&self) {
        print!(" | Wrong guesses: ");
        print!("[");
        for (index, wrong_guess) in self.wrong_guesses.iter().enumerate() {
            if index == self.wrong_guesses.len() - 1 {
                if index == 0 {
                    print!("{}", format!("{}", wrong_guess.red()));
                } else {
                    print!(" {} ", format!("{}", wrong_guess.red()));
                }
            } else {
                print!(" {} |", format!("{}", wrong_guess.red()));
            }
        }
        print!("]");
        println!("");
    }

    pub fn insert_correct_letter(&mut self, correct_letter: &String) {
        let cleaned_correct_word = &correct_letter.replace("\r\n", "").to_string();

        for index in 0..self.word_length {
            if self.word_array[index].to_lowercase() == *cleaned_correct_word {
                self.correct_guesses[index] = self.word_array[index].to_string();
                if self.correct_guesses.len() != self.word_length {
                    self.correct_guesses.pop();
                }
            }
        }
    }

    pub async fn handle_guess(&mut self, guess: String) {
        println!("You guessed: {}", guess.bold());
        let lowercase_guess = &guess.replace("\r\n", "").to_lowercase();
        self.remaining_guesses = self.remaining_guesses - 1;

        if self.guessed_letters.contains(&guess) {
            println!(
                "{}`{}`",
                format!("Already guessed ").red(),
                format!("{}", &guess.red().underline())
            );
            return;
        }

        if lowercase_guess.eq(&self.word) {
            self.game_won().await;
            return;
        } else {
            if !(self.word.contains(lowercase_guess)) {
                let fail_message = lowercase_guess.red().underline();
                println!("{}", format!("Wrong! {fail_message} is not in word").red());

                self.wrong_guesses.push(lowercase_guess.to_string());
            } else {
                println!("{}", format!("Correct!").bright_green());
                self.correct_guesses.push(lowercase_guess.to_string());
                self.insert_correct_letter(&guess);
            }
            self.guessed_letters.push(lowercase_guess.to_string());
        }

        self.print_hangman_output();

        if !self.correct_guesses.contains(&"_".to_string()) {
            self.game_won().await;
            return;
        }
    }

    pub async fn game_won(&mut self) {
        self.game_won = true;
        println!("{}", format!("You won the game!").green());

        println!(
            "Play again? {}",
            format!("{} {}", "Yes".bright_blue(), "No".red())
        );

        let mut play_again = String::new();
        io::stdin().read_line(&mut play_again).unwrap();
        play_again = play_again.replace("\r\n", "");

        if play_again.to_lowercase() == "yes" {
            self.reset().await;
        } else if play_again.to_lowercase() == "no" {
            process::exit(0);
        }

        return;
    }

    pub fn print_hangman_output(&self) {
        for index in 0..self.word_length {
            print!("{}", self.correct_guesses[index].bold());
        }
        println!("");
    }

    pub async fn reset(&mut self) {
        let (word_array, random_word) = get_random_word().await;
        let word_length = word_array.len();

        self.word_array = word_array;
        self.word = random_word;
        self.word_length = word_length;
        self.correct_guesses = vec!["_".to_string(); word_length];
        self.wrong_guesses = Vec::new();
        self.remaining_guesses = 10;
        self.game_won = false;
        self.replay = true;
    }

    pub fn get_word_length(&self) -> usize {
        return self.word_length;
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let arg_amount = args.len();
    let mut cheat_mode = false;

    if arg_amount > 2 {
        println!("{}", format!("Too many arguments: {arg_amount}").red());
        process::exit(101);
    }

    if args.len() > 1 {
        if args[1] == "cheat" {
            cheat_mode = true;
        }
    }
    let (word_array, random_word) = get_random_word().await;
    let mut game = Game::new(word_array, random_word, cheat_mode);
    game.play().await;
}

async fn get_random_word() -> (Vec<String>, String) {
    let result = reqwest::get("https://api.api-ninjas.com/v1/randomword").await;

    let response_status = result.as_ref().unwrap().status();

    match response_status {
        reqwest::StatusCode::OK => {
            let uncleaned_word = result.unwrap().text().await.unwrap();
            let split_result: Vec<&str> = uncleaned_word.split(":").collect();

            let word = split_result[1]
                .replace(&['\"', '}', ' '][..], "")
                .to_lowercase();

            let mut char_array: Vec<String> = Vec::new();

            for letter in word.chars() {
                char_array.push(letter.to_string());
            }

            char_array[0] = char_array[0].to_uppercase();

            return (char_array, word);
        }
        _ => {
            println!("Error: {}", response_status);
            return (Vec::from(["".to_string()]), "".to_string());
        }
    }
}
