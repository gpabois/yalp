use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq)]
pub struct Range(std::ops::RangeInclusive<char>);

impl Range {
    pub fn not(&self) -> NotRange {
        NotRange(self.0.clone())
    }

    pub fn range_intersect(&self, rhs: &Self) -> Range {
        let start = *self.0.start().max(rhs.0.start());
        let end = *self.0.end().min(rhs.0.end());
        Range(start..=end)
    }

    pub fn intersect(&self, rhs: &Atomic) -> Atomic {
        match rhs {
            Atomic::NotRange(rhs) => {
                // Empty sequence
                if rhs.not().range_intersect(self) == *self {
                    return Atomic::Epsilon;
                }

                if (self.0.start() < rhs.0.start()) && (self.0.end() < rhs.0.start()) {
                    return self.clone().into();
                }

                if self.0.start() > rhs.0.end() {
                    return self.clone().into();
                }

                if (self.0.start() < rhs.0.start()) && (self.0.end() < rhs.0.end()) {
                    return Range(*self.0.start()..=*rhs.0.start()).into();
                }

                return [
                    Range(*self.0.start()..=*rhs.0.start()).into(),
                    Range(*rhs.0.end()..=*self.0.end()).into(),
                ]
                .into_iter()
                .collect();
            }
            Atomic::Range(rhs) => self.range_intersect(rhs).into(),
            Atomic::Set(rhs) => Atomic::Set(Set::from_iter(
                rhs.clone().into_iter().filter(|ch| self.contains(ch)),
            )),
            // Fragment the range into as much pieces which do not include the char in the set.
            Atomic::NotSet(set) => {
                let shared = set.clone().into_iter().filter(|ch| self.contains(ch));

                self.clone()
                    .fragments(shared)
                    .into_iter()
                    .map(Atomic::from)
                    .collect()
            }
            Atomic::List(atomics) => atomics.iter().map(|a| self.intersect(a)).collect(),
            Atomic::Any => rhs.clone(),
            Atomic::Epsilon => Atomic::Epsilon,
        }
    }

    pub fn split(self, ch: &char) -> [Range; 2] {
        let left = Self(*self.0.start()..=(((*ch as u8) - 1) as char));
        let right = Self((((*ch as u8) + 1) as char)..=*self.0.end());
        [left, right]
    }

    pub fn fragments(self, chs: impl Iterator<Item = char>) -> Vec<Self> {
        let mut parts: Vec<Self> = vec![self.clone()];

        for ch in chs {
            let mut step: Vec<Self> = vec![];

            while let Some(part) = parts.pop() {
                if part.contains(&ch) {
                    step.extend(part.split(&ch));
                } else {
                    step.push(part);
                }
            }

            parts = step;
        }

        parts.into_iter().filter(|r| !r.is_empty()).collect()
    }

    pub fn contains(&self, ch: &char) -> bool {
        self.0.contains(ch)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone)]
pub struct NotRange(std::ops::RangeInclusive<char>);

impl NotRange {
    pub fn is_empty(&self) -> bool {
        (*self.0.start() == '\0') && (*self.0.end() == char::MAX)
    }

    pub fn contains(&self, ch: &char) -> bool {
        !self.0.contains(ch)
    }

    pub fn not(&self) -> Range {
        Range(self.0.clone())
    }
}

#[derive(Clone)]
pub struct Set(HashSet<char>);

impl IntoIterator for Set {
    type Item = char;

    type IntoIter = <HashSet<char> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<char> for Set {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Set {
    pub fn not(&self) -> NotSet {
        NotSet(self.0.clone())
    }
}

#[derive(Clone)]
pub struct NotSet(HashSet<char>);

impl IntoIterator for NotSet {
    type Item = char;

    type IntoIter = <std::collections::HashSet<char> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl NotSet {
    pub fn not(&self) -> Set {
        Set(self.0.clone())
    }
}

#[derive(Clone)]
pub enum Atomic {
    NotRange(NotRange),
    Range(Range),
    Set(Set),
    NotSet(NotSet),
    List(Vec<Atomic>),
    Any,
    Epsilon,
}

impl FromIterator<Atomic> for Atomic {
    fn from_iter<T: IntoIterator<Item = Atomic>>(iter: T) -> Self {
        let mut atomics: Vec<_> = iter.into_iter().collect();
        if atomics.is_empty() {
            Atomic::Epsilon
        } else if atomics.len() == 1 {
            atomics.pop().unwrap()
        } else {
            Atomic::List(atomics)
        }
    }
}

impl From<Range> for Atomic {
    fn from(value: Range) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            Self::Range(value)
        }
    }
}

impl From<NotRange> for Atomic {
    fn from(value: NotRange) -> Self {
        if value.is_empty() {
            Self::Epsilon
        } else {
            Self::NotRange(value)
        }
    }
}

impl From<Set> for Atomic {
    fn from(value: Set) -> Self {
        Atomic::Set(value)
    }
}

impl From<NotSet> for Atomic {
    fn from(value: NotSet) -> Self {
        Atomic::NotSet(value)
    }
}

impl std::ops::Not for Atomic {
    type Output = Atomic;

    fn not(self) -> Self::Output {
        match self {
            Atomic::NotRange(range) => range.not().into(),
            Atomic::Range(range) => range.not().into(),
            Atomic::Set(set) => set.not().into(),
            Atomic::NotSet(set) => set.not().into(),
            Atomic::List(atomics) => Atomic::List(atomics.into_iter().map(Atomic::not).collect()),
            Atomic::Any => Atomic::Epsilon,
            Atomic::Epsilon => Atomic::Any,
        }
    }
}

impl Atomic {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Epsilon)
    }

    /// Intersection
    pub fn intersect(&self, rhs: &Atomic) -> Self {
        match self {
            Atomic::NotRange(_) => todo!(),
            Atomic::Range(_) => todo!(),
            Atomic::Set(_) => todo!(),
            Atomic::NotSet(_) => todo!(),
            Atomic::List(_) => todo!(),
            Atomic::Any => rhs.clone(),
            Atomic::Epsilon => Atomic::Epsilon,
        }
    }
}
