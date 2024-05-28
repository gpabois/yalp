use std::marker::PhantomData;

use crate::{token::Token, YalpResult};

use self::traits::Lexer as _;

pub mod atomic;
//pub mod graph;

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
pub enum Action<'kind> {
    /// Reconsume the current character
    Reconsume,
    Consume,
    Write,
    Push(&'kind str),
    Merge(&'kind str, usize),
}
#[derive(Debug, Default)]
pub struct ActionSequence<'kind> {
    actions: Vec<Action<'kind>>,
    goto: usize,
}

impl<'kind> ActionSequence<'kind> {
    pub fn new(goto: usize) -> Self {
        Self {
            actions: vec![],
            goto,
        }
    }

    pub fn act(mut self, action: Action<'kind>) -> Self {
        self.actions.push(action);
        self
    }

    pub fn reconsume(self) -> Self {
        self.act(Action::Reconsume)
    }

    pub fn consume(self) -> Self {
        self.act(Action::Consume)
    }

    pub fn write(self) -> Self {
        self.act(Action::Write)
    }

    pub fn push(self, kind: &'kind str) -> Self {
        self.act(Action::Push(kind))
    }

    pub fn merge(self, kind: &'kind str, n: usize) -> Self {
        self.act(Action::Merge(kind, n))
    }
}

impl<'kind> IntoIterator for ActionSequence<'kind> {
    type Item = Action<'kind>;

    type IntoIter = <Vec<Action<'kind>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.actions.into_iter()
    }
}

pub type State<'kind, Error> = fn(char) -> YalpResult<ActionSequence<'kind>, Error>;

pub struct Lexer<'kind, 'state, Stream, Error>
where
    Stream: Iterator<Item = char>,
{
    state: usize,
    states: &'state [State<'kind, Error>],
    span: Span,
    reconsume: Option<char>,
    /// The current token's buffer
    buffer: String,
    /// Fragmented tokens are intermediate results for complex tokenization
    fragments: Vec<Token<'kind>>,
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
    pub fn new(states: &'state [State<'kind, Error>], stream: Stream) -> Self {
        Self {
            state: 0,
            states,
            stream,
            buffer: String::default(),
            reconsume: None,
            span: Span::default(),
            fragments: vec![],
            _phantom: PhantomData,
        }
    }

    /// Push the current buffer as a fragment
    fn push(&mut self, kind: &'kind str) {
        let token = Token::new(kind, self.take(), self.span(), vec![]);
    }

    /// Merge the n last fragments on the stack
    fn merge(&mut self, kind: &'kind str, n: usize) {
        let consume = self.fragments.len().saturating_sub(n);
        let token = Token::new(
            kind,
            self.take(),
            self.span(),
            self.fragments.drain(consume..).collect(),
        );
    }

    /// Write the TOS fragment in the output stream.
    fn write(&mut self) -> Token<'kind> {
        self.fragments.pop().unwrap()
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

    pub fn reconsume(&mut self, ch: char) {
        self.reconsume = Some(ch);
    }

    pub fn consume(&mut self, ch: char) {
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
                return Some(Err(action_result.unwrap_err()));
            }

            let seq = action_result.unwrap_or_else(|_| unreachable!());
            self.state = seq.goto;

            for action in seq {
                match action {
                    Action::Reconsume => self.reconsume(ch),
                    Action::Consume => self.consume(ch),
                    Action::Write => return self.fragments.pop().map(|f| Ok(f)),
                    Action::Push(kind) => self.push(kind),
                    Action::Merge(kind, n) => self.merge(kind, n),
                }
            }
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
    use crate::{ActionSequence, ErrorKind, NoCustomError, YalpError, YalpResult};

    use super::{Action, Lexer, State};

    fn lr0_root_state(ch: char) -> YalpResult<ActionSequence<'static>, NoCustomError> {
        match ch {
            '0' => Ok(ActionSequence::new(0).consume().push("0").write()),
            '1' => Ok(ActionSequence::new(0).consume().push("1").write()),
            '+' => Ok(ActionSequence::new(0).consume().push("+").write()),
            '*' => Ok(ActionSequence::new(0).consume().push("*").write()),
            ' ' => Ok(ActionSequence::new(0)),
            _ => Err(YalpError::new(
                ErrorKind::unexpected_symbol(&ch.to_string(), vec!["0", "1", "*", " "]),
                None,
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

    fn lr1_root_state(ch: char) -> YalpResult<ActionSequence<'static>, NoCustomError> {
        match ch {
            '+' => Ok(ActionSequence::new(0).consume().push("+").write()),
            'n' => Ok(ActionSequence::new(0).consume().push("n").write()),
            '(' => Ok(ActionSequence::new(0).consume().push("(").write()),
            ')' => Ok(ActionSequence::new(0).consume().push(")").write()),
            '0' => Ok(ActionSequence::new(0).consume()),
            ' ' => Ok(ActionSequence::new(0)),
            _ => Err(YalpError::new(
                ErrorKind::unexpected_symbol(&ch.to_string(), vec!["0", "1", "*", " "]),
                None,
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
            Token::new("1", "1", Span::new(1, 1), vec![]),
            Token::new("+", "+", Span::new(1, 3), vec![]),
            Token::new("1", "1", Span::new(1, 5), vec![]),
            Token::new("*", "*", Span::new(1, 7), vec![]),
            Token::new("0", "0", Span::new(1, 9), vec![]),
        ];

        assert_eq!(tokens, expected_tokens);
    }
}
