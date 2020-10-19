// Copyright (c) 2019-2020 Peter Jin.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fs::{File};
use std::io::{BufRead, BufReader, Error as IoError};
use std::path::{Path};

#[derive(Debug)]
pub enum LatexTokenizerError {
  Io(IoError),
  Unexpected(&'static str),
  UnexpectedCmd(char),
  Unterminated(&'static str),
  MismatchedMath(&'static str, &'static str),
}

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
  LQuote,
  RQuote,
  LDQuote,
  RDQuote,
  VBar,
  Space,
  Super,
  Sub,
  LGroup,
  RGroup,
  StartInlineMath,
  EndInlineMath,
  StartDisplayMath,
  EndDisplayMath,
}

#[derive(Clone, Copy, Debug)]
enum MathStart {
  Dollar,
  LBrack,
}

#[derive(Debug)]
enum Item {
  Text(String),
  Number(String),
  Command(String),
  LQuote,
  RQuote,
  Space,
  InlineMath(MathStart, bool, String),
  DisplayMath(bool, String),
}

#[derive(Debug)]
enum Mode {
  Text,
  InlineMath,
  DisplayMath,
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

  fn _tokenize_math_char<F: FnOnce(&mut LatexTokenizer), G: Fn(&mut LatexTokenizer, String)>(&mut self, mut buf: String, c: char, push_math: F, push_math2: G) {
    if c == '+' || c == '-' || c == '<' || c == '>' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::Symbol(c).into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '.' || c == ',' || c == '?' || c == ';' || c == ':' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::Punct(c).into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '[' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::LBrack.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == ']' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::RBrack.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '(' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::LParen.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == ')' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::RParen.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '\\' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
      self.stack.push(Item::Command(String::new()));
    } else if c.is_whitespace() {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
      self.stack.push(Item::Space);
    } else if c == '^' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::Super.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '_' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::Sub.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '{' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::LGroup.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '}' {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::RGroup.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c == '\'' {
      /*buf.push(c);
      self.stack.push(Item::InlineMath(start, false, buf));*/
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      self.toks.push(LatexToken::LQuote.into());
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
    } else if c.is_numeric() {
      if !buf.is_empty() {
        self.toks.push(LatexToken::Text(buf).into());
      }
      //self.stack.push(Item::InlineMath(start, false, String::new()));
      (push_math)(self);
      self.stack.push(Item::Number(c.to_string()));
    } else {
      if buf.is_empty() {
        buf.push(c);
        //self.stack.push(Item::InlineMath(start, false, buf));
        (push_math2)(self, buf);
      } else {
        self.toks.push(LatexToken::Text(buf).into());
        //self.stack.push(Item::InlineMath(start, false, c.to_string()));
        (push_math2)(self, c.to_string());
      }
    }
  }

