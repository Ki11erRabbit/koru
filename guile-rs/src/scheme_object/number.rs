use crate::scheme_object::{Number, SchemeObject, SchemeProcedure};

/// Represents a Number in Scheme
/// This type provides a math interface to allow for ease of use while providing invariance.
pub struct SchemeNumber {
    base: SchemeObject,
}

impl SchemeNumber {
    
    /// Constructor from any Rust number type except u128 or i128
    pub fn new(number: impl Number) -> SchemeNumber {
        SchemeNumber {
            base: number.into()
        }
    }

    /// Internal Constructor
    /// This should never be called by the user as it would violate invariance.
    pub(crate) unsafe fn from_base(x: SchemeObject) -> SchemeNumber {
        SchemeNumber { base: x }
    }
}

impl Into<SchemeObject> for SchemeNumber {
    fn into(self) -> SchemeObject {
        self.base
    }
}

impl std::ops::Add for SchemeNumber {
    type Output = SchemeNumber;
    fn add(self, other: SchemeNumber) -> SchemeNumber {
        let add = SchemeProcedure::new("+");
        let result = add.call2(self, other);
        result.cast_number().unwrap()
    }
}

impl std::ops::Sub for SchemeNumber {
    type Output = SchemeNumber;
    fn sub(self, other: SchemeNumber) -> SchemeNumber {
        let sub = SchemeProcedure::new("-");
        let result = sub.call2(self, other);
        result.cast_number().unwrap()
    }
}

impl std::ops::Mul for SchemeNumber {
    type Output = SchemeNumber;
    fn mul(self, other: SchemeNumber) -> SchemeNumber {
        let mul = SchemeProcedure::new("*");
        let result = mul.call2(self, other);
        result.cast_number().unwrap()
    }
}

impl std::ops::Div for SchemeNumber {
    type Output = SchemeNumber;
    fn div(self, other: SchemeNumber) -> SchemeNumber {
        let div = SchemeProcedure::new("/");
        let result = div.call2(self, other);
        result.cast_number().unwrap()
    }
}

impl std::ops::Rem for SchemeNumber {
    type Output = SchemeNumber;
    fn rem(self, other: SchemeNumber) -> SchemeNumber {
        let rem = SchemeProcedure::new("%");
        let result = rem.call2(self, other);
        result.cast_number().unwrap()
    }
}