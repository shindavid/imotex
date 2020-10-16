A simple tokenizer for a subset of LaTeX.

Build with cargo:

    cargo build --release --lib --bins

Run the tokenizer on corpus documents:

    cd ../..
    ./src/latex_tokenizer/target/release/tokenize_corpus
