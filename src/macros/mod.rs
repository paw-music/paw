// macro_rules! const_family {
//     ($vis: vis trait $name: ident ($impl_name: ident) {
//         $(const $const: ident: $ty: ty = $method: ident;)*
//     }) => {
//         $vis trait $name {
//             $(const $const: $ty;)*
//         }

//         pub struct $impl_name<$(
//             const $const: $ty,
//         )*>;

//         impl<$(const $const: $ty,)*> $name for $impl_name {
//             $(const $const: $ty = $const;)*

//             $(
//                 pub fn $method<const NEW: $ty>(self) -> $impl_name
//             )
//         }
//     };
// }

// const_family! {
//     pub trait Family (FamilyImpl) {
//         const A: i32 = a;
//     }
// }

macro_rules! debug_assert_unit {
    ($unit: expr) => {
        debug_assert!(
            $unit <= 1.0 && $unit >= -1.0,
            "Expected {} to be a unit vector in range [-1.0; 1.0], got {}",
            stringify!($unit),
            $unit
        );
    };

    ($($unit: expr),* $(,)?) => {
        $($crate::macros::debug_assert_unit!($unit);)*
    };
}

#[allow(unused)]
pub(crate) use debug_assert_unit;
