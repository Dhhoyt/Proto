use std::{
    collections::{HashMap, HashSet},
    vec,
};

fn main() {}

struct Graph {
    components: HashMap<usize, Component>,
    next_free_id: usize,
    evaluation_order: Vec<usize>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            components: HashMap::new(),
            next_free_id: 0,
            evaluation_order: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, buffer_size: usize) {
        for i in self.components.values_mut() {
            i.reset_inputs(buffer_size);
            i.reset_outputs(buffer_size);
        }
        for i in self.evaluation_order.clone().iter() {
            let current_compoent = self.components.get_mut(i).unwrap();
            current_compoent.evaluate();
            for connection in current_compoent.out_connections.clone().iter() {
                self.copy_buffers(connection);
            }
        }
    }

    fn copy_buffers(&mut self, connection: &Connection) {
        match connection.from.io {
            IOType::Voltage => {
                let from = self.components.get(&connection.from.id).unwrap();
                let source = from
                    .outputs
                    .voltages
                    .get(&connection.to.name)
                    .unwrap()
                    .clone();
                let to = self.components.get_mut(&connection.to.id).unwrap();
                let dest = to.inputs.voltages.get_mut(&connection.to.name).unwrap();
                for (a, b) in dest.iter_mut().zip(source) {
                    *a += b;
                }
            }
            IOType::Midi => {}
        }
    }

    fn process(&mut self, buffer_size: usize) {
        for controller in self.components.values_mut() {
            controller.reset_inputs(buffer_size);
        }
    }

    pub fn add_component(&mut self, model: Box<dyn Model>) -> usize {
        self.components
            .insert(self.next_free_id, Component::new(self.next_free_id, model));
        self.next_free_id += 1;
        self.next_free_id - 1
    }

    pub fn set_model(
        &mut self,
        target: usize,
        new_model: Box<dyn Model>,
    ) -> Result<(), ConnectionError> {
        match self.components.get_mut(&target) {
            Some(c) => {
                c.model = new_model;
                return Ok(());
            }
            None => Err(ConnectionError::ControllerNotInGraph),
        }
    }

    pub fn remove_component(&mut self, to_remove: usize) -> bool {
        let to_remove = self.components.get(&to_remove);
        let to_remove = match to_remove {
            Some(v) => (v.in_connections.clone(), v.out_connections.clone()),
            None => return false,
        };
        for c in to_remove.0 {
            self.remove_connection(c).unwrap();
        }
        for c in to_remove.1 {
            self.remove_connection(c).unwrap();
        }
        true
    }

    pub fn add_connection(&mut self, new_connection: Connection) -> Result<(), ConnectionError> {
        if new_connection.from.id == new_connection.to.id {
            return Result::Err(ConnectionError::LoopingConnection);
        }
        let from = self.components.get_mut(&new_connection.from.id);
        let from: &mut Component = match from {
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph),
        };
        from.out_connections.insert(new_connection.clone());
        let to = self.components.get_mut(&new_connection.from.id);
        let to: &mut Component = match to {
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph),
        };
        to.in_connections.insert(new_connection.clone());
        let ord = self.sort();
        match ord {
            Err(e) => {
                self.remove_connection(new_connection).unwrap();
                Err(e)
            }
            Ok(l) => {
                self.evaluation_order = l;
                Ok(())
            }
        }
    }

    fn connections(&self) -> HashSet<Connection> {
        let mut connections: HashSet<Connection> = HashSet::new();
        for (_, component) in &self.components {
            connections.extend(component.out_connections.clone());
        }
        return connections;
    }

    pub fn remove_connection(
        &mut self,
        old_connection: Connection,
    ) -> Result<bool, ConnectionError> {
        let from = self.components.get_mut(&old_connection.from.id);
        let from = match from {
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph),
        };
        from.out_connections.remove(&old_connection);
        let to = self.components.get_mut(&old_connection.from.id);
        let to = match to {
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph),
        };
        Ok(to.in_connections.remove(&old_connection))
    }

    fn sort(&self) -> Result<Vec<usize>, ConnectionError> {
        //https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm; Comments are from the pseudocode
        let mut connections: Vec<(usize, usize)> = self
            .connections()
            .into_iter()
            .map(|c| (c.from.id, c.to.id))
            .collect();
        //S ← Set of all nodes with no incoming edge
        let mut s: Vec<usize> = self
            .components
            .iter()
            .filter(|x| x.1.in_connections.len() == 0)
            .map(|x| x.0.clone())
            .collect();
        //L ← Empty list that will contain the sorted elements
        let mut l: Vec<usize> = Vec::new();

        //while S is not empty do
        while s.len() > 0 {
            //remove a node n from S
            let n = s.pop().unwrap();
            //add n to L
            l.push(n);
            //for each node m with an edge e from n to m do
            for m in connections.clone().iter().filter(|x| x.0 == n).map(|x| x.1) {
                //remove edge e from the graph
                connections.retain(|x| x.0 != n || x.1 != m);
                //if m has no other incoming edges then
                if !connections.iter().any(|x| x.1 == m) {
                    s.push(m);
                }
            }
        }
        //if graph has edges then
        if !connections.is_empty() {
            //return error   (graph has at least one cycle)
            return Err(ConnectionError::LoopingConnection);
        }
        //else
        //return L   (a topologically sorted order)
        Ok(l)
    }
}

