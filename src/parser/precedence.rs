#![allow(dead_code)]
/// Operator precedence levels for Pratt parsing
/// Higher numbers = higher precedence (bind tighter)

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precedence(pub u8);

impl Precedence {
    pub const MIN: Precedence = Precedence(0);
    pub const PIPELINE: Precedence = Precedence(0);
    pub const ASSIGNMENT: Precedence = Precedence(1);
    pub const OR: Precedence = Precedence(2);
    pub const AND: Precedence = Precedence(3);
    pub const COMPARISON: Precedence = Precedence(4);
    pub const BITWISE_OR: Precedence = Precedence(5);
    pub const BITWISE_XOR: Precedence = Precedence(6);
    pub const BITWISE_AND: Precedence = Precedence(7);
    pub const BITWISE_SHIFT: Precedence = Precedence(8);
    pub const SUM: Precedence = Precedence(9);
    pub const PRODUCT: Precedence = Precedence(10);
    pub const EXPONENT: Precedence = Precedence(11);
    pub const UNARY: Precedence = Precedence(12);
    pub const CALL: Precedence = Precedence(13);
    pub const MAX: Precedence = Precedence(14);

    pub fn is_right_associative(self) -> bool {
        self == Self::ASSIGNMENT || self == Self::EXPONENT
    }
}
