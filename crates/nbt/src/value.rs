use std::collections::HashMap;

use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, prelude::*},
};

macro_rules! read_number {
    ($type:ty, $r:ident, $buf:ident) => {{
        $r.read_exact(&mut $buf[..std::mem::size_of::<$type>()])?;
        <$type>::from_be_bytes($buf[..std::mem::size_of::<$type>()].try_into().unwrap())
    }};
}

macro_rules! write_number {
    ($v:expr, $w:expr) => {
        $w.write_all(&$v.to_be_bytes())?;
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TryFrom<u8> for TagType {
    type Error = NbtParseError;
    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0..=12 => Ok(unsafe { std::mem::transmute(i) }),
            _ => Err(NbtParseError::InvalidTagType(i)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(TagType, Vec<NbtValue>),
    Compound(HashMap<String, NbtValue>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtValue {
    pub fn read_named<R: Read>(r: &mut R) -> Result<(String, Self), NbtParseError> {
        let mut buf = [0u8];
        let tag: TagType = read_number!(u8, r, buf).try_into()?;
        if tag == TagType::End {
            return Err(NbtParseError::UnexpectedEnd);
        }
        Ok((read_string(r)?, Self::read_payload(r, tag)?))
    }

    pub fn read_payload<R: Read>(r: &mut R, tag: TagType) -> Result<Self, NbtParseError> {
        let mut buf = [0u8; 8];
        match tag {
            TagType::End => Err(NbtParseError::UnexpectedEnd),
            TagType::Byte => Ok(NbtValue::Byte(read_number!(i8, r, buf))),
            TagType::Short => Ok(NbtValue::Short(read_number!(i16, r, buf))),
            TagType::Int => Ok(NbtValue::Int(read_number!(i32, r, buf))),
            TagType::Long => Ok(NbtValue::Long(read_number!(i64, r, buf))),
            TagType::Float => Ok(NbtValue::Float(read_number!(f32, r, buf))),
            TagType::Double => Ok(NbtValue::Double(read_number!(f64, r, buf))),
            TagType::ByteArray => {
                let len = read_number!(i32, r, buf);
                let mut out = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    out.push(read_number!(i8, r, buf));
                }
                Ok(NbtValue::ByteArray(out))
            }
            TagType::String => read_string(r).map(NbtValue::String),
            TagType::List => {
                let tag: TagType = read_number!(u8, r, buf).try_into()?;
                let len = read_number!(i32, r, buf);
                if len > 0 && tag == TagType::End {
                    return Err(NbtParseError::UnexpectedEnd);
                }
                let mut out = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    out.push(Self::read_payload(r, tag)?);
                }
                Ok(NbtValue::List(tag, out))
            }
            TagType::Compound => {
                let mut out = HashMap::new();
                loop {
                    let (name, tag) = match Self::read_named(r) {
                        Ok(v) => v,
                        Err(NbtParseError::UnexpectedEnd) => break,
                        Err(err) => return Err(err),
                    };
                    out.insert(name, tag);
                }
                Ok(NbtValue::Compound(out))
            }
            TagType::IntArray => {
                let len = read_number!(i32, r, buf);
                let mut out = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    out.push(read_number!(i32, r, buf));
                }
                Ok(NbtValue::IntArray(out))
            }
            TagType::LongArray => {
                let len = read_number!(i32, r, buf);
                let mut out = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    out.push(read_number!(i64, r, buf));
                }
                Ok(NbtValue::LongArray(out))
            }
        }
    }

    pub fn write_named<W: Write>(&self, w: &mut W, name: &str) -> Result<(), NbtParseError> {
        write_number!(self.tag_type() as u8, w);
        write_string(w, &name)?;
        self.write_payload(w)?;
        Ok(())
    }

    pub fn write_payload<W: Write>(&self, w: &mut W) -> Result<(), NbtParseError> {
        match self {
            Self::Byte(v) => write_number!(v, w),
            Self::Short(v) => write_number!(v, w),
            Self::Int(v) => write_number!(v, w),
            Self::Long(v) => write_number!(v, w),
            Self::Float(v) => write_number!(v, w),
            Self::Double(v) => write_number!(v, w),
            Self::ByteArray(arr) => {
                write_number!(arr.len() as i32, w);
                for byte in arr {
                    write_number!(byte, w);
                }
            }
            Self::String(string) => {
                write_string(w, &string)?;
            }
            Self::List(tag, list) => {
                write_number!(*tag as u8, w);
                write_number!(list.len() as i32, w);
                for x in list {
                    x.write_payload(w)?;
                }
            }
            Self::Compound(cpd) => {
                for (k, v) in cpd {
                    v.write_named(w, &k)?;
                }
                // End tag
                write_number!(0u8, w);
            }
            Self::IntArray(arr) => {
                write_number!(arr.len() as i32, w);
                for int in arr {
                    write_number!(int, w);
                }
            }
            Self::LongArray(arr) => {
                write_number!(arr.len() as i32, w);
                for long in arr {
                    write_number!(long, w);
                }
            }
        }
        Ok(())
    }

    pub fn tag_type(&self) -> TagType {
        match self {
            Self::Byte(_) => TagType::Byte,
            Self::Short(_) => TagType::Short,
            Self::Int(_) => TagType::Int,
            Self::Long(_) => TagType::Long,
            Self::Float(_) => TagType::Float,
            Self::Double(_) => TagType::Double,
            Self::ByteArray(_) => TagType::ByteArray,
            Self::String(_) => TagType::String,
            Self::List(..) => TagType::List,
            Self::Compound(_) => TagType::Compound,
            Self::IntArray(_) => TagType::IntArray,
            Self::LongArray(_) => TagType::LongArray,
        }
    }

    fn ind_fmt(&self, f: &mut Formatter, ind: usize) -> fmt::Result {
        let indstr = "    ".repeat(ind);
        if f.alternate() {
            write!(f, "{}", indstr)?;
        }
        match self {
            Self::Byte(v) => write!(f, "{}b", v),
            Self::Short(v) => write!(f, "{}s", v),
            Self::Int(v) => write!(f, "{}", v),
            Self::Long(v) => write!(f, "{}l", v),
            Self::Float(v) => write!(f, "{}f", v),
            Self::Double(v) => write!(f, "{}d", v),
            Self::String(v) => fmt_string(f, v),
            Self::ByteArray(v) => fmt_array(f, "B;", "b", &v, f.alternate(), ind),
            Self::IntArray(v) => fmt_array(f, "I;", "", &v, f.alternate(), ind),
            Self::LongArray(v) => fmt_array(f, "L;", "l", &v, f.alternate(), ind),
            Self::List(_, v) => {
                if v.is_empty() {
                    write!(f, "[]")?;
                } else if f.alternate() {
                    write!(f, "[\n    {}", indstr)?;
                    v[0].ind_fmt(f, ind + 1)?;
                    for x in v.iter().skip(1) {
                        write!(f, ",\n    {}", indstr)?;
                        x.ind_fmt(f, ind + 1)?;
                    }
                    write!(f, "\n{}]", indstr)?;
                } else {
                    write!(f, "[{}", v[0])?;
                    for x in v.iter().skip(1) {
                        write!(f, ",{}", x)?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            Self::Compound(v) => {
                if v.is_empty() {
                    write!(f, "{{}}")?;
                } else {
                    let vec = v.iter().collect::<Vec<(&String, &NbtValue)>>();
                    if f.alternate() {
                        write!(f, "{{\n    {}", indstr)?;
                        fmt_string(f, vec[0].0)?;
                        write!(f, ": ")?;
                        vec[0].1.ind_fmt(f, ind + 1)?;
                        for (k, v) in vec.into_iter().skip(1) {
                            write!(f, "\n,    ")?;
                            fmt_string(f, k)?;
                            write!(f, ": ")?;
                            v.ind_fmt(f, ind + 1)?;
                        }
                        write!(f, "\n{}}}", indstr)?;
                    } else {
                        write!(f, "{{")?;
                        fmt_string(f, vec[0].0)?;
                        write!(f, ": {}", vec[0].1)?;
                        for (k, v) in vec.into_iter().skip(1) {
                            write!(f, ",")?;
                            fmt_string(f, k)?;
                            write!(f, ":{}", v)?;
                        }
                        write!(f, "}}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

fn fmt_array<T: Display>(
    f: &mut Formatter,
    prefix: &str,
    suffix: &str,
    vals: &[T],
    pretty: bool,
    ind: usize,
) -> fmt::Result {
    let indstr = "    ".repeat(ind);
    if vals.is_empty() {
        write!(f, "[{}]", prefix)?;
    } else {
        write!(f, "[{}", prefix)?;
        if pretty {
            write!(f, "\n    {}{}{}", indstr, vals[0], suffix)?;
            for x in vals.iter().skip(1) {
                write!(f, ",\n    {}{}{}", indstr, x, suffix)?;
            }
            write!(f, "\n{}]", indstr)?;
        } else {
            write!(f, "{}{}", vals[0], suffix)?;
            for x in vals.iter().skip(1) {
                write!(f, ",{}{}", x, suffix)?;
            }
            write!(f, "]")?;
        }
    }
    Ok(())
}

fn fmt_string(f: &mut Formatter, string: &str) -> fmt::Result {
    for c in string.chars() {
        match c {
            '"' | '\\' => write!(f, "\\{}", c)?,
            // Not valid NBT, but needed to be roundtripable
            '\n' => write!(f, "\\n")?,
            v => write!(f, "{}", v)?,
        };
    }
    Ok(())
}

impl Display for NbtValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.ind_fmt(f, 0)
    }
}

#[derive(Debug)]
pub enum NbtParseError {
    InvalidTagType(u8),
    Io(io::Error),
    Utf16(std::string::FromUtf16Error),
    UnexpectedEnd,
}

impl Display for NbtParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidTagType(v) => write!(f, "Invalid tag type '{}'", v),
            Self::Io(v) => write!(f, "{}", v),
            Self::Utf16(v) => write!(f, "{}", v),
            Self::UnexpectedEnd => write!(f, "Unexpected end tag type"),
        }
    }
}

impl Error for NbtParseError {}

impl From<io::Error> for NbtParseError {
    fn from(err: io::Error) -> Self {
        NbtParseError::Io(err)
    }
}

impl From<std::string::FromUtf16Error> for NbtParseError {
    fn from(err: std::string::FromUtf16Error) -> Self {
        NbtParseError::Utf16(err)
    }
}

fn read_string<R: Read>(r: &mut R) -> Result<String, NbtParseError> {
    let mut buf = [0u8; 2];
    let len = read_number!(u16, r, buf);
    // A pretty good estimation because most of the time the modified UTF-8 won't be in effect
    let mut utf16 = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let byte = read_number!(u8, r, buf);
        if byte >> 7 == 0 {
            utf16.push(byte as u16);
        } else if byte >> 5 == 0b110 {
            let byte2 = read_number!(u8, r, buf);
            utf16.push(((byte as u16) & 0b11111 << 6) | ((byte2 as u16) & 0b111111));
        } else if byte >> 4 == 0b1110 {
            let byte2 = read_number!(u8, r, buf);
            let byte3 = read_number!(u8, r, buf);
            utf16.push(
                ((byte as u16) & 0b1111 << 12)
                    | ((byte2 as u16) & 0b111111 << 6)
                    | ((byte3 as u16) & 0b111111),
            );
        }
    }
    Ok(String::from_utf16(&utf16)?)
}

fn write_string<W: Write>(w: &mut W, string: &str) -> Result<(), NbtParseError> {
    let mut out: Vec<u8> = Vec::with_capacity(string.len());
    for c in string.encode_utf16() {
        match c {
            0x0001..=0x007F => {
                out.push(c as u8);
            }
            0x0000 | 0x0080..=0x07FF => {
                out.push((c >> 6) as u8 & 0b11111);
                out.push(c as u8 & 0b111111);
            }
            0x0800..=0xFFFF => {
                out.push((c >> 12) as u8 & 0b11111);
                out.push((c >> 6) as u8 & 0b111111);
                out.push(c as u8 & 0b111111);
            }
        }
    }
    write_number!(out.len() as u16, w);
    for x in out {
        write_number!(x, w);
    }
    Ok(())
}

#[cfg(feature = "snbt")]
mod snbt {
    use super::*;
    use mcfunction_mcf::syntax::cst::{self, NbtSequenceType, Node, NH};
    use mcfunction_parse::ast::CstNode;
    use std::borrow::Cow;
    use std::num::{ParseFloatError, ParseIntError};

    pub enum SnbtParseError {
        // placeholder
        None,
        ParseIntError(ParseIntError),
        ParseFloatError(ParseFloatError),
        InvalidListType(Vec<NbtValue>),
        MissingCompoundKey(HashMap<String, NbtValue>),
        MissingCompoundVal(HashMap<String, NbtValue>, String),
        InvalidBoolean(cst::NbtBoolean<Node>),
        InvalidNumber(cst::NbtNumber<Node>),
    }

    impl NbtValue {
        pub fn from_cst<N: NH>(node: cst::NbtValue<N>) -> Result<Self, SnbtParseError> {
            if let Some(node) = node.as_ref().compound() {
                let mut map = HashMap::new();
                for entry in node.entries() {
                    let name = match entry.key() {
                        Some(v) => v,
                        None => return Err(SnbtParseError::MissingCompoundKey(map)),
                    };
                    let name = string_value(name.view().string());
                    let value = match entry.value() {
                        Some(v) => v,
                        None => {
                            return Err(SnbtParseError::MissingCompoundVal(map, name.into_owned()))
                        }
                    };
                    map.insert(name.into_owned(), NbtValue::from_cst(value)?);
                }
                Ok(Self::Compound(map))
            } else if let Some(node) = node.as_ref().sequence() {
                let mut vec: Vec<NbtValue> = Vec::new();
                for value in node.entries() {
                    vec.push(NbtValue::from_cst(value)?);
                }
                match node.seq_type() {
                    NbtSequenceType::ByteArray => Ok(Self::ByteArray(
                        vec.into_iter()
                            .map(|v| match v {
                                Self::Byte(b) => Ok(b),
                                _ => Err(SnbtParseError::None),
                            })
                            .collect::<Result<_, _>>()?,
                    )),
                    NbtSequenceType::IntArray => Ok(Self::IntArray(
                        vec.into_iter()
                            .map(|v| match v {
                                Self::Int(b) => Ok(b),
                                _ => Err(SnbtParseError::None),
                            })
                            .collect::<Result<_, _>>()?,
                    )),
                    NbtSequenceType::LongArray => Ok(Self::LongArray(
                        vec.into_iter()
                            .map(|v| match v {
                                Self::Long(b) => Ok(b),
                                _ => Err(SnbtParseError::None),
                            })
                            .collect::<Result<_, _>>()?,
                    )),
                    NbtSequenceType::List => Ok(Self::List(
                        {
                            if vec.is_empty() {
                                TagType::End
                            } else {
                                let ty = vec[0].tag_type();
                                for x in &vec[1..] {
                                    if x.tag_type() != ty {
                                        return Err(SnbtParseError::InvalidListType(vec));
                                    }
                                }
                                ty
                            }
                        },
                        vec,
                    )),
                    NbtSequenceType::ErrorArray => Err(SnbtParseError::InvalidListType(vec)),
                }
            } else if let Some(node) = node.as_ref().boolean() {
                let val = match node.view().string() {
                    "true" => 1,
                    "false" => 0,
                    // this might never get returned but its here for safety
                    _ => return Err(SnbtParseError::InvalidBoolean(node.into_arc())),
                };
                Ok(Self::Byte(val))
            } else if let Some(node) = node.as_ref().number() {
                if let Some(byte) = node.byte() {
                    Ok(Self::Byte(byte.string().parse::<i8>()?))
                } else if let Some(short) = node.short() {
                    Ok(Self::Short(short.string().parse::<i16>()?))
                } else if let Some(long) = node.long() {
                    Ok(Self::Long(long.string().parse::<i64>()?))
                } else if let Some(float) = node.float() {
                    Ok(Self::Float(float.string().parse::<f32>()?))
                } else if let Some(double) = node.double() {
                    Ok(Self::Double(double.string().parse::<f64>()?))
                } else if let Some(untagged) = node.untagged() {
                    let string = untagged.string();
                    match string.parse::<i32>() {
                        Ok(v) => Ok(Self::Int(v)),
                        _ => Ok(Self::Double(string.parse::<f64>()?)),
                    }
                } else {
                    Err(SnbtParseError::InvalidNumber(node.into_arc()))
                }
            } else if let Some(node) = node.as_ref().string() {
                Ok(Self::String(
                    string_value(node.view().string()).into_owned(),
                ))
            } else {
                Err(SnbtParseError::None)
            }
        }
    }

    // Gets a best effor representation of the given string. No error checking is done
    fn string_value(string: &str) -> Cow<str> {
        if string.starts_with('"') || string.starts_with('\'') {
            let term = string.chars().next().unwrap();
            let mut buf = String::new();
            let mut escaped = false;
            for c in string[1..].chars() {
                if escaped {
                    if c == term {
                        buf.push(c);
                    } else if c == '\\' {
                        buf.push('\\');
                    } else {
                        buf.push(c);
                    }
                } else if c == term {
                    break;
                } else if c == '\\' {
                    escaped = true;
                } else {
                    buf.push(c);
                }
            }
            Cow::Owned(buf)
        } else {
            Cow::Borrowed(string)
        }
    }

    impl From<ParseIntError> for SnbtParseError {
        fn from(v: ParseIntError) -> Self {
            SnbtParseError::ParseIntError(v)
        }
    }

    impl From<ParseFloatError> for SnbtParseError {
        fn from(v: ParseFloatError) -> Self {
            SnbtParseError::ParseFloatError(v)
        }
    }
}
