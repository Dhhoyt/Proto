use std::collections::HashMap;

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