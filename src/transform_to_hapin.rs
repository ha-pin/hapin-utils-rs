use std::slice::SliceIndex;

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
            return  "".to_owned();
        }

        while self._idx < self._word.chars().count() as i32 {
            // TODO 获取字串字符
            // let c = ;
        }

        return self._res.to_owned();
    }
}

/**
 * Tranform to hapin
 */
pub fn transform_to_hapin(o: &str, easy: bool) -> &str {
    let o_array: Vec<&str> = o
        .split(' ')
        .map(|item| item.trim())
        .filter(|item| item.len() != 0)
        .collect();
    println!("{:?}", o_array);
    ""
}
