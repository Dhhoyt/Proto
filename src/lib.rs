use std::{sync::Arc, ops::{DerefMut}, collections::{HashSet, HashMap}};
use cpal::{traits::{DeviceTrait, StreamTrait}, SampleRate, StreamConfig, Stream};
use graph::Graph;
use audio_io::{AudioOutput};
use parking_lot::Mutex;

mod graph;

pub mod audio_io;
pub mod model_utils;

pub struct Engine {
    pub stream_config: StreamConfig,
    graph: Arc<Mutex<Graph>>,
    #[allow(dead_code)]
    output_stream: Stream,
}

impl Engine {
    pub fn new(output_device: cpal::Device, buffer_size: usize, sample_rate: usize) -> Self {
        let config = cpal::StreamConfig { channels: 1, sample_rate: SampleRate(sample_rate as u32), buffer_size: cpal::BufferSize::Fixed(buffer_size as u32)};
        let mut graph = Graph::new(buffer_size);
        let (output, mut consumer) = AudioOutput::new();
        graph.add_model(Box::new(output));
        let graph = Arc::new(Mutex::new(graph));
        let output_reference = Arc::clone(&graph);
        //The actual code that outputs and runs the graph. This function runs once for every buffer the device requests.
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
            //Evaluate the graph here so its synced with the 
            let mut output_reference = output_reference.lock();
            let graph = output_reference.deref_mut();
            graph.evaluate();
            
            if input_fell_behind {
                eprintln!("input stream fell behind: try increasing latency");
            }
        };
        let output_stream = output_device.build_output_stream(&config, output_data_fn, err_fn, None).unwrap();
        output_stream.play().unwrap();
        Engine { graph: graph, stream_config: config, output_stream: output_stream }
    }

    pub fn add_model(&mut self, model: Box<dyn Model + Send + Sync>) -> usize {
        self.graph.lock().add_model(model)
    }
    
    pub fn connections(&self) -> HashSet<Connection> {
        self.graph.lock().connections()
    }

    pub fn add_connection(&mut self, new_connection: Connection) -> Result<(), ConnectionError> {
        self.graph.lock().add_connection(new_connection)
    }
}

pub trait Model {
    fn output_format(&self) -> HashMap<String, IOType>;

    fn input_format(&self) -> HashMap<String, IOType>;

    fn evaluate(&mut self, buffer_size: usize, inputs: Input, outputs: &mut Output);
}

#[derive(Default)]
pub struct Input<'a> {
    pub voltages: HashMap<String, &'a Vec<f32>>,
}

#[derive(Default)]
pub struct Output {
    pub voltages: HashMap<String, Vec<f32>>,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum IOType {
    Voltage,
}

#[derive(Debug)]
pub enum ConnectionError {
    LoopingConnection,
    ControllerNotInGraph,
    InputOccupied,
    MismatchedConnectionTypes,
    InputNotInComponent,
    OutputNotInComponent,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Connection {
    pub from: Node,
    pub to: Node,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Node {
    pub id: usize,
    pub io: IOType,
    pub name: String,
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}