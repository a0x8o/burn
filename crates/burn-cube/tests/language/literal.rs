use burn_cube::{cube, Float};

#[cube]
pub fn literal<F: Float>(lhs: F) {
    let _ = lhs + F::from_int(5);
}

// #[cube(debug)]
// pub fn literal_float_no_decimals<F: Float>(lhs: F) {
//     let _ = lhs + F::new(5.);
// }

#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn literal_float_no_decimals<F: Float>(lhs: F) {
    let _ = lhs + F::new(5.);
}
#[allow(unused_mut)]
#[allow(clippy::too_many_arguments)]
pub fn literal_float_no_decimals_expand<F: Float>(
    context: &mut burn_cube::CubeContext,
    lhs: <F as burn_cube::CubeType>::ExpandType,
) -> () {
    let _ = {
        let _lhs = lhs;
        let _rhs = F::new_expand(context, 5f32);
        burn_cube::add::expand(context, _lhs, _rhs)
    };
}

mod tests {
    use super::{literal_expand, literal_float_no_decimals_expand};
    use burn_cube::{
        cpa,
        dialect::{Item, Variable},
        CubeContext, CubeElem, F32,
    };

    type ElemType = F32;

    #[test]
    fn cube_literal_test() {
        let mut context = CubeContext::root();

        let lhs = context.create_local(Item::new(ElemType::as_elem()));

        literal_expand::<ElemType>(&mut context, lhs);
        let scope = context.into_scope();

        assert_eq!(format!("{:?}", scope.operations), inline_macro_ref());
    }

    #[test]
    fn cube_literal_float_no_decimal_test() {
        let mut context = CubeContext::root();

        let lhs = context.create_local(Item::new(ElemType::as_elem()));

        literal_float_no_decimals_expand::<ElemType>(&mut context, lhs);
        let scope = context.into_scope();

        assert_eq!(format!("{:?}", scope.operations), inline_macro_ref());
    }

    fn inline_macro_ref() -> String {
        let mut context = CubeContext::root();
        let item = Item::new(ElemType::as_elem());
        let lhs = context.create_local(item);

        let mut scope = context.into_scope();
        let lhs: Variable = lhs.into();
        cpa!(scope, lhs = lhs + 5.0f32);

        format!("{:?}", scope.operations)
    }
}
