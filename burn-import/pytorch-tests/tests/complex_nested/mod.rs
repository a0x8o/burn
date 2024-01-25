use burn::record::{FullPrecisionSettings, HalfPrecisionSettings, Recorder};

use burn::{
    module::Module,
    nn::{
        conv::{Conv2d, Conv2dConfig},
        BatchNorm, BatchNormConfig, Linear, LinearConfig,
    },
    tensor::{
        activation::{log_softmax, relu},
        backend::Backend,
        Tensor,
    },
};
use burn_import::pytorch::{LoadArgs, PyTorchFileRecorder};

#[derive(Module, Debug)]
struct ConvBlock<B: Backend> {
    conv: Conv2d<B>,
    norm: BatchNorm<B, 2>,
}

#[derive(Module, Debug)]
struct Net<B: Backend> {
    conv_blocks: Vec<ConvBlock<B>>,
    norm1: BatchNorm<B, 2>,
    fc1: Linear<B>,
    fc2: Linear<B>,
}

impl<B: Backend> Net<B> {
    /// Create a new model from the given record.
    pub fn new_with(record: NetRecord<B>) -> Self {
        let mut record = record;
        let record_block_0 = record.conv_blocks.remove(0);
        let record_block_1 = record.conv_blocks.remove(0);

        let conv_blocks = vec![
            ConvBlock {
                conv: Conv2dConfig::new([2, 4], [3, 2]).init_with(record_block_0.conv),
                norm: BatchNormConfig::new(2).init_with(record_block_0.norm),
            },
            ConvBlock {
                conv: Conv2dConfig::new([4, 6], [3, 2]).init_with(record_block_1.conv),
                norm: BatchNormConfig::new(4).init_with(record_block_1.norm),
            },
        ];
        let norm1 = BatchNormConfig::new(6).init_with(record.norm1);
        let fc1 = LinearConfig::new(120, 12).init_with(record.fc1);
        let fc2 = LinearConfig::new(12, 10).init_with(record.fc2);
        Self {
            conv_blocks,
            norm1,
            fc1,
            fc2,
        }
    }

    /// Forward pass of the model.
    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 2> {
        let x = self.conv_blocks[0].forward(x);
        let x = self.conv_blocks[1].forward(x);
        let x = self.norm1.forward(x);
        let x = x.reshape([0, -1]);
        let x = self.fc1.forward(x);
        let x = relu(x);
        let x = self.fc2.forward(x);

        log_softmax(x, 1)
    }
}

impl<B: Backend> ConvBlock<B> {
    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        let x = self.conv.forward(x);

        self.norm.forward(x)
    }
}

/// Partial model to test loading of partial records.
#[derive(Module, Debug)]
pub struct PartialNet<B: Backend> {
    conv1: ConvBlock<B>,
}

impl<B: Backend> PartialNet<B> {
    /// Create a new model from the given record.
    pub fn new_with(record: PartialNetRecord<B>) -> Self {
        let conv1 = ConvBlock {
            conv: Conv2dConfig::new([2, 4], [3, 2]).init_with(record.conv1.conv),
            norm: BatchNormConfig::new(2).init_with(record.conv1.norm),
        };
        Self { conv1 }
    }

    /// Forward pass of the model.
    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        self.conv1.forward(x)
    }
}

/// Model with extra fields to test loading of records (e.g. from a different model).
#[derive(Module, Debug)]
pub struct PartialWithExtraNet<B: Backend> {
    conv1: ConvBlock<B>,
    extra_field: bool, // This field is not present in the pytorch model
}

impl<B: Backend> PartialWithExtraNet<B> {
    /// Create a new model from the given record.
    pub fn new_with(record: PartialWithExtraNetRecord<B>) -> Self {
        let conv1 = ConvBlock {
            conv: Conv2dConfig::new([2, 4], [3, 2]).init_with(record.conv1.conv),
            norm: BatchNormConfig::new(2).init_with(record.conv1.norm),
        };
        Self {
            conv1,
            extra_field: true,
        }
    }

    /// Forward pass of the model.
    pub fn forward(&self, x: Tensor<B, 4>) -> Tensor<B, 4> {
        self.conv1.forward(x)
    }
}

type TestBackend = burn_ndarray::NdArray<f32>;

fn model_test(record: NetRecord<TestBackend>, precision: usize) {
    let device = Default::default();
    let model = Net::<TestBackend>::new_with(record);

    let input = Tensor::<TestBackend, 4>::ones([1, 2, 9, 6], &device) - 0.5;

    let output = model.forward(input);

    let expected = Tensor::<TestBackend, 2>::from_data(
        [[
            -2.306_613,
            -2.058_945_4,
            -2.298_372_7,
            -2.358_294,
            -2.296_395_5,
            -2.416_090_5,
            -2.107_669,
            -2.428_420_8,
            -2.526_469,
            -2.319_918_6,
        ]],
        &device,
    );

    output
        .to_data()
        .assert_approx_eq(&expected.to_data(), precision);
}

#[test]
fn full_record() {
    let device = Default::default();
    let record = PyTorchFileRecorder::<FullPrecisionSettings>::default()
        .load("tests/complex_nested/complex_nested.pt".into(), &device)
        .expect("Should decode state successfully");

    model_test(record, 8);
}

#[test]
fn half_record() {
    let device = Default::default();
    let record = PyTorchFileRecorder::<HalfPrecisionSettings>::default()
        .load("tests/complex_nested/complex_nested.pt".into(), &device)
        .expect("Should decode state successfully");

    model_test(record, 4);
}

#[test]
fn partial_model_loading() {
    // Load the full model but rename "conv_blocks.0.*" to "conv1.*"
    let load_args = LoadArgs::new("tests/complex_nested/complex_nested.pt".into())
        .with_key_remap("conv_blocks\\.0\\.(.*)", "conv1.$1");

    let device = Default::default();
    // Load the partial record from the full model
    let record = PyTorchFileRecorder::<FullPrecisionSettings>::default()
        .load(load_args, &device)
        .expect("Should decode state successfully");

    let model = PartialNet::<TestBackend>::new_with(record);

    let input = Tensor::<TestBackend, 4>::ones([1, 2, 9, 6], &device) - 0.5;

    let output = model.forward(input);

    // get the sum of all elements in the output tensor for quick check
    let sum = output.sum();

    assert_eq!(4.871538, sum.into_scalar());
}

#[test]
fn extra_field_model_loading() {
    // Load the full model but rename "conv_blocks.0.*" to "conv1.*"
    let load_args = LoadArgs::new("tests/complex_nested/complex_nested.pt".into())
        .with_key_remap("conv_blocks\\.0\\.(.*)", "conv1.$1");

    let device = Default::default();

    // Load the partial record from the full model
    let record = PyTorchFileRecorder::<FullPrecisionSettings>::default()
        .load(load_args, &device)
        .expect("Should decode state successfully");

    let model = PartialWithExtraNet::<TestBackend>::new_with(record);

    let input = Tensor::<TestBackend, 4>::ones([1, 2, 9, 6], &device) - 0.5;

    let output = model.forward(input);

    // get the sum of all elements in the output tensor for quick check
    let sum = output.sum();

    assert_eq!(4.871538, sum.into_scalar());

    assert!(model.extra_field);
}
