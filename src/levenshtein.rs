// This module computes the Damerau–Levenshtein distance.
//
// The normal Levenshtein distance only considers insertions and deletions. The Damerau-Levenshtein distance is a metric also considers transposition of adjacent characters. 
//
// It operates on a prefix as it is intended to be used in a context of a search-tool, where the user might only have input part of the string to be searched.

use std::cmp;

pub fn dam_lev_prefix(prefix_str: &str, word_str: &str, max_dist: usize) -> Option<usize> {
    // Compute the Damerau-Levenshtein for a prefix up to a maximum. The return value is 0 if the strings are equal, otherwise it is the actual distance or None.
    // The None value signals the distance exceeds the 'max_dist'.
    //
    // Note: this version does not do full backtracking, so it might miss out on complex patterns. We should make a recursive version with full back-tracking.
    let prefix: Vec<char> = prefix_str.chars().collect();
    let word = word_str.chars().collect::<Vec<char>>(); 
    
    let mut dist = 0;

    let mut skip_char = false;
    let mut word_offset: isize = 0; // When positive a character from word is deleted when negative a character from prefix is deleted
    for idx in 0..prefix.len() {
        if dist > max_dist {
            return None;
        }

        if skip_char {
            skip_char = false;
            continue;
        }
        let word_idx = (idx as isize + word_offset) as usize;
        if word_idx >= word.len() {
            dist += 1;
            continue;
        }
        if prefix[idx] != word[word_idx] {
            dist += 1;            

            let check_word_idx = word_idx + 1;            
            if word.len() > check_word_idx  && prefix[idx] == word[check_word_idx] {
                // consider deletion in word, a transpose.
                if  prefix.len() > idx+1 && prefix[idx + 1] == word[word_idx] {
                    // transpose possible and solves issue in current and next index, so skip next index to prevent double-counting
                    skip_char = true;
                    } else {
                        // drop a character from prefix
                        word_offset += 1 
                    } 
                } else {
                    // consider a drop of a character from prefix or a replace
                    if prefix.len() > idx+1 &&  prefix[idx + 1] == word[word_idx] {
                        //drop a character from prefix
                        word_offset -= 1;
                        skip_char = true;  //next iteration will success anyway (but this is not necessary)
                    }
                    else {
                        // assume a replace
                    }

                }
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
        assert_eq!(dam_lev_prefix("abc", "acdef", 2), Some(1)); // one deletion is better than two replaces
        assert_eq!(dam_lev_prefix("abc", "acdef", 2), Some(1)); // one deletion is better than two replaces
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
