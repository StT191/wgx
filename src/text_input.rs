
#[derive(Debug)]
pub struct SimpleTextInput {
    text: String,
    curser: usize,
}


impl SimpleTextInput {

    pub fn new(text: &str) -> Self {
        Self { text: String::from(text), curser: 0 }
    }

    // getters
    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn curser(&self) -> usize {
        self.curser
    }

    pub fn text<'a>(&'a self) -> &'a str {
        &self.text[..]
    }

    pub fn text_before_curser<'a>(&'a self) -> &'a str {
        &self.text[..self.curser]
    }

    pub fn text_after_curser<'a>(&'a self) -> &'a str {
        &self.text[self.curser..]
    }


    // curser moves
    pub fn advance(&mut self) -> bool {
        if self.curser != self.text.len() {
            loop {
                self.curser += 1;
                if self.text.is_char_boundary(self.curser) { break; }
            }
            true
        }
        else { false }
    }

    pub fn set_curser(&mut self, positon: usize) -> bool {
        let was = self.curser;
        self.curser = 0;
        for _ in 0..positon {
            if !self.advance() { break }
        }
        self.curser != was
    }

    pub fn set_curser_end(&mut self) -> bool {
        if self.curser != self.text.len() {
            self.curser = self.text.len();
            true
        }
        else { false }
    }

    pub fn recede(&mut self) -> bool {
        if self.curser != 0 {
            loop {
                self.curser -= 1;
                if self.text.is_char_boundary(self.curser) { break; }
            }
            true
        }
        else { false }
    }


    // char insertion
    pub fn insert(&mut self, character:char) -> bool {
        match (character, character.is_control()) {
            ('\n', _) | ('\r', _) | (_, false) =>
            {
                self.text.insert(self.curser, character);
                self.advance();
                true
            },
            ('\t', _) => {
               self.text.insert_str(self.curser, "    ");
               self.curser += 4;
               true
            }
            _ => false
        }
    }


    pub fn insert_str(&mut self, text: &str) {
        self.text.insert_str(self.curser, text);
    }


    // deletion
    pub fn remove(&mut self) -> bool {
        if self.curser != 0 {
            let end = self.curser;
            self.recede();
            self.text.replace_range(self.curser..end, "");
            true
        }
        else { false }
    }

    pub fn delete(&mut self) -> bool {
        if self.curser != self.text.len() {
            let start = self.curser;
            self.advance();
            self.text.replace_range(start..self.curser, "");
            self.recede();
            true
        }
        else { false }
    }
}