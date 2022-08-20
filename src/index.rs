use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::{time::{Instant, Duration}, 
        io::{Write, stdout}};
use crate::time_aux::saved_duration;
use crate::type_aux::*;


//mod super::levenshtein;

pub struct WordIndex {
    bt: BTreeMap<String, Vec<WordLoc>>,
    pub duration: Duration,
    pub record_count: usize,
    pub word_count: usize
}

impl WordIndex {
    pub fn len(&self) -> usize {
        self.bt.len()
    }

    pub fn find_matches(&self, search_str: &str) -> Option<&Vec<WordLoc>> {
        self.bt.get(search_str)
    }

    pub fn build_index(reader: BufReader<File>) -> WordIndex {
        let mut word_index = BTreeMap::new();
    
        let start = Instant::now();   
        //    println!("Dynamic usage of tree is {}", word_count.dynamic_usage());
        // adding all words to the word-count
        let mut record_count = 0;
        let mut word_count = 0;
        let mut stdout = stdout();
        for (line_idx, line) in reader.lines().enumerate() {
            record_count += 1;
            for (word_idx, word) in line
                                    .unwrap()
    //                                .to_lowercase()
                                    .split_whitespace()
                                    .map(remove_interpunction)
                                    .enumerate() {
                word_count += 1;
                let w_string = word.to_string();
                let WordLoc = WordLoc{line: line_idx as u32, word: word_idx as u16};
                (*word_index.entry(w_string).or_insert(Vec::new())).push(WordLoc);
            }
            if record_count % 1000 == 0 {
                print!(".");
                stdout.flush();
            }
        }
        let duration = start.elapsed();
        println!("\nTime elapsed to index the full file with {} lines and {} words. Duration: {:?}", record_count, word_count, duration);
    
        return WordIndex{bt: word_index, duration, record_count, word_count};
    }
    

    pub fn find_completions(&self, check_word: &String, num_completions: usize) -> CompletionsRec {
        // Find the 'num_completions'  completions that are most common in the indexed text.
        let mut end_range: String = check_word.clone();
        end_range.push_str("zzzzzzzz");
    
        let start = Instant::now();   
        let mut completions_rec: CompletionsRec = self.bt
                .range(check_word.to_owned()..end_range)
                .fold(CompletionsRec::new(num_completions), top_completions).into();
        let duration = start.elapsed();
        completions_rec.duration = duration;
        println!("\nTime elapsed to compute completions is + size: {:?}", duration);
        println!("\tNew results:\n\t{completions_rec:?}");
    
        completions_rec
    }

    pub fn find_dl_completions(&self, check_word: &String, num_completions: usize, max_dist: usize) -> CompletionsRec {
        // look over full index for words that are within 'max_dist' and order by frequency.
        use crate::levenshtein::dam_lev_prefix;
    
        let start = Instant::now();   
        let mut completions_rec: CompletionsRec =  self.bt
            .iter()
            .filter(|&(s, _)| !s.starts_with(check_word)) 
            .fold(CompletionsRec::new(num_completions), |state, kv| if let Some(_) = dam_lev_prefix(check_word, kv.0, max_dist) {top_completions(state, kv)} else {state});
        let duration = start.elapsed();
        completions_rec.duration = duration;
        println!("Time elapsed {:?}\n", duration);
    
        completions_rec
    }
    
    
}



pub fn build_indexes_from_file_name(filename: &str, num_btrees: u32) -> Vec<WordIndex> {
    // only for testing purposes.  Measure memory rqequirements as a function of number of btrees.
    let mut store = Vec::new();
    for _ in 0..num_btrees {
        let reader = BufReader::new(File::open(filename).expect("Cannot open file."));
        store.push(WordIndex::build_index(reader))
    }
    return store;
}



#[derive(Debug)]
pub struct WordLoc {
    line: u32,
    word: u16  // the size of this struct is 8 bytes anyway due to alignment, so this could also be a u32
}

#[derive(Debug)]
pub struct Completion {
    pub completion: String,
    pub count: usize
}

#[derive(Debug)]
pub struct CompletionsRec {
    pub compl: Vec::<Completion>,
    pub total_count: usize,
    pub duration: Duration
}

trait NewCompl {
    fn new(num_compl: usize) -> Self;
}

impl NewCompl for CompletionsRec {
    fn new(num_compl: usize) -> Self {
        CompletionsRec{ compl: Vec::<Completion>::with_capacity(num_compl),
            total_count: 0,
            duration: Duration::default()}
    }
}


