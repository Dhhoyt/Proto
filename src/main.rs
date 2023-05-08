use std::{sync::Arc, ops::{DerefMut}};

use cpal::{traits::{HostTrait, DeviceTrait, StreamTrait}, SampleRate};
use graph::{Connection, Node};
use models::{AudioInput, AudioOutput};
use parking_lot::Mutex;

mod graph;
mod models;

const STREAMCONFIG: cpal::StreamConfig = cpal::StreamConfig { channels: 2, sample_rate: SampleRate(48000), buffer_size: cpal::BufferSize::Fixed(64)};

fn main() {
    let mut graph = graph::Graph::new(64);
    let input = AudioInput::new();
    let from = graph.add_model(Box::new(input));
    let (output, mut consumer) = AudioOutput::new();
    let to = graph.add_model(Box::new(output));
    println!("to: {}, from: {}", to, from);
    let con = Connection {
        from: Node {
            id: from,
            io: graph::IOType::Voltage,
            name: String::from("Audio")
        },
        to: Node {
            id: to,
            io: graph::IOType::Voltage,
            name: String::from("Audio")
        },
    };
    graph.add_connection(con).unwrap();
    let graph = Arc::new(Mutex::new(graph));
    let output_reference = Arc::clone(&graph);
    let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut input_fell_behind = false;
        for sample in data {
            *sample = match consumer.pop() {
                Ok(s) => s,
                Err(_e) => {
                    input_fell_behind = true;
                    0.0
                }
            };
        }
        let output_reference = output_reference.to_owned();
        let mut output_reference = output_reference.lock();
        let graph = output_reference.deref_mut();
        graph.evaluate();
        
        if input_fell_behind {
            eprintln!("input stream fell behind: try increasing latency");
        }
    };
    let output_device = cpal::default_host().default_output_device().unwrap();
    let output_stream = output_device.build_output_stream(&STREAMCONFIG, output_data_fn, err_fn, None).unwrap();
    output_stream.play().unwrap();
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}