#![allow(dead_code)]

use std::{collections::{HashMap, HashSet}, vec};

fn main() {
    
}


struct Graph {
    components: HashMap<usize, Component>,
    next_free_id: usize
}

impl Graph {
    fn new() -> Self {
        Graph { components: HashMap::new(), next_free_id: 0 }
    }

    fn process(&mut self, buffer_size: usize) {
        for controller in self.components.values_mut() {
            controller.reset_inputs(buffer_size);
        }
    }

    fn add_component(&mut self, model: Box<dyn Model>) -> usize{
        self.components.insert(self.next_free_id, Component::new(model));
        self.next_free_id += 1;
        self.next_free_id - 1
    }

    fn add_connection(&mut self, new_connection: Connection) -> Result<(), ConnectionError>{
        if new_connection.from.id == new_connection.to.id {
            return Result::Err(ConnectionError::LoopingConnection);
        }
        let from = self.components.get_mut(&new_connection.from.id);
        let from = match from { 
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph)
        };
        from.out_connections.insert(new_connection.clone());
        let to = self.components.get_mut(&new_connection.from.id);
        let to = match to { 
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph)
        };
        to.in_connections.insert(new_connection.clone());
        if self.loooping_graph(new_connection.to.id) {
            #[allow(unused_must_use)] {
                self.remove_connection(new_connection);
            }
            return Result::Err(ConnectionError::LoopingConnection);
        }

        Result::Ok(())
    }

    fn remove_connection(&mut self, old_connection: Connection) -> Result<(), ConnectionError>{
        let from = self.components.get_mut(&old_connection.from.id);
        let from = match from { 
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph)
        };
        from.out_connections.remove(&old_connection);
        let to = self.components.get_mut(&old_connection.from.id);
        let to = match to { 
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph)
        };
        to.in_connections.remove(&old_connection);
        Result::Ok(())
    }

    fn loooping_graph(&self, start: usize) -> bool {
        //Simple depth first search
        let mut stack: Vec<usize> = Vec::new();
        let mut visited: HashSet<usize> = HashSet::new();
        stack.push(start);
        visited.insert(start);
        while stack.len() > 0 {
            let current: usize = stack.pop().unwrap();
            let current: &Component = self.components.get(&current).unwrap();
            for i in current.out_connections.clone().into_iter() {
                if i.to.id == start {
                    return true;
                }
                if visited.insert(i.to.id) {
                    stack.push(i.to.id);
                }
            }
        }
        false
    }
}

enum ConnectionError {
    LoopingConnection,
    ControllerNotInGraph
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct Node {
    id: usize,
    io: IOType,
    name: String
}


#[derive(Clone, Hash, Eq, PartialEq)]
struct Connection {
    from: Node,
    to: Node
}

struct Component {
    model: Box<dyn Model>,
    inputs: IO,
    outputs: IO,
    input_format: HashMap<String, IOType>,
    output_format: HashMap<String, IOType>,
    in_connections: HashSet<Connection>,
    out_connections: HashSet<Connection>
}

impl Component {
    fn new(model: Box<dyn Model>) -> Self{
        Component { 
            inputs: IO::default(), 
            outputs: IO::default(), 
            input_format: model.input_format(),
            output_format: model.output_format(),
            in_connections: HashSet::new(),
            out_connections: HashSet::new(),
            model: model,
        }
    }

    fn reset_inputs(&mut self, buffer_size: usize) {
        let request = self.model.input_format();

        let mut new_input = IO::default();

        for (name, io_type) in request {
            match io_type {
                IOType::Voltage => {new_input.voltages.insert(name, vec![0.;buffer_size]);},
                IOType::Midi => {new_input.midi.insert(name, Vec::new());}
            }
        }

        self.inputs = new_input;
    }
}

#[derive(Default)]
struct MidiEvent {

}

#[derive(Default)]
struct IO {
    voltages: HashMap<String, Vec<f32>>,
    midi: HashMap<String, Vec<MidiEvent>>
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
enum IOType {
    Voltage,
    Midi
}

trait Model {
    fn process(&mut self, inputs: &IO, outputs: &mut IO);

    fn input_format(&self) -> HashMap<String, IOType>;

    fn output_format(&self) -> HashMap<String, IOType>;
}