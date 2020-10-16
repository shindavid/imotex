// Copyright (c) 2020 Peter Jin.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate latex_tokenizer;

use latex_tokenizer::{AnnotatedLatex};

use std::path::{PathBuf};

fn main() {
  let corpus: &[(&str, &str, &[&str])] = &[
    // FIXME: support for displaymode math.
    //("imo", "2011", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2012", &["g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8"]),
    ("imo", "2013", &["g1", "g2", "g3", "g4", "g5", "g6"]),
  ];
  for &(tag, year, qs) in corpus.iter() {
    for q in qs.iter() {
      let latex = AnnotatedLatex::open(&PathBuf::from(format!("{}/{}/{}.txt", tag, year, q))).unwrap();
      let tokens = latex.tokenize();
      println!("DEBUG: {}/{}/{}: {:?}", tag, year, q, tokens);
    }
  }
}
