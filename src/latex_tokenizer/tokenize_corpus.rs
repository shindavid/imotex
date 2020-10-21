// Copyright (c) 2020 Peter Jin.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate latex_tokenizer;

use latex_tokenizer::{AnnotatedLatexDocument};

use std::path::{PathBuf};

fn main() {
  let corpus: &[(&str, &str, &[&str])] = &[
    ("imo", "1986", &["q1", "q2", "q3", "q4", "q5", "q6"]),
    ("imo", "2006", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8", "g9", "g10"]),
    ("imo", "2009", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2010", &["g1", "g2", "g3", "g4", "g5", "g6", "g6a", "g7"]),
    ("imo", "2011", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2012", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2013", &["g1", "g2", "g3", "g4", "g5", "g6"]),
    ("imo", "2014", &["g1", "g2", "g3", "g4", "g5", "g6", "g7"]),
    ("imo", "2015", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2016", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2017", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2018", &["g1", "g2", "g3", "g4", "g5", "g6", "g7"]),
    ("imo", "2019", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2020", &["q1", "s1-dshin", "q2", "q3", "q4", "q5", "q6"]),
    ("ireland", "1996", &["q1", "q2", "q3", "q4", "q7", "q8"]),
  ];
  let mut errs = Vec::new();
  for &(tag, year, qs) in corpus.iter() {
    for q in qs.iter() {
      let doc = AnnotatedLatexDocument::open(&PathBuf::from(format!("{}/{}/{}.txt", tag, year, q))).unwrap();
      let tokens = doc.tokenize();
      if tokens.is_err() {
        errs.push((tag, year, q));
      }
      println!("DEBUG: {}/{}/{}: {:?}", tag, year, q, tokens);
    }
  }
  if errs.len() == 0 {
    println!("DEBUG: all passed");
  } else {
    println!("DEBUG: errors: n={} {:?}", errs.len(), errs);
  }
}
