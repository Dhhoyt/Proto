use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use parking_lot::Mutex;
use rtrb::{Consumer, Producer, RingBuffer};
use std::{collections::HashMap, sync::Arc};

use crate::{Config, IOType, Input, Model, ModelHolder, Output};

pub struct AudioInput(Consumer<f32>, Stream);

impl AudioInput {
    pub fn new(stream_config: &cpal::StreamConfig) -> ModelHolder {
        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();
        let (mut producer, consumer) = RingBuffer::<f32>::new(16384);

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut output_fell_behind = false;
            for &sample in data {
                if producer.push(sample).is_err() {
                    output_fell_behind = true;
                }
            }
            if output_fell_behind {
                eprintln!("output stream fell behind: try increasing latency");
            }
        };
        let input_stream = input_device
            .build_input_stream(stream_config, input_data_fn, err_fn, None)
            .unwrap();
        input_stream.play().unwrap();
        Arc::new(Mutex::new(Box::new(AudioInput(consumer, input_stream))))
    }
}

impl Model for AudioInput {
    fn input_format(&self) -> HashMap<String, IOType> {
        HashMap::new()
    }
    fn output_format(&self) -> HashMap<String, IOType> {
        let mut outputs = HashMap::new();
        outputs.insert(String::from("Audio"), IOType::Voltage);
        outputs
    }
    fn evaluate(
        &mut self,
        buffer_size: usize,
        _inputs: Input,
        outputs: &mut Output,
        _config: &Config,
    ) {
        println!("doing stuff");
        let mut audio: Vec<f32> = Vec::with_capacity(buffer_size);
        for _ in 0..buffer_size {
            match self.0.pop() {
                Ok(value) => {
                    audio.push(value);
                }
                Err(_e) => {
                    audio.push(0.);
                }
            }
        }
        outputs.voltages.insert(String::from("Audio"), audio);
    }
}

unsafe impl Sync for AudioInput {}
unsafe impl Send for AudioInput {}

pub struct AudioOutput(Producer<f32>);

impl AudioOutput {
    pub fn new() -> (Self, rtrb::Consumer<f32>) {
        let (mut producer, consumer) = RingBuffer::<f32>::new(16384);
        for _ in 0..1024 {
            producer.push(0.).unwrap();
        }
        (AudioOutput(producer), consumer)
    }
}

impl Model for AudioOutput {
    fn input_format(&self) -> HashMap<String, IOType> {
        let mut inputs = HashMap::new();
        inputs.insert(String::from("Audio"), IOType::Voltage);
        inputs
    }
    fn output_format(&self) -> HashMap<String, IOType> {
        HashMap::new()
    }
    fn evaluate(
        &mut self,
        _buffer_size: usize,
        inputs: Input,
        _outputs: &mut Output,
        _config: &Config,
    ) {
        let mut output_fell_behind = false;
        for i in inputs.voltages.get("Audio").unwrap().iter() {
            match self.0.push(*i) {
                Ok(_) => (),
                Err(_) => output_fell_behind = true,
            };
        }
        if output_fell_behind {
            println!("output device fell behind: try increasing latency");
        }
        println!("{}", inputs.voltages.get("Audio").unwrap().len());
    }
}

unsafe impl Sync for AudioOutput {}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}
