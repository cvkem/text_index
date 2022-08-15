use std::fs::File;
use std::env;
use std::io::{BufRead, BufReader,Write, stdout};
use std::time::Duration;
use crate::type_aux::type_of;
use crate::levenshtein::dam_lev_prefix;

extern crate crossterm;


use crossterm::{queue, QueueableCommand, cursor, execute, terminal, 
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

mod count_only;
mod index;
mod levenshtein;


#[derive(PartialEq)]
enum InputStatus {
    Quit,
    Changed,
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
        let word_index = index::build_index(BufReader::new(File::open(filename).expect("Cannot open file.")));

        terminal::enable_raw_mode()?;

        let mut search_str = String::default();
        let mut most_likely_completion = String::default();    
        loop {
            let status = get_input(&mut search_str, &most_likely_completion)?;
            if status == InputStatus::Quit {
                break;
            }

            if status == InputStatus::Changed {
                let compl_rec = index::find_completions(&word_index, &search_str, 7);
                if compl_rec.compl.len() > 0 {
                    most_likely_completion = compl_rec.compl[0].completion.clone();
                } else {
                    most_likely_completion = String()::default();
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

    queue!(stdout,  cursor::MoveTo(0, 3), terminal::Clear(terminal::ClearType::CurrentLine));
    let completion_suffix: String = completion.chars().skip((&search_str).chars().count()).collect();
    let sstr = search_str.clone();
    let len_compl_suffix = completion_suffix.len() as u16;
    //println!("{}{}{}", "SEARCH: ".bold(), search_str.as_ref::<String>().blue(), completion_suffix.grey());
    print!("{}{}{}", "SEARCH: ".bold(), sstr.blue(), completion_suffix.grey());
    queue!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown), cursor::MoveLeft(len_compl_suffix));
    stdout.flush();


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
        stdout.flush();
    }

    return Ok(InputStatus::None);
}

