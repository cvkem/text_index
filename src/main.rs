use std::fs::File;
use std::env;
use std::io::{BufRead, BufReader,Write, stdout};
use std::time::Duration;
use crate::type_aux::type_of;
use crate::levenshtein::dam_lev_prefix;

extern crate crossterm;


use crossterm::{queue, QueueableCommand, cursor, execute, terminal,
        cursor::{SavePosition, RestorePosition}, 
            style::Stylize,
            event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers}, 
            Result};
//    terminal::{clear, ClearType}};

// use crossterm::{
// //    event::{self, Event as CEvent, KeyCode},
//     terminal::{disable_raw_mode, enable_raw_mode},
// };


mod time_aux;
mod type_aux;

mod index;
mod levenshtein;

use index::Completion;


#[derive(PartialEq)]
enum InputStatus {
    Quit,
    Changed,
    ShowResults,
    None
}

fn search_file_via_console(filename: &str) -> Result<()> {
    let mut stdout = stdout();

    queue!(stdout,  cursor::MoveTo(0, 0), terminal::Clear(terminal::ClearType::All));

    // some other code ...
    println!("{}", "Building the index".magenta()); 

    // move operation is performed only if we flush the buffer.
    stdout.flush();

    execute!(stdout, EnableMouseCapture)?;

    {
        let word_index = index::WordIndex::build_index(BufReader::new(File::open(filename).expect("Cannot open file.")));
        let num_completions = 10;

        // let res = word_index.bt.get("the").unwrap().len();
        // println!("The word 'the' has {} occurences", res);

        // return Ok(());

        queue!(stdout,  cursor::MoveTo(0, 0), terminal::Clear(terminal::ClearType::All));
        println!("{}", format!("Index compressed {} records containing {} words to an index of {} items in {:?}", 
            word_index.record_count, word_index.word_count, word_index.len(), word_index.duration).magenta()); 

        let mut row: u16 = 0;
        terminal::enable_raw_mode()?;

        let mut search_str = String::default();
        let mut most_likely_completion = String::default();
        loop {
            let status = get_input(&mut search_str, &most_likely_completion)?;

            match status {
            InputStatus::Quit => break,
            InputStatus::ShowResults => {
                queue!(stdout, cursor::MoveTo(0, row));
                // queue!(stdout, cursor::MoveTo(0, row), terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, row));
                print!("{}", format!("Locations of the word '{}':\r\n", &search_str).magenta());

                match word_index.find_matches(&search_str) {
                    Some(occurrences) => {
                        print!("\r\nObserved {} instances of '{}'\r\n", &occurrences.len(), &search_str);
                        for (idx, oc) in occurrences.iter().enumerate() {
                            print!("{}: {:?}\r\n", idx, oc);
                        }
                    },
                    None => print!("No matches of '{}' found.\r\n", &search_str)
                };
                break
            },
            InputStatus::None => continue,
            InputStatus::Changed => {
                let compl_rec = word_index.find_completions(&search_str, num_completions);
                queue!(stdout,  cursor::MoveTo(0, 4), terminal::Clear(terminal::ClearType::FromCursorDown));
                print!("{}", format!("Search for completions completed in {:?}\r\n", compl_rec.duration).green());
                
                if compl_rec.compl.len() > 0 {
                    most_likely_completion = compl_rec.compl[0].completion.clone();
                    for (idx, Completion{completion, count}) in compl_rec.compl.iter().enumerate() {
                        print!("{}: completion '{}' occurs  {} times\r\n", idx + 1, completion, count);
                    }
                } else {
                    most_likely_completion = String::default();
                }
                print!("\r\n\r\n");
                stdout.flush().unwrap();

                {
                    let num_chars = search_str.chars().count();
                    if num_chars <= 1 {
                        print!("{}", "need at least two letter to compute Damerau–Levenshtein distance.\r\n".green());
                        continue;
                    }
                    let max_dist = if num_chars > 3 {2} else {1};

                    execute!(stdout, SavePosition);
                    let compl_rec_dl = word_index.find_dl_completions(&search_str, num_completions, max_dist);
                    execute!(stdout, cursor::RestorePosition, terminal::Clear(terminal::ClearType::FromCursorDown));

                    print!("{}", format!("Search for Damerau–Levenshtein (max_dist={}) completed in {:?}\r\n", max_dist, compl_rec_dl.duration).green());
                    if compl_rec_dl.compl.len() > 0 {
                        for (idx, Completion{completion, count}) in compl_rec_dl.compl.iter().enumerate() {
                            print!("{}: completion '{}' occurs  {} times\r\n", idx + 1, completion, count);
                        }
                    }
                    print!("\r\n");
                }
                (_, row) = cursor::position().unwrap();
            }
        }
        }
        terminal::disable_raw_mode();

    } 
    execute!(stdout, DisableMouseCapture)?;

    return Ok(()); //temporary
    
}

fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();
    let default_filename = "t8.shakespeare.txt".to_string();  // define as is will be a temporary inside unwrap_or
    let filename = args.get(1).unwrap_or(&default_filename);

    search_file_via_console(filename);

    return Ok(());
}


fn get_input(search_str: &mut String, completion: &String) -> crossterm::Result<InputStatus> {
    // prints the key-codes in an event-loop. Als catches CTRL-C so use <ESC> to get out.
    let mut stdout = stdout();

    queue!(stdout,  cursor::MoveTo(0, 2), terminal::Clear(terminal::ClearType::CurrentLine));
    print!("Special commands: Tab=Accept completion, Enter=Search-locations, CTRL-C=quit program\r\n");
    let completion_suffix: String = completion.chars().skip((&search_str).chars().count()).collect();
    let sstr = search_str.clone();
    let len_compl_suffix = completion_suffix.len() as u16;
    //println!("{}{}{}", "SEARCH: ".bold(), search_str.as_ref::<String>().blue(), completion_suffix.grey());
    print!("{}{}{}", "SEARCH: ".bold(), sstr.blue(), completion_suffix.grey());
    queue!(stdout, cursor::MoveLeft(len_compl_suffix));
    stdout.flush().unwrap();


    if poll(Duration::from_millis(500))? {
        // It's guaranteed that the `read()` won't block when the `poll()`
        // function returns `true`
        let event = read()?;

        //println!("Event {:?} of type {} is not {:?} of type {}", event,  type_of(&event), KeyCode::Backspace, type_of(&KeyCode::Backspace));

        match event {
            Event::FocusGained => (),
            Event::FocusLost => (),
            Event::Key(event) if event == KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(InputStatus::Quit),
            Event::Key(event) if event == KeyCode::Esc.into() =>  return Ok(InputStatus::Quit),
            Event::Key(event) if event == KeyCode::Tab.into() =>  {
                search_str.push_str(&completion.chars().skip((&search_str).chars().count()).collect::<String>());
                return Ok(InputStatus::Changed);
            },
            Event::Key(event) if event == KeyCode::Enter.into() =>  {
                return Ok(InputStatus::ShowResults);
            },
            Event::Key(event) if event == KeyCode::Backspace.into() => {
                search_str.pop();
                return Ok(InputStatus::Changed);
            },
            Event::Key(event) if event == KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                search_str.pop();
                return Ok(InputStatus::Changed);
            },
            //Event::Key(KeyEvent{KeyCode::Char(ch), _, _, _})  => {
            Event::Key(KeyEvent{code, ..})  => {
                if let KeyCode::Char(ch) = code {
                    search_str.push(ch);
                    return Ok(InputStatus::Changed);
                } else {
                    println!("Ignoring code {:?}", code);
                    return Ok(InputStatus::None);
                }
            },
            Event::Mouse(event) => println!("{:?}", event),
//                #[cfg(feature = "bracketed-paste")]
            Event::Paste(data) => println!("Pasting: {}", data),
            Event::Resize(width, height) => ()
        }
        queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown), cursor::MoveLeft(len_compl_suffix)).unwrap();
        stdout.flush().unwrap();
    }

    return Ok(InputStatus::None);
}

