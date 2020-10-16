// Copyright (c) 2019-2020 Peter Jin.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fs::{File};
use std::io::{BufRead, BufReader, Error as IoError};
use std::path::{Path};

#[derive(Clone, Debug)]
pub enum LatexToken {
  Text(String),
  Number(String),
  Command(String),
  Symbol(char),
  Punct(char),
  LBrack,
  RBrack,
  LCurly,
  RCurly,
  LParen,
  RParen,
  LSQuote,
  RSQuote,
  LDQuote,
  RDQuote,
  Space,
  Super,
  Sub,
  LGroup,
  RGroup,
  StartInlineMath,
  EndInlineMath,
}

#[derive(Debug)]
enum Item {
  Text(String),
  Number(String),
  Command(String),
  LQuote,
  RQuote,
  Space,
  InlineMath(bool, String),
  DisplayMath(String),
}

#[derive(Clone, Copy, Debug)]
enum Mode {
  Text,
  InlineMath,
  //DisplayMath,
}

pub struct LatexTokenizer {
  stack:    Vec<Item>,
  mode:     Mode,
  toks:     Vec<LatexToken>,
}

impl LatexTokenizer {
  pub fn new() -> LatexTokenizer {
    LatexTokenizer{
      stack:    vec![Item::Text(String::new())],
      mode:     Mode::Text,
      toks:     Vec::new(),
    }
  }

