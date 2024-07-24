mod m31_ext;
mod simd_m31_ext;

pub use m31_ext::M31Ext3;
pub use simd_m31_ext::SimdM31Ext3;

use crate::{Field, FieldSerde};

/// Configurations for Extension Field over
/// the Binomial polynomial x^DEGREE - W
pub trait BinomialExtensionField<const DEGREE: usize>:
    From<Self::BaseField> + Field + FieldSerde
{
    /// Extension Field
    const W: u32;

    /// Base field for the extension
    type BaseField: Field + FieldSerde + Send;

    /// Multiply the extension field with the base field
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self;

    /// Add the extension field with the base field
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self;
}
