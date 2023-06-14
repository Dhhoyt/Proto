use std::{collections::HashMap, iter::zip};

use crate::{Config, IOType, Model};

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
    fn evaluate(
        &mut self,
        _buffer_size: usize,
        inputs: crate::Input,
        outputs: &mut crate::Output,
        _config: &Config,
    ) {
        let input = inputs.voltages.get("Input").unwrap();
        let output = outputs.voltages.get_mut("Output").unwrap();
        for (index, sample) in input.iter().enumerate() {
            output[index] = sample * self.0;
        }
    }
}

pub struct DuoSignalMixer(f32, f32);

impl DuoSignalMixer {
    pub fn new(input_1_mult: f32, input_2_mult: f32) -> Self {
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
    fn evaluate(
        &mut self,
        _buffer_size: usize,
        inputs: crate::Input,
        outputs: &mut crate::Output,
        _config: &Config,
    ) {
        let input_1 = inputs.voltages.get("Input1").unwrap();
        let input_2 = inputs.voltages.get("Input2").unwrap();
        let output = outputs.voltages.get_mut("Output").unwrap();
        for (index, sample) in zip(input_1.iter(), input_2.iter()).enumerate() {
            output[index] = (sample.0 * self.0) + (sample.1 * self.1);
        }
    }
}

pub struct Vca;

impl Model for Vca {
    fn input_format(&self) -> std::collections::HashMap<String, crate::IOType> {
        let mut input_format = HashMap::new();
        input_format.insert(String::from("Input"), IOType::Voltage);
        input_format.insert(String::from("Control"), IOType::Voltage);
        input_format
    }
    fn output_format(&self) -> std::collections::HashMap<String, crate::IOType> {
        let mut output_format = HashMap::new();
        output_format.insert(String::from("Output"), IOType::Voltage);
        output_format
    }
    fn evaluate(
        &mut self,
        _buffer_size: usize,
        inputs: crate::Input,
        outputs: &mut crate::Output,
        _config: &Config,
    ) {
        let input = inputs.voltages.get("Input").unwrap();
        let control = inputs.voltages.get("Control").unwrap();
        let output = outputs.voltages.get_mut("Output").unwrap();
        for (index, sample) in zip(input.iter(), control.iter()).enumerate() {
            output[index] = sample.0 * sample.1;
        }
    }
}

pub struct Tone {
    resistor_one_value: f32,
    resistor_two_value: f32,
    inductor_value: f32,
    capicitor_value: f32,
    current: f32,
    capicitor_voltage: f32,
}

impl Tone {
    pub fn new(resistor_value: f32) -> Self {
        Tone {
            capicitor_value: 1.5e-8,
            resistor_one_value: 100_000.,
            resistor_two_value: resistor_value,
            inductor_value: 2.,
            current: 0.,
            capicitor_voltage: 0.,
        }
    }
}

impl Model for Tone {
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

    fn evaluate(
        &mut self,
        _buffer_size: usize,
        inputs: crate::Input,
        outputs: &mut crate::Output,
        config: &Config,
    ) {
        let inputs = *inputs.voltages.get("Input").unwrap();
        let outputs = outputs.voltages.get_mut("Output").unwrap();
        for (index, sample) in inputs.iter().enumerate() {
            //This is silly circuit simulation.
            //It's an eulerian approximation of a voltage source to a resistor to an inductor to another resistor to a capacitor to ground
            //The voltage out is between the inductor and second resistor
            self.current += (((sample - self.current * self.resistor_one_value)
                - (self.current * self.resistor_two_value + self.capicitor_voltage))
                / self.inductor_value)
                * config.delta;
            self.capicitor_value += (self.current/self.capicitor_voltage) * config.delta;
            outputs[index] = self.current * self.resistor_two_value + self.capicitor_voltage;
        }
    }
}