  fn _tokenize_char(&mut self, _idx: usize, c: char) -> Result<(), LatexTokenizerError> {
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
            self.stack.push(Item::InlineMath(MathStart::Dollar, true, String::new()));
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
            //panic!("BUG: syntax error: superscript in text mode");
            return Err(LatexTokenizerError::Unexpected("^"));
          } else if c == '_' {
            //panic!("BUG: syntax error: subscript in text mode");
            return Err(LatexTokenizerError::Unexpected("_"));
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
            self.toks.push(LatexToken::LQuote.into());
            continue;
          }
        }
        Some(Item::RQuote) => {
          if c == '\'' {
            self.toks.push(LatexToken::RDQuote.into());
          } else {
            self.toks.push(LatexToken::RQuote.into());
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
            match self.stack.pop() {
              Some(Item::Text(text_buf)) => {
                if !text_buf.is_empty() {
                  self.toks.push(LatexToken::Text(text_buf).into());
                }
                self.toks.push(LatexToken::StartInlineMath.into());
                self.stack.push(Item::Text(String::new()));
                self.stack.push(Item::InlineMath(MathStart::LBrack, false, String::new()));
              }
              Some(_) | None => {
                return Err(LatexTokenizerError::Unexpected("\\["));
              }
            }
          } else if buf.is_empty() && c == ']' {
            match self.stack.pop() {
              Some(Item::InlineMath(start, _init, math_buf)) => {
                match start {
                  MathStart::LBrack => {
                    if !math_buf.is_empty() {
                      self.toks.push(LatexToken::Text(math_buf).into());
                    }
                    self.toks.push(LatexToken::EndInlineMath.into());
                  }
                  MathStart::Dollar => {
                    return Err(LatexTokenizerError::MismatchedMath("$", "\\]"));
                  }
                }
              }
              Some(_) | None => {
                return Err(LatexTokenizerError::Unexpected("\\]"));
              }
            }
          } else if buf.is_empty() && c == '{' {
            self.toks.push(LatexToken::LCurly.into());
          } else if buf.is_empty() && c == '}' {
            self.toks.push(LatexToken::RCurly.into());
          } else if buf.is_empty() && c == '|' {
            self.toks.push(LatexToken::VBar.into());
          } else if c.is_alphabetic() {
            buf.push(c);
            self.stack.push(Item::Command(buf));
          } else {
            if buf.is_empty() {
              return Err(LatexTokenizerError::UnexpectedCmd(c));
            }
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
              Mode::InlineMath | Mode::DisplayMath => {}
            }
            continue;
          }
        }
        Some(Item::InlineMath(start, init, buf)) => {
          self.mode = Mode::InlineMath;
          if init {
            if c == '$' {
              match start {
                MathStart::Dollar => {
                  assert!(buf.is_empty(), "bug");
                  self.stack.push(Item::DisplayMath(false, buf));
                  break;
                }
                MathStart::LBrack => {
                  return Err(LatexTokenizerError::Unexpected("\\[$"));
                }
              }
            } else {
              self.toks.push(LatexToken::StartInlineMath.into());
            }
          }
          if c == '$' {
            match start {
              MathStart::Dollar => {
                if !buf.is_empty() {
                  self.toks.push(LatexToken::Text(buf).into());
                }
                self.toks.push(LatexToken::EndInlineMath.into());
              }
              MathStart::LBrack => {
                return Err(LatexTokenizerError::MismatchedMath("\\[", "$"));
              }
            }
          } else {
            self._tokenize_math_char(buf, c,
                move |this| this.stack.push(Item::InlineMath(start, false, String::new())),
                move |this, buf| this.stack.push(Item::InlineMath(start, false, buf)),
            );
          }
        }
        Some(Item::DisplayMath(end, buf)) => {
          self.mode = Mode::DisplayMath;
          if end {
            if c == '$' {
              self.toks.push(LatexToken::EndDisplayMath.into());
              break;
            } else {
              return Err(LatexTokenizerError::MismatchedMath("$$", "$"));
            }
          }
          if c == '$' {
            self.stack.push(Item::DisplayMath(true, buf));
          } else {
            self._tokenize_math_char(buf, c,
                move |this| this.stack.push(Item::DisplayMath(false, String::new())),
                move |this, buf| this.stack.push(Item::DisplayMath(false, buf)),
            );
          }
        }
        None => panic!("bug")
      }
      break;
    }
    Ok(())
  }

  pub fn tokenize(mut self, source: &str) -> Result<Vec<LatexToken>, LatexTokenizerError> {
    for (idx, c) in source.char_indices() {
      self._tokenize_char(idx, c)?;
    }
    match self.stack.pop() {
      Some(Item::Text(buf)) => {
        if !buf.is_empty() {
          self.toks.push(LatexToken::Text(buf).into());
        }
      }
      Some(Item::LQuote) => {
        self.toks.push(LatexToken::LQuote.into());
      }
      Some(Item::RQuote) => {
        self.toks.push(LatexToken::RQuote.into());
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
      Some(Item::InlineMath(start, ..)) => {
        let start = match start {
          MathStart::Dollar => "$",
          MathStart::LBrack => "\\[",
        };
        return Err(LatexTokenizerError::Unterminated(start));
      }
      Some(Item::DisplayMath(..)) => {
        return Err(LatexTokenizerError::Unterminated("$$"));
      }
      None => {}
    }
    assert!(self.stack.is_empty(), "bug");
    Ok(self.toks)
  }
}

#[derive(Clone)]
pub struct AnnotatedLatexDocument {
  content:  String,
}

impl AnnotatedLatexDocument {
  pub fn open<P: AsRef<Path>>(path: &P) -> Result<AnnotatedLatexDocument, LatexTokenizerError> {
    let f = File::open(path).map_err(|e| LatexTokenizerError::Io(e))?;
    let reader = BufReader::new(f);
    let mut header = true;
    let mut file = AnnotatedLatexDocument{content: String::new()};
    for line in reader.lines() {
      let line = line.map_err(|e| LatexTokenizerError::Io(e))?;
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

  pub fn tokenize(&self) -> Result<Vec<LatexToken>, LatexTokenizerError> {
    LatexTokenizer::new().tokenize(&self.content)
  }
}
