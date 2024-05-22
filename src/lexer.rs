use std::marker::PhantomData;

use crate::token::Token;

#[derive(Debug)]
pub enum LexerErrorKind {
    UnexpectedEndOfStream,
    UnexpectedChar(char)
}

impl std::fmt::Display for LexerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerErrorKind::UnexpectedEndOfStream => write!(f, "unexpected end of stream"),
            LexerErrorKind::UnexpectedChar(c) => write!(f, "unexpected char '{}'", c),
        }
    }
}

#[derive(Debug)]
pub struct LexerError {
    location: SourceLocation,
    kind: LexerErrorKind
}

impl LexerError {
    pub fn unexpected_end_of_stream(location: SourceLocation) -> Self {
        Self {
            location,
            kind: LexerErrorKind::UnexpectedEndOfStream
        }
    }


}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line={}, col={}", self.kind, self.location.line, self.location.column)
    }
}

pub type LexerResult<T> = Result<T, LexerError>;

pub mod traits {
    use crate::token::traits::Token;

    use super::{LexerResult, SourceLocation};

    /// The trait for a Lexer.
    pub trait Lexer: Iterator<Item=LexerResult<Self::Token>> {
        type Token: Token;

        fn current_location(&self) -> SourceLocation;
    }
}

pub enum ActionKind {
    Reconsume,
    Consume,
    ConsumeAndReduce(&'static str),
    Skip
}
pub struct Action {
    kind: ActionKind,
    goto: usize
}

impl Action {
    pub fn reconsume(goto: usize) -> Self {
        Action {
            kind: ActionKind::Reconsume,
            goto
        }
    }

    pub fn skip(goto: usize) -> Self {
        Self {
            kind: ActionKind::Skip,
            goto
        }
    }

    pub fn consume_and_reduce(kind: &'static str, goto: usize) -> Self {
        Self {
            kind: ActionKind::ConsumeAndReduce(kind),
            goto
        }
    }
}

pub type State = fn(char) -> Result<Action, LexerErrorKind>;

pub struct Lexer<'kind, 'state, Stream> where Stream: Iterator<Item=char> {
    state: usize,
    states: &'state [State],
    current_location: SourceLocation,
    reconsume: Option<char>,
    buffer: String,
    stream: Stream,
    _phantom: PhantomData<&'kind ()>
}

impl<'kind, 'state, Stream> traits::Lexer for Lexer<'kind, 'state, Stream> where Stream: Iterator<Item=char> {
    type Token = Token<'kind>;

    fn current_location(&self) -> SourceLocation {
        self.current_location
    }
}

impl<'kind, 'state, Stream> Lexer<'kind, 'state, Stream> where Stream: Iterator<Item=char> {
    pub fn new(states: &'state [State], stream: Stream) -> Self {
        Self {
            state: 0,
            states,
            stream,
            buffer: String::default(),
            reconsume: None,
            current_location: SourceLocation::default(),
            _phantom: PhantomData::default()
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.reconsume.is_some() {
            let char = self.reconsume.unwrap();
            self.reconsume = None;
            return Some(char);
        }

        self.stream.next().inspect(|&ch| {
            if ch == '\n' {
                self.current_location += NextLine;
            } else {
                self.current_location += NextColumn;
            }
        })
    }

    fn reconsume(&mut self, ch: char) {
        self.reconsume = Some(ch);
    }

    fn consume(&mut self, ch: char) {
        self.buffer.push(ch)
    }

    fn take(&mut self) -> String {
        std::mem::take(&mut self.buffer)
    }

}

impl<'kind, 'state, Stream> Iterator for Lexer<'kind, 'state, Stream> where Stream: Iterator<Item=char> {
    type Item = LexerResult<Token<'kind>>;

    fn next(&mut self) -> Option<Self::Item> {
        let state = self.states[self.state];

        while let Some(ch) = self.next_char() {
            let action_result = state(ch).map_err(|kind| LexerError {kind, location: self.current_location});
            
            if let Err(err) = action_result {
                return Some(Err(err));
            } 

            let action = action_result.unwrap();

            match action.kind {
                ActionKind::Reconsume => self.reconsume(ch),
                ActionKind::Consume => self.consume(ch),
                ActionKind::ConsumeAndReduce(kind) => {
                    self.consume(ch);
                    let value = self.take();
                    return Some(Ok(Token {kind, value, location: self.current_location}));
                },
                ActionKind::Skip => {},
            };

            self.state = action.goto;
        }

        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The location of the Token in the stream.
pub struct SourceLocation {
    pub line: usize,
    pub column: usize
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        Self{line, column}
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self { line: 1, column: 0 }
    }
}

pub struct NextLine;
pub struct NextColumn;

impl std::ops::Add<NextLine> for SourceLocation {
    type Output = Self;

    fn add(mut self, rhs: NextLine) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::Add<NextColumn> for SourceLocation {
    type Output = Self;

    fn add(mut self, rhs: NextColumn) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign<NextLine> for SourceLocation {
    fn add_assign(&mut self, _: NextLine) {
        self.column = 0;
        self.line += 1;
    }
}

impl std::ops::AddAssign<NextColumn> for SourceLocation {
    fn add_assign(&mut self, _: NextColumn) {
        self.column += 1;
    }
}

#[cfg(test)] 
pub mod fixtures {
    use super::{Action, Lexer, LexerErrorKind, State};

    fn lr0_root_state(c: char) -> Result<Action, LexerErrorKind> {
        match c {
            '0' => Ok(Action::consume_and_reduce("0", 0)),
            '1' => Ok(Action::consume_and_reduce("1", 0)),
            '+' => Ok(Action::consume_and_reduce("+", 0)),
            '*' => Ok(Action::consume_and_reduce("*", 0)),
            ' ' => Ok(Action::skip(0)),
            _ => Err(LexerErrorKind::UnexpectedChar(c))
        }
    }

    static LR0_LEXER_STATES: &[State] = &[
        // 0 : root
        lr0_root_state,
    ];

    pub fn lexer_fixture_lr0<I>(iter: I) -> Lexer<'static, 'static, I> where I: Iterator<Item=char> {
        Lexer::new(LR0_LEXER_STATES, iter)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::SourceLocation, token::Token};

    use super::fixtures::lexer_fixture_lr0;

    #[test]
    fn test_lexer() {
        let lexer = lexer_fixture_lr0("1 + 1 * 0".chars());
        let tokens  = lexer.collect::<Result<Vec<_>, _>>().unwrap();
        let expected_tokens = vec![
            Token::new("1", "1", SourceLocation::new(1, 1)),
            Token::new("+", "+", SourceLocation::new(1, 3)),
            Token::new("1", "1", SourceLocation::new(1, 5)),
            Token::new("*", "*", SourceLocation::new(1, 7)),
            Token::new("0", "0", SourceLocation::new(1, 9))
        ];
        
        assert_eq!(tokens, expected_tokens);
    }
}