fn top_completions(mut state: CompletionsRec, kv: (&String, &Vec<WordLoc>)) -> CompletionsRec {
    // find the series of most frequent completions where the number of completions selected is state.compl.capacity and count the total number of completions.
    // internal function to be mapped over a iterable with results.
    state.total_count += 1;
    if state.compl.len() < state.compl.capacity() || state.compl[state.compl.capacity() -1].count < (*kv.1).len() {
        if state.compl.len() == state.compl.capacity() {
            _ = state.compl.pop();
        }

        // the new completion should be inserted in the proper position to retain ordering
        let count = (*kv.1).len();
        let new_compl = Completion{completion: kv.0.clone(), count};
        for i in (0..=state.compl.len()).rev() {
            if i == 0 || count <= state.compl[i-1].count  {  // when i==0 we insert at first position
                //println!("For completions={:?} adding {:?} at position {}", &state, &new_compl, i);
                state.compl.insert(i, new_compl);
                break;
            }
        }
    } 
    state
}





fn remove_interpunction(s: &str) -> String {
    // for a word remove heading and trailing interpunction
    let chs: Vec<char> = s.chars().collect();
    
    let mut start_idx = if chs[0] == '"' || chs[0] == '\u{27}' || chs[0] == '[' || chs[0] == '(' || chs[0] == '{' {1} else {0};
    let end_idx = match chs[chs.len()-1] {
        ';' | '.' | ',' | '"' | '\u{27}' | '?' | '!' | ')' | ']' | '}'=> chs.len() - 1,
        _ => chs.len()
    };

    if start_idx >= end_idx {
        String::default()
    } else {
        chs[start_idx..end_idx].iter().collect::<String>()
    }
}


#[cfg(test)]
mod tests {
    use super::{find_completions_internal, WordLoc, CompletionsRec};
    use std::time::Duration;

    #[test]
    fn test_find_completions() {
        let state = CompletionsRec{ compl: Vec::<super::Completion>::with_capacity(2), total_count: 0, duration: Duration::default()};

        // add the first item to 'state'
        let state = find_completions_internal(state, (&"initial-value".to_string(), &(vec!(WordLoc{line: 1, word: 1}, WordLoc{line: 2, word: 2}, WordLoc{line: 3, word: 3}))));
        assert_eq!(state.compl[0].count, 3);
        // // add the second item to 'state'
        let state = find_completions_internal(state, (&"at end".to_string(), &vec!(WordLoc{line: 3, word: 3})));
        assert_eq!(state.compl[0].count, 3);
        assert_eq!(state.compl[1].count, 1);
        // and append a third item
        let state = find_completions_internal(state, (&"at start".to_string(), &vec!(WordLoc{line: 4, word: 3}, WordLoc{line: 5, word: 3}, WordLoc{line: 6, word: 3}, WordLoc{line: 7, word: 3})));
        assert_eq!(state.compl[0].count, 4);
        assert_eq!(state.compl[1].count, 3);
    }
}



pub fn test_index(reader: BufReader<File>) {
 
    let mut word_count = WordIndex::build_index(reader);

    println!("The datastructure contains {} items", word_count.bt.len());
    println!(
        "After indexing: Current size_of={}  and size_of_val={}",
        mem::size_of::<BTreeMap<String, i32>>(),
        mem::size_of_val(&word_count)
    );
    //    println!("Dynamic usage of tree is {}", word_count.dynamic_usage());
    let check_word = "the".to_string();
    match word_count.bt.get(&check_word) {
        Some(occurences) => println!("the word '{check_word}'  appeared {} times.", occurences.len()),
        None => println!("Could not find the word '{check_word}'!"),
    }

    let check_word = "the".to_string();
    let cr = word_count.find_completions(&check_word, 10);

    // let start4 = Instant::now();   
    // let check_word = "the".to_string();
    // let end_range = "thf".to_string();
    // let mut cnt = 0i32;
    // let longest_completion = word_count.bt
    //         .range(check_word..end_range)
    //         .fold(String::new(), |longest, kv| if kv.0.len() > longest.len() {kv.0.clone()} else {longest});
    // let duration4 = start4.elapsed();
    // println!("\nTime elapsed to find longest completion is: {:?}", duration4);
    // println!("\tNew results:\n\t{longest_completion}");


    {
        use crate::levenshtein::dam_lev_prefix;

        let full_check_word = "themselves_";
        let max_dist = 2;
        for curr_len in 3..full_check_word.len() {
            let check_word =  &full_check_word[..curr_len];
            println!("Computing number strings within a dist 2 of '{check_word}'  (length = {curr_len})");
            let start5 = Instant::now();   
            let (num_dl_match, num_total) = word_count.bt
                .iter()
                .fold((0, 0), |(num, num_total), kv| if let Some(_) = dam_lev_prefix(check_word, kv.0, max_dist) {(num+1, num_total+1)} else {(num, num_total+1)});
            let duration5 = start5.elapsed();
            let fraction = 100.0 * num_dl_match as f64/(num_total as f64);
            println!("Time elapsed {:?} and found {num_dl_match} entries out of {num_total} at distance {max_dist} ({fraction:.1}%)\n", duration5);
            }
    }
}