#[derive(Debug)]
enum ConnectionError {
    LoopingConnection,
    ControllerNotInGraph,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct Node {
    id: usize,
    io: IOType,
    name: String,
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct Connection {
    from: Node,
    to: Node,
}

struct Component {
    id: usize,
    model: Box<dyn Model>,
    inputs: IO,
    outputs: IO,
    in_connections: HashSet<Connection>,
    out_connections: HashSet<Connection>,
}

impl Component {
    fn new(id: usize, model: Box<dyn Model>) -> Self {
        Component {
            id: id,
            inputs: IO::default(),
            outputs: IO::default(),
            in_connections: HashSet::new(),
            out_connections: HashSet::new(),
            model: model,
        }
    }

    fn evaluate(&mut self) {
        self.model.evaluate(&mut self.inputs, &mut self.outputs);
    }

    fn reset_inputs(&mut self, buffer_size: usize) {
        let request = self.model.input_format();

        let mut new_inputs = IO::default();

        for (name, io_type) in request {
            match io_type {
                IOType::Voltage => {
                    new_inputs.voltages.insert(name, vec![0.; buffer_size]);
                }
                IOType::Midi => {
                    new_inputs.midi.insert(name, Vec::new());
                }
            }
        }

        self.inputs = new_inputs;
    }

    fn reset_outputs(&mut self, buffer_size: usize) {
        let request = self.model.output_format();

        let mut new_outputs = IO::default();

        for (name, io_type) in request {
            match io_type {
                IOType::Voltage => {
                    new_outputs.voltages.insert(name, vec![0.; buffer_size]);
                }
                IOType::Midi => {
                    new_outputs.midi.insert(name, Vec::new());
                }
            }
        }

        self.outputs = new_outputs;
    }
}

#[derive(Default)]
struct MidiEvent {}

#[derive(Default)]
struct IO {
    voltages: HashMap<String, Vec<f32>>,
    midi: HashMap<String, Vec<MidiEvent>>,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
enum IOType {
    Voltage,
    Midi,
}

//Gotta change this name
enum DataTypes {
    Voltage(f32),
}

trait Model {
    fn evaluate(&mut self, inputs: &IO, outputs: &mut IO);

    fn input_format(&self) -> HashMap<String, IOType>;

    fn output_format(&self) -> HashMap<String, IOType>;

    fn pass_data(&self, data: DataTypes);
}
