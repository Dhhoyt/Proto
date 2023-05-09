use std::{collections::HashMap, iter::zip};

use crate::{Model, IOType};

pub struct ConstantAmplifier(f32);

impl ConstantAmplifier {
    pub fn new(value: f32) -> Self {
        ConstantAmplifier(value)
    }
}

impl Model for ConstantAmplifier {
    fn input_format(&self) -> std::collections::HashMap<String, crate::IOType> {
        let mut input_format = HashMap::new();
        input_format.insert(String::from("Input"), IOType::Voltage);
        input_format
    }
    fn output_format(&self) -> std::collections::HashMap<String, crate::IOType> {
        let mut output_format = HashMap::new();
        output_format.insert(String::from("Output"), IOType::Voltage);
        output_format
    }
    fn evaluate(&mut self, _buffer_size: usize, inputs: crate::Input, outputs: &mut crate::Output) {
        let input = inputs.voltages.get("Input").unwrap();
        let output = outputs.voltages.get_mut("Output").unwrap();
        for (index, sample) in input.iter().enumerate() {
            output[index] = sample * self.0;
        }
    }
}

pub struct DuoSignalMixer(f32, f32);

impl DuoSignalMixer {
    fn new(input_1_mult: f32, input_2_mult: f32) -> Self {
        DuoSignalMixer(input_1_mult, input_2_mult)
    }
}

impl Model for DuoSignalMixer {
    fn input_format(&self) -> std::collections::HashMap<String, crate::IOType> {
        let mut input_format = HashMap::new();
        input_format.insert(String::from("Input1"), IOType::Voltage);
        input_format.insert(String::from("Input2"), IOType::Voltage);
        input_format
    }
    fn output_format(&self) -> std::collections::HashMap<String, crate::IOType> {
        let mut output_format = HashMap::new();
        output_format.insert(String::from("Output"), IOType::Voltage);
        output_format
    }
    fn evaluate(&mut self, _buffer_size: usize, inputs: crate::Input, outputs: &mut crate::Output) {
        let input_1 = inputs.voltages.get("Input1").unwrap();
        let input_2 = inputs.voltages.get("Input2").unwrap();
        let output = outputs.voltages.get_mut("Output").unwrap();
        for (index, sample) in zip(input_1.iter(), input_2.iter()).enumerate() {
            output[index] = (sample.0 * self.0) + (sample.1 * self.1);
        }
    }
}