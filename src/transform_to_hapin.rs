use crate::utils;

pub struct TransformToHapin {
    _weak_tone: bool,
    _word: String,
    _idx: i32,
    _res: String,
}

impl TransformToHapin {
    /**
     * Initial TranformToHapin
     */
    pub fn new(word: String) -> TransformToHapin {
        TransformToHapin {
            _weak_tone: false,
            _word: word,
            _idx: 0,
            _res: String::new(),
        }
    }

    /**
     * Add Spearator
     */
    fn add_spearator(&mut self, easy: bool) {
        if easy && self._idx != self._word.chars().count() as i32 - 1 {
            self._res.push('\'')
        }
    }

    /**
     * Get Result
     */
    pub fn go(&mut self, easy: bool) -> String {
        if self._word.chars().count() == 0 {
            return "".to_owned();
        }

        while self._idx < self._word.chars().count() as i32 {
            // TODO 获取子串字符
            let c = utils::get_char(&self._word, self._idx as usize) as u32;
        }

        return self._res.to_owned();
    }
}

// fn handle_special_chars(c: &str) -> &str {
//     c.chars().map(|i| -> &str {
//         // 获取 Unicode
//         i.to_string()
//     })
// }

// /**
//  * Tranform to hapin
//  */
// pub fn transform_to_hapin(o: &str, easy: bool) -> &str {
//     let o_array = o
//         .split(' ')
//         .map(|item| item.trim())
//         .filter(|item| item.len() != 0)
//         .map(|item| TransformToHapin::new().go(easy));
//     println!("{:?}", o_array);
//     ""
// }
