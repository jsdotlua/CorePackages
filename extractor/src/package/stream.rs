pub const DOWN_SYMBOL: &str = "│  ";
pub const TEE_SYMBOL: &str = "├──";
pub const ELL_SYMBOL: &str = "└──";

#[derive(Debug)]
pub enum Indent {
    Some,
    None,
}

#[derive(Debug)]
pub struct Stream {
    pub stream: String,
    pub indents: Vec<Indent>,
}

impl Stream {
    pub fn new() -> Self {
        Self {
            indents: Vec::new(),
            stream: String::new(),
        }
    }

    pub fn write_line(&mut self, str: &str, final_line: bool) {
        for text in str.split('\n') {
            if !self.stream.is_empty() {
                self.stream.push('\n');
            }

            let mut indents = self.indents.iter().peekable();
            while let Some(indent) = indents.next() {
                let is_last_indent = indents.peek().is_none();

                let symbol = match *indent {
                    Indent::None => "   ",
                    Indent::Some => match is_last_indent {
                        false => DOWN_SYMBOL,
                        true => match final_line {
                            true => ELL_SYMBOL,
                            false => TEE_SYMBOL,
                        },
                    },
                };

                self.stream.push_str(symbol);
            }

            self.stream.push_str(text);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::package::stream::Indent;

    use super::{Stream, DOWN_SYMBOL, ELL_SYMBOL, TEE_SYMBOL};

    #[test]
    fn expected_indentation_outputs() {
        let mut stream = Stream::new();
        let mut expected_stream = String::new();

        stream.write_line("Test1", false);
        expected_stream.push_str("Test1");
        assert_eq!(stream.stream, expected_stream);

        stream.write_line("Test2", false);
        expected_stream.push_str("\nTest2");
        assert_eq!(stream.stream, expected_stream);

        stream.indents.push(Indent::Some);
        stream.write_line("Test3", false);
        expected_stream.push_str(&format!("\n{TEE_SYMBOL}Test3"));
        assert_eq!(stream.stream, expected_stream);

        stream.write_line("Test4", false);
        expected_stream.push_str(&format!("\n{TEE_SYMBOL}Test4"));
        assert_eq!(stream.stream, expected_stream);

        stream.indents.push(Indent::Some);
        stream.write_line("Test5", false);
        expected_stream.push_str(&format!("\n{DOWN_SYMBOL}{TEE_SYMBOL}Test5"));
        assert_eq!(stream.stream, expected_stream);

        stream.write_line("Test6", true);
        expected_stream.push_str(&format!("\n{DOWN_SYMBOL}{ELL_SYMBOL}Test6"));
        assert_eq!(stream.stream, expected_stream);

        stream.indents.pop();
        stream.write_line("Test7", false);
        expected_stream.push_str(&format!("\n{TEE_SYMBOL}Test7"));
        assert_eq!(stream.stream, expected_stream);

        stream.indents.push(Indent::Some);
        stream.write_line("Test8", true);
        expected_stream.push_str(&format!("\n{DOWN_SYMBOL}{ELL_SYMBOL}Test8"));
        assert_eq!(stream.stream, expected_stream);
    }
}
