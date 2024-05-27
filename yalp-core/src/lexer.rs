use std::marker::PhantomData;

use crate::{token::Token, YalpResult};

use self::traits::Lexer as _;

pub mod traits {
    use crate::{token::traits::Token, YalpResult};

    use super::Span;

    /// The trait for a Lexer.
    pub trait Lexer<Error>: Iterator<Item = YalpResult<Self::Token, Error>> {
        type Token: Token;

        fn span(&self) -> Span;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ActionKind {
    Reconsume,
    Consume,
    ConsumeAndReduce(&'static str),
    Skip,
}

#[derive(Debug, Clone, Copy)]
pub struct Action {
    kind: ActionKind,
    goto: usize,
}

impl Action {
    pub fn reconsume(goto: usize) -> Self {
        Action {
            kind: ActionKind::Reconsume,
            goto,
        }
    }

    pub fn skip(goto: usize) -> Self {
        Self {
            kind: ActionKind::Skip,
            goto,
        }
    }

    pub fn consume_and_reduce(kind: &'static str, goto: usize) -> Self {
        Self {
            kind: ActionKind::ConsumeAndReduce(kind),
            goto,
        }
    }
}

pub type State<Error> = fn(char) -> YalpResult<Action, Error>;

pub struct Lexer<'kind, 'state, Stream, Error>
where
    Stream: Iterator<Item = char>,
{
    state: usize,
    states: &'state [State<Error>],
    span: Span,
    reconsume: Option<char>,
    buffer: String,
    stream: Stream,
    _phantom: PhantomData<(&'kind (), Error)>,
}

impl<'kind, 'state, Stream, Error> traits::Lexer<Error> for Lexer<'kind, 'state, Stream, Error>
where
    Stream: Iterator<Item = char>,
{
    type Token = Token<'kind>;

    fn span(&self) -> Span {
        self.span
    }
}

impl<'kind, 'state, Stream, Error> Lexer<'kind, 'state, Stream, Error>
where
    Stream: Iterator<Item = char>,
{
    pub fn new(states: &'state [State<Error>], stream: Stream) -> Self {
        Self {
            state: 0,
            states,
            stream,
            buffer: String::default(),
            reconsume: None,
            span: Span::default(),
            _phantom: PhantomData,
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
                self.span += NextLine;
            } else {
                self.span += NextColumn;
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

impl<'kind, 'state, Stream, Error> Iterator for Lexer<'kind, 'state, Stream, Error>
where
    Stream: Iterator<Item = char>,
{
    type Item = YalpResult<Token<'kind>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let state = self.states[self.state];

        while let Some(ch) = self.next_char() {
            let action_result = state(ch).map_err(|mut err| {
                err.span = Some(self.span());
                err
            });

            if action_result.is_err() {
                return Some(Err(action_result.unwrap_err()))
            }

            let action = action_result.unwrap_or_else(|_| unreachable!());

            match action.kind {
                ActionKind::Reconsume => self.reconsume(ch),
                ActionKind::Consume => self.consume(ch),
                ActionKind::ConsumeAndReduce(kind) => {
                    self.consume(ch);
                    let value = self.take();
                    return Some(Ok(Token {
                        kind,
                        value,
                        location: self.span,
                    }));
                }
                ActionKind::Skip => {}
            };

            self.state = action.goto;
        }

        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The location of the Token in the stream.
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { line: 1, column: 0 }
    }
}

pub struct NextLine;
pub struct NextColumn;

impl std::ops::Add<NextLine> for Span {
    type Output = Self;

    fn add(mut self, rhs: NextLine) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::Add<NextColumn> for Span {
    type Output = Self;

    fn add(mut self, rhs: NextColumn) -> Self::Output {
        self += rhs;
        self
    }
}

impl std::ops::AddAssign<NextLine> for Span {
    fn add_assign(&mut self, _: NextLine) {
        self.column = 0;
        self.line += 1;
    }
}

impl std::ops::AddAssign<NextColumn> for Span {
    fn add_assign(&mut self, _: NextColumn) {
        self.column += 1;
    }
}

#[cfg(test)]
pub mod fixtures {
    use crate::{YalpError, ErrorKind, NoCustomError, YalpResult};

    use super::{Action, Lexer, State};

    fn lr0_root_state(ch: char) -> YalpResult<Action, NoCustomError> {
        match ch {
            '0' => Ok(Action::consume_and_reduce("0", 0)),
            '1' => Ok(Action::consume_and_reduce("1", 0)),
            '+' => Ok(Action::consume_and_reduce("+", 0)),
            '*' => Ok(Action::consume_and_reduce("*", 0)),
            ' ' => Ok(Action::skip(0)),
            _ => Err(
                YalpError::new(
                    ErrorKind::unexpected_symbol(
                    &ch.to_string(), 
                    vec!["0", "1", "*", " "]
                    )
                , None
            )),
        }
    }

    static LR0_LEXER_STATES: &[State<NoCustomError>] = &[
        // 0 : root
        lr0_root_state,
    ];

    pub fn lexer_fixture_lr0<I>(iter: I) -> Lexer<'static, 'static, I, NoCustomError>
    where
        I: Iterator<Item = char>,
    {
        Lexer::new(LR0_LEXER_STATES, iter)
    }

    fn lr1_root_state(ch: char) -> YalpResult<Action, NoCustomError> {
        match ch {
            '+' => Ok(Action::consume_and_reduce("+", 0)),
            'n' => Ok(Action::consume_and_reduce("n", 0)),
            '(' => Ok(Action::consume_and_reduce("(", 0)),
            ')' => Ok(Action::consume_and_reduce(")", 0)),
            ' ' => Ok(Action::skip(0)),
            _ => Err(
                YalpError::new(
                    ErrorKind::unexpected_symbol(
                    &ch.to_string(), 
                    vec!["0", "1", "*", " "]
                    )
                , None
            )),
        }
    }

    static LR1_LEXER_STATES: &[State<NoCustomError>] = &[
        // 0 : root
        lr1_root_state,
    ];

    pub fn lexer_fixture_lr1<I>(iter: I) -> Lexer<'static, 'static, I, NoCustomError>
    where
        I: Iterator<Item = char>,
    {
        Lexer::new(LR1_LEXER_STATES, iter)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Span, token::Token};

    use super::fixtures::lexer_fixture_lr0;

    #[test]
    fn test_lexer() {
        let lexer = lexer_fixture_lr0("1 + 1 * 0".chars());
        let tokens = lexer.collect::<Result<Vec<_>, _>>().unwrap();
        let expected_tokens = vec![
            Token::new("1", "1", Span::new(1, 1)),
            Token::new("+", "+", Span::new(1, 3)),
            Token::new("1", "1", Span::new(1, 5)),
            Token::new("*", "*", Span::new(1, 7)),
            Token::new("0", "0", Span::new(1, 9)),
        ];

        assert_eq!(tokens, expected_tokens);
    }
}
