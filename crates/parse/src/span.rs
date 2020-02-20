use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

/// The span of a region of source code.
///
/// Internally, this struct just contains two [`LineCol`]s,
/// with one being the start and one being the end of the span.
///
/// [`LineCol`]: ./struct.LineCol.html
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Span {
    start: LineCol,
    end: LineCol,
}

impl Span {
    /// Constructs a new span from two [`LineCol`]s.
    ///
    /// # Panics
    ///
    /// Panics if `start` is greater than `end`.
    ///
    /// [`LineCol`]: ./struct.LineCol.html
    ///
    /// # Examples
    ///
    /// ```
    /// use mcfunction_parse::{ Span, LineCol };
    ///
    /// let span = Span::new(LineCol::new(0, 2), LineCol::new(0, 5));
    /// println!("{:?}", span);
    /// ```
    pub fn new(start: LineCol, end: LineCol) -> Self {
        assert!(start <= end, "start > end");
        Span { start, end }
    }

    /// Gets the start of the span.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcfunction_parse::{ Span, LineCol };
    ///
    /// let span = Span::new(LineCol::new(1, 3), LineCol::new(7, 2));
    /// assert_eq!(span.start(), LineCol::new(1, 3));
    /// ```
    pub fn start(&self) -> LineCol {
        self.start
    }

    /// Gets the end of the span.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcfunction_parse::{ Span, LineCol };
    ///
    /// let span = Span::new(LineCol::new(1, 3), LineCol::new(7, 2));
    /// assert_eq!(span.end(), LineCol::new(7, 2));
    /// ```
    pub fn end(&self) -> LineCol {
        self.end
    }

    /// Joins two spans into one span that contains both.
    ///
    /// The resulting span will have the smaller start and larger end.
    /// The two spans may be disjoint, intersecting,
    /// or one may be fully inside the other.
    ///
    /// # Examples
    ///
    /// ```
    /// use mcfunction_parse::{ Span, LineCol };
    ///
    /// let span1 = Span::new(LineCol::new(0, 0), LineCol::new(0, 2));
    /// let span2 = Span::new(LineCol::new(0, 5), LineCol::new(1, 3));
    /// assert_eq!(span1.union(&span2), Span::new(LineCol::new(0, 0), LineCol::new(1, 3)));
    /// ```
    ///
    /// ```
    /// use mcfunction_parse::{ Span, LineCol };
    ///
    /// let span1 = Span::new(LineCol::new(0, 0), LineCol::new(1, 7));
    /// let span2 = Span::new(LineCol::new(0, 2), LineCol::new(1, 0));
    /// assert_eq!(span1.union(&span2), Span::new(LineCol::new(0, 0), LineCol::new(1, 7)));
    /// ```
    pub fn union(&self, other: &Span) -> Self {
        Span {
            start: self.start().min(other.start()),
            end: self.end().max(other.end()),
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.start, self.end)
    }
}

/// A specific line and column in a source file.
/// Both the line and the column are zero indexed.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct LineCol {
    line: usize,
    col: usize,
}

impl LineCol {
    /// Construct a new line-column from a line and a column
    ///
    /// # Examples
    /// ```
    /// use mcfunction_parse::LineCol;
    ///
    /// let linecol = LineCol::new(123, 456);
    /// println!("{:?}", linecol);
    /// ```
    pub fn new(line: usize, col: usize) -> Self {
        LineCol { line, col }
    }

    /// Gets the line of the position
    ///
    /// # Examples
    /// ```
    /// use mcfunction_parse::LineCol;
    ///
    /// let linecol = LineCol::new(0, 7);
    /// assert_eq!(linecol.line(), 0);
    /// ```
    pub fn line(&self) -> usize {
        self.line
    }

    /// Gets the column of the position
    ///
    /// # Examples
    /// ```
    /// use mcfunction_parse::LineCol;
    ///
    /// let linecol = LineCol::new(0, 7);
    /// assert_eq!(linecol.col(), 7);
    /// ```
    pub fn col(&self) -> usize {
        self.col
    }
}

impl Ord for LineCol {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.line().cmp(&other.line()) {
            Ordering::Equal => self.col().cmp(&other.col()),
            v => v,
        }
    }
}

impl PartialOrd for LineCol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for LineCol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}
