//! Small, tolerant display lexers, not validators or script interpreters.
//! Cache complete lexical state at line boundaries; relex only changed lines
//! and the following lines until multiline state converges again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Context {
    #[default]
    Plain,
    PowerShell,
    Json,
    Yaml,
    Xml,
    Ini,
}
impl Context {
    pub const ALL: [Self; 6] = [
        Self::Plain,
        Self::PowerShell,
        Self::Json,
        Self::Yaml,
        Self::Xml,
        Self::Ini,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Plain => "Plain text",
            Self::PowerShell => "PowerShell",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Xml => "XML / HTML",
            Self::Ini => "INI / config",
        }
    }
    pub fn from_id(id: u8) -> Option<Self> {
        Self::ALL.get(id as usize).copied()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Comment,
    String,
    Key,
    Number,
    Literal,
    Keyword,
    Command,
    Parameter,
    Variable,
    Type,
    Member,
    Punctuation,
}
impl Kind {
    pub const ALL: [Self; 12] = [
        Self::Comment,
        Self::String,
        Self::Key,
        Self::Number,
        Self::Literal,
        Self::Keyword,
        Self::Command,
        Self::Parameter,
        Self::Variable,
        Self::Type,
        Self::Member,
        Self::Punctuation,
    ];
    /// Semantic syntax palette. All colors exceed WCAG 4.5:1 on editor backgrounds.
    pub fn color(self, dark: bool) -> u32 {
        let (dark_color, light_color) = match self {
            Self::Comment => (0x91AC91, 0x426642),
            Self::String => (0xE6BA94, 0x873F1D),
            Self::Key => (0x88CBEC, 0x005A83),
            Self::Number => (0xADDBA5, 0x28652A),
            Self::Literal => (0xA9BFFF, 0x354FA0),
            Self::Keyword => (0xD6A8E9, 0x793B91),
            Self::Command => (0xEFDA9C, 0x745500),
            Self::Parameter => (0x98D9DD, 0x00646A),
            Self::Variable => (0xA0D4F5, 0x005B8B),
            Self::Type => (0x76D6BE, 0x006550),
            Self::Member => (0xEEC2DF, 0x863960),
            Self::Punctuation => (0xBCC5D1, 0x495363),
        };
        if dark {
            dark_color
        } else {
            light_color
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: i32,
    pub end: i32,
    pub kind: Kind,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Normal,
    PsComment,
    PsSingle,
    PsDouble,
    PsHereSingle,
    PsHereDouble,
    XmlComment,
    XmlTag,
    XmlSingle,
    XmlDouble,
    YamlBlock(usize),
}
struct Line {
    text: String,
    start: i32,
    len: i32,
    input: State,
    state: State,
    spans: Vec<Span>,
}
#[derive(Default)]
pub struct Cache {
    context: Context,
    lines: Vec<Line>,
    pub revision: u64,
    pub parsed_lines: usize,
}
impl Cache {
    pub fn update(&mut self, text: &str, context: Context) {
        if context == Context::Plain {
            if self.context != Context::Plain || !self.lines.is_empty() {
                self.lines.clear();
                self.context = context;
                self.revision = self.revision.wrapping_add(1);
            }
            self.parsed_lines = 0;
            return;
        }
        let parts: Vec<&str> = text.split_inclusive('\n').collect();
        let same_context = self.context == context;
        let mut prefix = if same_context {
            self.lines
                .iter()
                .zip(&parts)
                .take_while(|(a, b)| a.text == **b)
                .count()
        } else {
            0
        };
        if prefix == parts.len() && prefix == self.lines.len() && same_context {
            self.parsed_lines = 0;
            return;
        }
        // JSON allows arbitrary whitespace between a key and ':'. Include the
        // previous nonblank line when changing its lookahead.
        prefix = prefix.saturating_sub(1);
        while prefix > 0 && parts.get(prefix).is_some_and(|s| s.trim().is_empty()) {
            prefix -= 1;
        }
        let mut lookahead = vec![None; parts.len()];
        let mut next_nonblank = None;
        for (i, part) in parts.iter().enumerate().rev() {
            lookahead[i] = next_nonblank;
            if let Some(c) = part.chars().find(|c| !c.is_whitespace()) {
                next_nonblank = Some(c);
            }
        }
        let mut suffix = 0;
        if same_context {
            while suffix < self.lines.len().saturating_sub(prefix)
                && suffix < parts.len().saturating_sub(prefix)
                && self.lines[self.lines.len() - 1 - suffix].text == parts[parts.len() - 1 - suffix]
            {
                suffix += 1;
            }
        }
        let old_len = self.lines.len();
        let mut old = std::mem::take(&mut self.lines).into_iter();
        let mut lines: Vec<Line> = old.by_ref().take(prefix).collect();
        let suffix_old = old_len - suffix;
        let mut remaining: Vec<Option<Line>> = old.map(Some).collect();
        let mut state = lines.last().map(|l| l.state).unwrap_or_default();
        let mut start = lines.last().map(|l| l.start + l.len).unwrap_or(0);
        self.parsed_lines = 0;
        for (i, part) in parts.iter().enumerate().skip(prefix) {
            let old_index = if i >= parts.len() - suffix {
                Some(suffix_old + i - (parts.len() - suffix))
            } else {
                None
            };
            let old_input = old_index.and_then(|o| remaining[o - prefix].as_ref().map(|l| l.input));
            let mut line = if old_input == Some(state) && i > prefix {
                remaining[old_index.unwrap() - prefix].take().unwrap()
            } else {
                self.parsed_lines += 1;
                let next = lookahead[i];
                let (spans, after) = lex(part, context, state, next);
                Line {
                    text: (*part).into(),
                    start,
                    len: part.encode_utf16().count() as i32,
                    input: state,
                    state: after,
                    spans,
                }
            };
            line.start = start;
            start += line.len;
            state = line.state;
            lines.push(line);
        }
        self.lines = lines;
        self.context = context;
        self.revision = self.revision.wrapping_add(1);
    }
    pub fn spans(&self, start: i32, end: i32) -> Vec<Span> {
        let first = self.lines.partition_point(|l| l.start + l.len <= start);
        self.lines[first..]
            .iter()
            .take_while(|l| l.start < end)
            .flat_map(|l| {
                l.spans.iter().map(|s| Span {
                    start: s.start + l.start,
                    end: s.end + l.start,
                    kind: s.kind,
                })
            })
            .filter(|s| s.end > start && s.start < end)
            .collect()
    }
}
fn ascii(s: &[u16]) -> String {
    String::from_utf16_lossy(s).to_ascii_lowercase()
}
fn word(c: u16) -> bool {
    c >= 128 || (c as u8).is_ascii_alphanumeric() || matches!(c, 95 | 45)
}
fn ws(c: u16) -> bool {
    matches!(c, 9 | 10 | 13 | 32)
}
fn at(s: &[u16], i: usize, pattern: &str) -> bool {
    s.get(i..i + pattern.len())
        .is_some_and(|x| x.iter().copied().eq(pattern.bytes().map(u16::from)))
}
fn push(out: &mut Vec<Span>, start: usize, end: usize, kind: Kind) {
    if end > start {
        out.push(Span {
            start: start as i32,
            end: end as i32,
            kind,
        });
    }
}
fn variable(s: &[u16], i: &mut usize) {
    *i += 1;
    if s.get(*i) == Some(&123) {
        *i += 1;
        while *i < s.len() && s[*i] != 125 {
            *i += 1;
        }
        if *i < s.len() {
            *i += 1;
        }
    } else {
        while *i < s.len() && (word(s[*i]) || matches!(s[*i], 58 | 63 | 94)) {
            *i += 1;
        }
    }
}
fn quoted(s: &[u16], i: &mut usize, quote: u16, context: Context, out: &mut Vec<Span>) -> bool {
    let ps = context == Context::PowerShell;
    let mut begin = *i;
    while *i < s.len() {
        let c = s[*i];
        if (ps && quote == 34 && c == 96)
            || (matches!(context, Context::Json | Context::Yaml) && quote == 34 && c == 92)
        {
            *i = (*i + 2).min(s.len());
            continue;
        }
        if ps && quote == 34 && c == 36 {
            push(out, begin, *i, Kind::String);
            begin = *i;
            variable(s, i);
            push(out, begin, *i, Kind::Variable);
            begin = *i;
            continue;
        }
        *i += 1;
        if c == quote {
            if (ps || (context == Context::Yaml && quote == 39)) && s.get(*i) == Some(&quote) {
                *i += 1;
                continue;
            }
            push(out, begin, *i, Kind::String);
            return true;
        }
    }
    push(out, begin, *i, Kind::String);
    false
}
fn lex(text: &str, context: Context, mut state: State, next: Option<char>) -> (Vec<Span>, State) {
    let s: Vec<u16> = text.encode_utf16().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut command = true;
    let mut declaration = false;
    if context == Context::Plain {
        return (out, State::Normal);
    }
    if let State::YamlBlock(indent) = state {
        let n = s.iter().take_while(|&&c| c == 32).count();
        if n > indent || s.iter().all(|&c| ws(c)) {
            push(&mut out, 0, s.len(), Kind::String);
            return (out, state);
        }
        state = State::Normal;
    }
    while i < s.len() {
        let begin = i;
        match state {
            State::PsComment => {
                while i < s.len() && !at(&s, i, "#>") {
                    i += 1;
                }
                if i < s.len() {
                    i += 2;
                    state = State::Normal;
                }
                push(&mut out, begin, i, Kind::Comment);
                continue;
            }
            State::PsHereSingle | State::PsHereDouble => {
                let quote = if state == State::PsHereSingle { 39 } else { 34 };
                if i == 0 && s.first() == Some(&quote) && s.get(1) == Some(&64) {
                    push(&mut out, 0, 2, Kind::String);
                    i = 2;
                    state = State::Normal;
                    continue;
                }
                if state == State::PsHereDouble && s[i] == 36 {
                    variable(&s, &mut i);
                    push(&mut out, begin, i, Kind::Variable);
                } else {
                    i += 1;
                    while i < s.len() && !(state == State::PsHereDouble && s[i] == 36) {
                        if s[i] == 96 {
                            i = (i + 2).min(s.len());
                        } else {
                            i += 1;
                        }
                    }
                    push(&mut out, begin, i, Kind::String);
                }
                continue;
            }
            State::PsSingle | State::PsDouble => {
                let q = if state == State::PsSingle { 39 } else { 34 };
                if quoted(&s, &mut i, q, Context::PowerShell, &mut out) {
                    state = State::Normal;
                }
                continue;
            }
            State::XmlComment => {
                while i < s.len() && !at(&s, i, "-->") {
                    i += 1;
                }
                if i < s.len() {
                    i += 3;
                    state = State::Normal;
                }
                push(&mut out, begin, i, Kind::Comment);
                continue;
            }
            State::XmlSingle | State::XmlDouble => {
                let q = if state == State::XmlSingle { 39 } else { 34 };
                if quoted(&s, &mut i, q, Context::Xml, &mut out) {
                    state = State::XmlTag;
                }
                continue;
            }
            _ => {}
        }
        if ws(s[i]) {
            i += 1;
            continue;
        }
        if context == Context::PowerShell {
            if at(&s, i, "<#") {
                state = State::PsComment;
                i += 2;
                push(&mut out, begin, i, Kind::Comment);
                continue;
            }
            if s[i] == 35 {
                push(&mut out, i, s.len(), Kind::Comment);
                break;
            }
            if s[i] == 64
                && matches!(s.get(i + 1), Some(34 | 39))
                && s[i + 2..].iter().all(|&c| ws(c))
            {
                state = if s[i + 1] == 39 {
                    State::PsHereSingle
                } else {
                    State::PsHereDouble
                };
                push(&mut out, i, s.len(), Kind::String);
                break;
            }
            if matches!(s[i], 34 | 39) {
                let q = s[i];
                i += 1;
                push(&mut out, begin, i, Kind::String);
                if !quoted(&s, &mut i, q, Context::PowerShell, &mut out) {
                    state = if q == 39 {
                        State::PsSingle
                    } else {
                        State::PsDouble
                    };
                }
                command = false;
                continue;
            }
            if s[i] == 36 || (s[i] == 64 && s.get(i + 1).is_some_and(|&c| word(c))) {
                variable(&s, &mut i);
                let k = if matches!(ascii(&s[begin..i]).as_str(), "$true" | "$false" | "$null") {
                    Kind::Literal
                } else {
                    Kind::Variable
                };
                push(&mut out, begin, i, k);
                command = false;
                continue;
            }
            if s[i] == 91 {
                i += 1;
                while i < s.len() && s[i] != 93 {
                    i += 1;
                }
                if i < s.len() {
                    i += 1;
                }
                push(&mut out, begin, i, Kind::Type);
                continue;
            }
            if s[i] == 45
                && s.get(i + 1)
                    .is_some_and(|&c| word(c) && !(48..=57).contains(&c))
            {
                i += 1;
                while i < s.len() && word(s[i]) {
                    i += 1;
                }
                let k = if matches!(
                    ascii(&s[begin..i]).as_str(),
                    "-eq"
                        | "-ne"
                        | "-gt"
                        | "-lt"
                        | "-ge"
                        | "-le"
                        | "-like"
                        | "-notlike"
                        | "-match"
                        | "-notmatch"
                        | "-and"
                        | "-or"
                        | "-not"
                        | "-in"
                        | "-notin"
                        | "-contains"
                        | "-is"
                        | "-as"
                        | "-replace"
                        | "-split"
                        | "-join"
                ) {
                    Kind::Keyword
                } else {
                    Kind::Parameter
                };
                push(&mut out, begin, i, k);
                continue;
            }
            if (48..=57).contains(&s[i])
                || (s[i] == 45 && s.get(i + 1).is_some_and(|c| (48..=57).contains(c)))
            {
                i += 1;
                while i < s.len() && (word(s[i]) || s[i] == 46) {
                    i += 1;
                }
                push(&mut out, begin, i, Kind::Number);
                command = false;
                continue;
            }
            if word(s[i]) {
                i += 1;
                while i < s.len() && word(s[i]) {
                    i += 1;
                }
                let w = ascii(&s[begin..i]);
                let keyword = matches!(
                    w.as_str(),
                    "function"
                        | "filter"
                        | "class"
                        | "enum"
                        | "param"
                        | "if"
                        | "else"
                        | "elseif"
                        | "foreach"
                        | "for"
                        | "while"
                        | "do"
                        | "until"
                        | "switch"
                        | "try"
                        | "catch"
                        | "finally"
                        | "throw"
                        | "return"
                        | "break"
                        | "continue"
                        | "begin"
                        | "process"
                        | "end"
                        | "in"
                        | "using"
                        | "data"
                        | "dynamicparam"
                        | "trap"
                        | "exit"
                );
                if keyword {
                    push(&mut out, begin, i, Kind::Keyword);
                    declaration = matches!(w.as_str(), "function" | "filter" | "class" | "enum");
                    command = true;
                } else {
                    let before = s[..begin].iter().rfind(|&&c| !ws(c)).copied();
                    let after = s[i..].iter().find(|&&c| !ws(c)).copied();
                    let k = if after == Some(61) {
                        Some(Kind::Key)
                    } else if before == Some(46) {
                        Some(Kind::Member)
                    } else if declaration || command || after == Some(40) {
                        Some(Kind::Command)
                    } else {
                        None
                    };
                    if let Some(k) = k {
                        push(&mut out, begin, i, k);
                    }
                    declaration = false;
                    command = false;
                }
                continue;
            }
            if s[i] == 96 {
                i = (i + 2).min(s.len());
                continue;
            }
            command = matches!(s[i], 124 | 59 | 123 | 125 | 38 | 61 | 40);
            i += 1;
            push(&mut out, begin, i, Kind::Punctuation);
            continue;
        }
        if context == Context::Xml {
            if at(&s, i, "<!--") {
                state = State::XmlComment;
                i += 4;
                push(&mut out, begin, i, Kind::Comment);
                continue;
            }
            if s[i] == 60 {
                i += 1;
                while i < s.len() && matches!(s[i], 47 | 63 | 33) {
                    i += 1;
                }
                while i < s.len() && (word(s[i]) || s[i] == 58) {
                    i += 1;
                }
                push(&mut out, begin, i, Kind::Keyword);
                state = State::XmlTag;
                continue;
            }
            if state == State::XmlTag {
                if matches!(s[i], 34 | 39) {
                    let q = s[i];
                    i += 1;
                    push(&mut out, begin, i, Kind::String);
                    if !quoted(&s, &mut i, q, Context::Xml, &mut out) {
                        state = if q == 39 {
                            State::XmlSingle
                        } else {
                            State::XmlDouble
                        };
                    }
                    continue;
                }
                if s[i] == 62 {
                    i += 1;
                    push(&mut out, begin, i, Kind::Keyword);
                    state = State::Normal;
                    continue;
                }
                if word(s[i]) {
                    i += 1;
                    while i < s.len() && (word(s[i]) || s[i] == 58) {
                        i += 1;
                    }
                    push(&mut out, begin, i, Kind::Key);
                    continue;
                }
                i += 1;
                push(&mut out, begin, i, Kind::Punctuation);
                continue;
            }
            if s[i] == 38 {
                i += 1;
                while i < s.len() && !ws(s[i]) && s[i] != 59 {
                    i += 1;
                }
                if i < s.len() && s[i] == 59 {
                    i += 1;
                }
                push(&mut out, begin, i, Kind::Literal);
                continue;
            }
            i += 1;
            continue;
        }
        if matches!(context, Context::Yaml | Context::Ini)
            && ((s[i] == 35 && (i == 0 || ws(s[i - 1]))) || (context == Context::Ini && s[i] == 59))
        {
            push(&mut out, i, s.len(), Kind::Comment);
            break;
        }
        if context == Context::Ini && s[i] == 91 {
            i += 1;
            while i < s.len() && s[i] != 93 {
                i += 1;
            }
            if i < s.len() {
                i += 1;
            }
            push(&mut out, begin, i, Kind::Keyword);
            continue;
        }
        if context == Context::Yaml && matches!(s[i], 124 | 62) {
            let indent = s.iter().take_while(|&&c| c == 32).count();
            push(&mut out, i, s.len(), Kind::Punctuation);
            state = State::YamlBlock(indent);
            break;
        }
        if s[i] == 34 || (context != Context::Json && s[i] == 39) {
            let q = s[i];
            i += 1;
            push(&mut out, begin, i, Kind::String);
            quoted(&s, &mut i, q, context, &mut out);
            if s[i..]
                .iter()
                .find(|&&c| !ws(c))
                .copied()
                .map(u32::from)
                .or(next.map(|c| c as u32))
                == Some(58)
            {
                out.retain(|p| p.start < begin as i32);
                push(&mut out, begin, i, Kind::Key);
            }
            continue;
        }
        if matches!(s[i], 123 | 125 | 91 | 93 | 58 | 44 | 61) {
            i += 1;
            push(&mut out, begin, i, Kind::Punctuation);
            continue;
        }
        if context == Context::Yaml && matches!(s[i], 38 | 42 | 33) {
            i += 1;
            while i < s.len() && !ws(s[i]) {
                i += 1;
            }
            push(&mut out, begin, i, Kind::Type);
            continue;
        }
        i += 1;
        while i < s.len()
            && !ws(s[i])
            && !matches!(s[i], 123 | 125 | 91 | 93 | 58 | 44 | 61 | 34 | 39)
        {
            i += 1;
        }
        let value = ascii(&s[begin..i]);
        let after = s[i..].iter().find(|&&c| !ws(c)).copied();
        let key = (context == Context::Yaml && after == Some(58))
            || (context == Context::Ini && after == Some(61));
        let kind = if key {
            Some(Kind::Key)
        } else if matches!(value.as_str(), "true" | "false" | "null" | "~") {
            Some(Kind::Literal)
        } else if value.parse::<f64>().is_ok() {
            Some(Kind::Number)
        } else {
            None
        };
        if let Some(k) = kind {
            push(&mut out, begin, i, k);
        }
    }
    (out, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tokens(text: &str, c: Context) -> Vec<(String, Kind)> {
        let mut cache = Cache::default();
        cache.update(text, c);
        let u: Vec<_> = text.encode_utf16().collect();
        cache
            .spans(0, u.len() as i32)
            .iter()
            .map(|s| {
                (
                    String::from_utf16_lossy(&u[s.start as usize..s.end as usize]),
                    s.kind,
                )
            })
            .collect()
    }
    #[test]
    fn powershell_blocks_and_tokens() {
        let text="function Test-Thing {\n<# multiline\ncomment #>\n[System.String] $env:Path\nGet-Process -Name \"hi $name\" | Where-Object { $_.Id -gt 12 }\n@'\n$literal\n'@\n}\n";
        let t = tokens(text, Context::PowerShell);
        for (word, kind) in [
            ("function", Kind::Keyword),
            ("Test-Thing", Kind::Command),
            ("[System.String]", Kind::Type),
            ("$env:Path", Kind::Variable),
            ("Get-Process", Kind::Command),
            ("-Name", Kind::Parameter),
            ("$name", Kind::Variable),
            ("-gt", Kind::Keyword),
            ("12", Kind::Number),
        ] {
            assert!(t.contains(&(word.into(), kind)), "{word}: {t:?}");
        }
        assert!(t
            .iter()
            .any(|(s, k)| s.contains("$literal") && *k == Kind::String));
        assert!(t
            .iter()
            .any(|(s, k)| s.contains("comment") && *k == Kind::Comment));
    }
    #[test]
    fn json_and_unicode_offsets() {
        let t = tokens(
            "{\"😀\": \"hi\\\"x\",\n\"key\"\n: true, \"n\": -2.5e+3, \"null\": null}",
            Context::Json,
        );
        for (s, k) in [
            ("\"😀\"", Kind::Key),
            ("\"key\"", Kind::Key),
            ("true", Kind::Literal),
            ("-2.5e+3", Kind::Number),
        ] {
            assert!(t.contains(&(s.into(), k)), "{t:?}");
        }
    }
    #[test]
    fn config_formats() {
        assert!(tokens("key: |\n  # literal\nnext: true\n", Context::Yaml)
            .iter()
            .any(|(s, k)| s.contains("# literal") && *k == Kind::String));
        assert!(tokens(
            "<root x=\"value\">\n<!--hi\nthere--> &amp;</root>",
            Context::Xml
        )
        .iter()
        .any(|(s, k)| s == "x" && *k == Kind::Key));
        assert!(tokens("[section]\nport=443\n# note\n", Context::Ini)
            .contains(&("port".into(), Kind::Key)));
        assert!(!tokens("key: \"$notAPowerShellVariable\"", Context::Yaml)
            .iter()
            .any(|(_, k)| *k == Kind::Variable));
    }
    #[test]
    fn cache_state_convergence() {
        let mut c = Cache::default();
        let original = (0..10000)
            .map(|i| format!("Get-Item -Path 'item{i}'\n"))
            .collect::<String>();
        c.update(&original, Context::PowerShell);
        c.update(
            &original.replace("item5000", "changed5000"),
            Context::PowerShell,
        );
        assert!(c.parsed_lines <= 3, "{}", c.parsed_lines);
        let changed = original.replacen("Get-Item", "<#", 1);
        c.update(&changed, Context::PowerShell);
        assert_eq!(c.spans(0, 10).first().unwrap().kind, Kind::Comment);
        c.update(&original, Context::Plain);
        assert!(c.spans(0, 100).is_empty());
    }
    #[test]
    fn incremental_matches_full_lexing() {
        for context in Context::ALL {
            let mut cache = Cache::default();
            let mut lines = vec![
                "\"key\"\n",
                "\n",
                ": true\n",
                "<#\n",
                "Get-Item $env:Path\n",
                "#>\n",
                "@\"\n",
                "$x\n",
                "\"@\n",
                "<!--\n",
                "tag x=\"y\"\n",
                "-->\n",
                "key: |\n",
                "  value\n",
            ];
            for i in 0..120 {
                let at = (i * 7 + 3) % lines.len();
                lines[at] = [
                    "# comment\n",
                    "\"value\"\n",
                    "\n",
                    "'@\n",
                    "x: true\n",
                    "#>\n",
                ][i % 6];
                let text = lines.concat();
                cache.update(&text, context);
                let mut full = Cache::default();
                full.update(&text, context);
                assert_eq!(
                    cache.spans(0, i32::MAX),
                    full.spans(0, i32::MAX),
                    "{context:?} edit {i}"
                );
                if i % 5 == 0 {
                    lines.insert(at, "\n");
                } else if i % 3 == 0 && lines.len() > 3 {
                    lines.remove(at);
                }
            }
        }
    }
    #[test]
    fn large_buffer_cache_timings() {
        let text = (0..100000)
            .map(|i| format!("Get-Item -Path 'item{i}' # comment\n"))
            .collect::<String>();
        let mut cache = Cache::default();
        let started = std::time::Instant::now();
        cache.update(&text, Context::PowerShell);
        let initial = started.elapsed();
        let changed = text.replace("item50000", "changed50000");
        let started = std::time::Instant::now();
        cache.update(&changed, Context::PowerShell);
        let edit = started.elapsed();
        assert!(cache.parsed_lines <= 3);
        eprintln!("Syntax benchmark: {} bytes, initial {initial:?}, one-line edit {edit:?}, {} lines lexed",text.len(),cache.parsed_lines);
    }
    #[test]
    fn theme_contrast() {
        fn lum(c: u32) -> f64 {
            [16, 8, 0]
                .into_iter()
                .zip([0.2126, 0.7152, 0.0722])
                .map(|(s, w)| {
                    let n = ((c >> s) & 255) as f64 / 255.;
                    w * if n <= 0.04045 {
                        n / 12.92
                    } else {
                        ((n + 0.055) / 1.055).powf(2.4)
                    }
                })
                .sum()
        }
        for dark in [true, false] {
            let bg = lum(if dark { 0x20242B } else { 0xF8FAFC });
            for k in Kind::ALL {
                let fg = lum(k.color(dark));
                let ratio = (fg.max(bg) + 0.05) / (fg.min(bg) + 0.05);
                assert!(ratio >= 4.5, "{k:?} dark={dark}: {ratio}");
            }
        }
    }
}
