fn main() {
    // Intentionally empty.
    //
    // The tokenizer dictionary (SudachiDict small+core plus extra vocabulary)
    // is built by scripts/build_sudachidict.py into cdn/dictionaries/ — there
    // is nothing to generate at compile time anymore. The runtime user
    // dictionary was removed: it kept a second POS schema in play which
    // silently broke Token::get for its words (どういう & co. surfaced as
    // Unspecified POS and never reached card analysis).
}