  fn _tokenize_char(&mut self, _idx: usize, c: char) {
    loop {
      let item = self.stack.pop();
      match item {
        Some(Item::Text(mut buf)) => {
          self.mode = Mode::Text;
          if c == '$' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::Text(String::new()));
            self.stack.push(Item::InlineMath(true, String::new()));
          } else if c == '.' || c == ',' || c == '?' || c == ';' || c == ':' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::Punct(c).into());
            self.stack.push(Item::Text(String::new()));
          } else if c == '(' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::LParen.into());
            self.stack.push(Item::Text(String::new()));
          } else if c == ')' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::RParen.into());
            self.stack.push(Item::Text(String::new()));
          } else if c == '`' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::Text(String::new()));
            self.stack.push(Item::LQuote);
          } else if c == '\'' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::Text(String::new()));
            self.stack.push(Item::RQuote);
          } else if c == '\\' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::Text(String::new()));
            self.stack.push(Item::Command(c.to_string()));
          } else if c.is_whitespace() {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::Text(String::new()));
            self.stack.push(Item::Space);
          } else if c == '^' {
            panic!("BUG: syntax error: superscript in text mode");
          } else if c == '_' {
            panic!("BUG: syntax error: subscript in text mode");
          } else if c == '{' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::LGroup.into());
            self.stack.push(Item::Text(String::new()));
          } else if c == '}' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::RGroup.into());
            self.stack.push(Item::Text(String::new()));
          } else {
            buf.push(c);
            self.stack.push(Item::Text(buf));
          }
        }
        Some(Item::LQuote) => {
          if c == '`' {
            self.toks.push(LatexToken::LDQuote.into());
          } else {
            self.toks.push(LatexToken::LSQuote.into());
            continue;
          }
        }
        Some(Item::RQuote) => {
          if c == '\'' {
            self.toks.push(LatexToken::RDQuote.into());
          } else {
            self.toks.push(LatexToken::RSQuote.into());
            continue;
          }
        }
        Some(Item::Number(mut buf)) => {
          if c.is_numeric() {
            buf.push(c);
            self.stack.push(Item::Number(buf));
          } else {
            assert!(!buf.is_empty(), "bug");
            self.toks.push(LatexToken::Number(buf).into());
            continue;
          }
        }
        Some(Item::Command(mut buf)) => {
          // FIXME: symbolic commands.
          if buf.is_empty() && c == '[' {
            unimplemented!();
          } else if buf.is_empty() && c == ']' {
            unimplemented!();
          } else if buf.is_empty() && c == '{' {
            self.toks.push(LatexToken::LCurly.into());
          } else if buf.is_empty() && c == '}' {
            self.toks.push(LatexToken::RCurly.into());
          } else if c.is_alphabetic() {
            buf.push(c);
            self.stack.push(Item::Command(buf));
          } else {
            assert!(!buf.is_empty(), "bug");
            self.toks.push(LatexToken::Command(buf).into());
            continue;
          }
        }
        Some(Item::Space) => {
          if c.is_whitespace() {
            self.stack.push(item.unwrap());
          } else {
            match self.mode {
              Mode::Text => {
                self.toks.push(LatexToken::Space.into());
              }
              Mode::InlineMath => {}
            }
            continue;
          }
        }
        Some(Item::InlineMath(init, mut buf)) => {
          self.mode = Mode::InlineMath;
          if init {
            if c == '$' {
              assert!(buf.is_empty(), "bug");
              self.stack.push(Item::DisplayMath(buf));
              break;
            } else {
              self.toks.push(LatexToken::StartInlineMath.into());
            }
          }
          if c == '$' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::EndInlineMath.into());
          } else if c == '+' || c == '-' || c == '<' || c == '>' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::Symbol(c).into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '.' || c == ',' || c == '?' || c == ';' || c == ':' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::Punct(c).into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '[' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::LBrack.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == ']' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::RBrack.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '(' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::LParen.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == ')' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::RParen.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '\\' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::InlineMath(false, String::new()));
            self.stack.push(Item::Command(String::new()));
          } else if c.is_whitespace() {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::InlineMath(false, String::new()));
            self.stack.push(Item::Space);
          } else if c == '^' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::Super.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '_' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::Sub.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '{' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::LGroup.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '}' {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.toks.push(LatexToken::RGroup.into());
            self.stack.push(Item::InlineMath(false, String::new()));
          } else if c == '\'' {
            buf.push(c);
            self.stack.push(Item::InlineMath(false, buf));
          } else if c.is_numeric() {
            if !buf.is_empty() {
              self.toks.push(LatexToken::Text(buf).into());
            }
            self.stack.push(Item::InlineMath(false, String::new()));
            self.stack.push(Item::Number(c.to_string()));
          } else {
            if buf.is_empty() {
              buf.push(c);
              self.stack.push(Item::InlineMath(false, buf));
            } else {
              self.toks.push(LatexToken::Text(buf).into());
              self.stack.push(Item::InlineMath(false, c.to_string()));
            }
          }
        }
        Some(Item::DisplayMath(..)) => {
          unimplemented!();
        }
        _ => unimplemented!()
      }
      break;
    }
  }

  pub fn tokenize(mut self, source: &str) -> Vec<LatexToken> {
    for (idx, c) in source.char_indices() {
      self._tokenize_char(idx, c);
    }
    match self.stack.pop() {
      Some(Item::Text(buf)) => {
        if !buf.is_empty() {
          self.toks.push(LatexToken::Text(buf).into());
        }
      }
      Some(Item::LQuote) => {
        self.toks.push(LatexToken::LSQuote.into());
      }
      Some(Item::RQuote) => {
        self.toks.push(LatexToken::RSQuote.into());
      }
      Some(Item::Number(buf)) => {
        if !buf.is_empty() {
          self.toks.push(LatexToken::Number(buf).into());
        }
      }
      Some(Item::Command(buf)) => {
        if !buf.is_empty() {
          self.toks.push(LatexToken::Command(buf).into());
        }
      }
      Some(Item::Space) => {
        self.toks.push(LatexToken::Space.into());
      }
      Some(Item::InlineMath(..)) => {
        panic!("BUG: syntax error: unterminated inline math");
      }
      Some(Item::DisplayMath(..)) => {
        unimplemented!();
      }
      None => {}
    }
    assert!(self.stack.is_empty(), "bug");
    self.toks
  }
}

#[derive(Clone)]
pub struct AnnotatedLatex {
  content:  String,
}

impl AnnotatedLatex {
  pub fn open<P: AsRef<Path>>(path: &P) -> Result<AnnotatedLatex, IoError> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut header = true;
    let mut file = AnnotatedLatex{content: String::new()};
    for line in reader.lines() {
      let line = line.unwrap();
      if header {
        let mut it = line.chars().take(2);
        match it.next() {
          Some('%') => {}
          Some(_) | None => {
            header = false;
          }
        }
        match it.next() {
          Some('%') => {}
          Some(_) | None => {
            header = false;
          }
        }
      }
      if header {
        // TODO
        continue;
      }
      file.content.push_str(&line);
    }
    Ok(file)
  }

  pub fn tokenize(&self) -> Vec<LatexToken> {
    LatexTokenizer::new().tokenize(&self.content)
  }
}
