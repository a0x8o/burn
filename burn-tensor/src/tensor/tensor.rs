use std::ops::Range;

use crate::{Data, Shape};

pub enum TensorError {
    ReshapeError(String),
}

pub trait FloatTensor<P: num_traits::Float, const D: usize>:
    TensorBase<P, D>
    + TensorOpsMul<P, D>
    + TensorOpsNeg<P, D>
    + TensorOpsAdd<P, D>
    + TensorOpsSub<P, D>
    + TensorOpsMatmul<P, D>
    + TensorOpsTranspose<P, D>
    + std::fmt::Debug
{
}

pub trait TensorBase<P, const D: usize> {
    fn shape(&self) -> &Shape<D>;
    fn into_data(self) -> Data<P, D>;
    fn to_data(&self) -> Data<P, D>;
}

pub trait TensorOpsAdd<P, const D: usize>:
    std::ops::Add<Self, Output = Self> + std::ops::Add<P, Output = Self>
where
    Self: Sized,
{
    fn add(&self, other: &Self) -> Self;
    fn add_scalar(&self, other: &P) -> Self;
}

pub trait TensorOpsSub<P, const D: usize>:
    std::ops::Sub<Self, Output = Self> + std::ops::Sub<P, Output = Self>
where
    Self: Sized,
{
    fn sub(&self, other: &Self) -> Self;
    fn sub_scalar(&self, other: &P) -> Self;
}

pub trait TensorOpsTranspose<P, const D: usize> {
    fn transpose(&self) -> Self;
}

pub trait TensorOpsMatmul<P, const D: usize> {
    fn matmul(&self, other: &Self) -> Self;
}

pub trait TensorOpsNeg<P, const D: usize>: std::ops::Neg<Output = Self> {
    fn neg(&self) -> Self;
}

pub trait TensorOpsMul<P, const D: usize>:
    std::ops::Mul<P, Output = Self> + std::ops::Mul<Self, Output = Self>
where
    Self: Sized,
{
    fn mul(&self, other: &Self) -> Self;
    fn mul_scalar(&self, other: &P) -> Self;
}

pub trait TensorOpsReshape<P, const D1: usize, const D2: usize, T: TensorBase<P, D2>> {
    fn reshape(&self, shape: Shape<D2>) -> T;
}

pub trait TensorOpsIndex<P, const D1: usize, const D2: usize> {
    fn index(&self, indexes: [Range<usize>; D2]) -> Self;
}
