use crate::utils;

struct ArabicToHapinTransformer {
	_weak_tone: bool,
	_word: String,
	_index: u32,
	_res: String,
}

impl ArabicToHapinTransformer {
	pub fn new(word: String) -> ArabicToHapinTransformer {
		ArabicToHapinTransformer {
			_weak_tone: false,
			_word: word,
			_index: 0,
			_res: String::new(),
		}
	}

	pub fn go(&mut self) -> String {
		if utils::utf8::get_string_length(&self._word) == 0 {
			return String::new();
		}

		let mut res = String::new();
		while self._index < utils::utf8::get_string_length(&self._word) {
			let c = utils::utf8::get_char(&self._word, self._index) as u32;
			// 获取字符unicode比较
			if c == 1652 || c == 1569 {
				self._weak_tone = true;
				self._index += 1;
				continue;
			}

			// 处理元音
		}

		return res;
	}
}