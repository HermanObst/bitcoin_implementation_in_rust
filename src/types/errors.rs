use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Errors {
    #[error("Point is not included in the curve")]
    InvalidPoint,
    #[error("Cannot add points from different curves")]
    DifferentCurves,
}