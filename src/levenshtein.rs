// This module computes the Damerau–Levenshtein distance.
//
// The normal Levenshtein distance only considers insertions and deletions. The Damerau-Levenshtein distance is a metric also considers transposition of adjacent characters. 
//
// It operates on a prefix as it is intended to be used in a context of a search-tool, where the user might only have input part of the string to be searched.

use std::cmp;

pub fn dam_lev_prefix(prefix_str: &str, word_str: &str, max_dist: usize) -> Option<usize> {
    // Compute the Damerau-Levenshtein for a prefix up to a maximum. The return value is 0 if the strings are equal, otherwise it is the actual distance or None.
    // The None value signals the distance exceeds the 'max_dist'.
    //println!("\nHandling {prefix_str}  for  word {word_str}.");
    let mut dist = match prefix_str.len() as i32 - word_str.len() as i32 {
                    prefix_excess_len if prefix_excess_len > max_dist as i32 => return None,
                    prefix_excess_len if prefix_excess_len > 0  => prefix_excess_len as usize,
                    _ => 0 };
    
    let prefix: Vec<char> = prefix_str.chars().collect();
    let word = word_str.chars().collect::<Vec<char>>(); 
    
    let mut skip_char = false;
    let mut num_deletions = 0;
    for idx in 0..cmp::min(prefix.len(), word.len() - num_deletions) {
        if dist > max_dist {
            return None;
        }

        if skip_char {
            skip_char = false;
            continue;
        }
        if prefix[idx] != word[idx - num_deletions] {
            //println!("Mismatch at position {} of {word:?} with current dist of {dist}", idx - num_deletions);
            let check_pos = idx - num_deletions + 1;
            if word.len() > check_pos  && prefix[idx] == word[check_pos] {
                // consider deletion or transpose
                //println!("Consider deletion or transpose on position {check_pos} in word {word:?}");
                if prefix.len() > idx+1 &&  prefix[idx + 1] == word[idx - num_deletions] {
                    // transpose possible
                    //println!{"Confirmed TRANSPOSE"};
                    skip_char = true;
                } else {
                    //println!{"Confirmed DELETE"};
                    // perform a deletion
                }
            } else {
                // perform a replace
                //println!("Replace at the position {} of {word:?}", idx - num_deletions);
            }
            dist += 1;            
        }
    }
    
    if dist > max_dist {
        return None;
    }

    Some(dist)
} 



#[cfg(test)]
mod tests {
    use super::dam_lev_prefix;
    
    #[test]
    fn test_dam_lev_prefix() {
        // equal strings
        assert_eq!(dam_lev_prefix("abc", "abc", 2), Some(0));
        // prefix is longer than word
        assert_eq!(dam_lev_prefix("abcX", "abc", 2), Some(1));
        assert_eq!(dam_lev_prefix("abcXY", "abc", 2), Some(2));
        assert_eq!(dam_lev_prefix("abcXYZ", "abc", 2), None);

        // prefix does not match with word, and word is longer
        assert_eq!(dam_lev_prefix("abc", "abc____", 2), Some(0));
        assert_eq!(dam_lev_prefix("abc", "Xbcdef", 2), Some(1));
        assert_eq!(dam_lev_prefix("abc", "Xbcdef", 2), Some(1));
        assert_eq!(dam_lev_prefix("abc", "aXcdef", 2), Some(1));
        assert_eq!(dam_lev_prefix("abc", "XYcdef", 2), Some(2));
        assert_eq!(dam_lev_prefix("abc", "XYZdef", 2), None);

        // a single transposition (swap) is counted, instead of 2 replace statements. 
        assert_eq!(dam_lev_prefix("abc", "acb____", 2), Some(1));
    }
}
