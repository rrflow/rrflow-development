//! The data RRFlow Lab tests with. `corpus` holds vector corpora, generated
//! from fixed arithmetic or imported by digest, with the exact answers that grade
//! every system. `knowledge` turns a repository's text into bounded chunks that
//! reasoning tasks retrieve.
pub mod corpus;
pub mod knowledge;
