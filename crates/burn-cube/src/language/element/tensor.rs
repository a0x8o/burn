use crate::{
    dialect::{Elem, Item, Metadata, Vectorization},
    language::{indexation::Index, CubeType, ExpandElement},
    unexpanded, ArgSettings, CubeContext, CubeElem, KernelLauncher, LaunchArg, Runtime, UInt,
};
use std::marker::PhantomData;

#[derive(new, Clone, Copy)]
pub struct Tensor<T: CubeType> {
    _val: PhantomData<T>,
}

impl<T: CubeType> CubeType for Tensor<T> {
    type ExpandType = ExpandElement;
}

impl<C: CubeElem> LaunchArg for Tensor<C> {
    type RuntimeArg<'a, R: Runtime> = TensorHandle<'a, R>;

    fn compile_input(
        builder: &mut crate::KernelBuilder,
        vectorization: Vectorization,
    ) -> ExpandElement {
        builder.input_array(Item::vectorized(C::as_elem(), vectorization))
    }

    fn compile_output(
        builder: &mut crate::KernelBuilder,
        vectorization: Vectorization,
    ) -> ExpandElement {
        builder.output_array(Item::vectorized(C::as_elem(), vectorization))
    }
}

#[derive(new)]
pub struct TensorHandle<'a, R: Runtime> {
    pub handle: &'a burn_compute::server::Handle<R::Server>,
    pub strides: &'a [usize],
    pub shape: &'a [usize],
}

impl<'a, R: Runtime> ArgSettings<R> for TensorHandle<'a, R> {
    fn register(&self, launcher: &mut KernelLauncher<R>) {
        launcher.register_tensor(self)
    }
}

impl<T: CubeType> Tensor<T> {
    /// Obtain the stride of input at dimension dim
    pub fn stride<C: Index>(self, _dim: C) -> UInt {
        unexpanded!()
    }

    /// Obtain the shape of input at dimension dim
    pub fn shape<C: Index>(self, _dim: C) -> UInt {
        unexpanded!()
    }

    /// Obtain the array length of input
    pub fn len(self) -> UInt {
        unexpanded!()
    }
}

impl ExpandElement {
    // Expanded version of Tensor::stride
    pub fn stride_expand<C: Index>(self, context: &mut CubeContext, dim: C) -> ExpandElement {
        let out = context.create_local(Item::new(Elem::UInt));
        context.register(Metadata::Stride {
            dim: dim.value().into(),
            var: self.into(),
            out: out.clone().into(),
        });
        out
    }

    // Expanded version of Tensor::shape
    pub fn shape_expand<C: Index>(self, context: &mut CubeContext, dim: C) -> ExpandElement {
        let out = context.create_local(Item::new(Elem::UInt));
        context.register(Metadata::Shape {
            dim: dim.value().into(),
            var: self.into(),
            out: out.clone().into(),
        });
        out
    }

    // Expanded version of Tensor::len
    pub fn len_expand(self, context: &mut CubeContext) -> ExpandElement {
        let out = context.create_local(Item::new(Elem::UInt));
        context.register(Metadata::ArrayLength {
            var: self.into(),
            out: out.clone().into(),
        });
        out
    }
}
