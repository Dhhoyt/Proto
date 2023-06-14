use std::collections::{HashMap, HashSet};

use crate::{Config, Connection, ConnectionError, IOType, Input, Model, Output};

pub struct Graph {
    buffer_size: usize,
    components: HashMap<usize, Component>,
    evaluation_order: Vec<usize>,
    next_free_id: usize,
}

impl Graph {
    pub fn new(buffer_size: usize) -> Self {
        Graph {
            buffer_size: buffer_size,
            components: HashMap::new(),
            evaluation_order: Vec::new(),
            next_free_id: 0,
        }
    }

    pub fn evaluate(&mut self, sample_rate: usize) {
        let default_voltage = vec![0.; self.buffer_size];
        let mut outputs: HashMap<usize, Output> = HashMap::new();
        let config = Config {
            buffer_size: self.buffer_size,
            delta: 1.0 / sample_rate as f32,
        };
        for id in self.evaluation_order.iter() {
            let component = self.components.get_mut(&id).unwrap();
            let mut output = component.construct_outputs(self.buffer_size);
            let mut input = Input::default();
            //Get references to all the inputs with a connection
            for i in component.in_connections.iter() {
                match i.from.io {
                    IOType::Voltage => {
                        let value = outputs
                            .get(&i.from.id)
                            .unwrap()
                            .voltages
                            .get(&i.from.name)
                            .unwrap();
                        input.voltages.insert(i.to.name.clone(), value);
                    }
                }
            }
            //Fill in defaults for all the inputs without a connection
            for i in component.model.input_format() {
                match i.1 {
                    IOType::Voltage => {
                        if !input.voltages.contains_key(&i.0) {
                            input.voltages.insert(i.0.clone(), &default_voltage);
                        }
                    }
                }
            }
            component
                .model
                .evaluate(self.buffer_size, input, &mut output, &config);
            outputs.insert(*id, output);
        }
    }

    pub fn add_model(&mut self, model: Box<dyn Model + Send + Sync>) -> usize {
        self.components.insert(
            self.next_free_id,
            Component {
                model: model,
                in_connections: HashSet::new(),
                out_connections: HashSet::new(),
            },
        );
        self.next_free_id += 1;
        //There is no situation in which adding a new unconnected model will cause the topo sort to return an error
        self.evaluation_order = self.sort().unwrap();
        self.next_free_id - 1
    }

    pub fn connections(&self) -> HashSet<Connection> {
        let mut connections: HashSet<Connection> = HashSet::new();
        for (_, component) in &self.components {
            connections.extend(component.out_connections.clone());
        }
        return connections;
    }

    pub fn add_connection(&mut self, new_connection: Connection) -> Result<(), ConnectionError> {
        //Check for self connection
        if new_connection.from.id == new_connection.to.id {
            return Result::Err(ConnectionError::LoopingConnection);
        }
        //Get node outputing to and verify that nothing is already connected to it
        let to = self.components.get_mut(&new_connection.to.id);
        let to: &mut Component = match to {
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph),
        };
        match to.model.input_format().get(&new_connection.to.name) {
            None => return Err(ConnectionError::InputNotInComponent),
            Some(t) => {
                if *t != new_connection.to.io {
                    return Err(ConnectionError::InputNotInComponent);
                }
            }
        }
        if to
            .in_connections
            .iter()
            .any(|c| c.to.name == new_connection.to.name)
        {
            return Err(ConnectionError::InputOccupied);
        }

        //Run checks on where its coming from
        let from = self.components.get_mut(&new_connection.from.id);
        let from: &mut Component = match from {
            Some(c) => c,
            None => return Result::Err(ConnectionError::ControllerNotInGraph),
        };
        match from.model.output_format().get(&new_connection.from.name) {
            None => return Err(ConnectionError::OutputNotInComponent),
            Some(t) => {
                if *t != new_connection.from.io {
                    return Err(ConnectionError::OutputNotInComponent);
                }
            }
        }

        //Add the connections now that the guard statements have been passed
        let from = self.components.get_mut(&new_connection.from.id).unwrap();
        from.out_connections.insert(new_connection.clone());

        let to = self.components.get_mut(&new_connection.to.id).unwrap();
        to.in_connections.insert(new_connection.clone());

        //Update the processing order and undo the connection if it creates a loop
        let order = self.sort();
        match order {
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

    fn sort(&mut self) -> Result<Vec<usize>, ConnectionError> {
        //https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm
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

    pub fn remove_model(&mut self, id: usize) -> bool {
        let c = match self.components.get(&id) {
            Some(c) => c,
            None => return false,
        };
        let in_connections = c.in_connections.clone().into_iter();
        let out_connections = c.out_connections.clone().into_iter();
        self.components.remove(&id);
        for i in in_connections {
            self.components
                .get_mut(&i.from.id)
                .unwrap()
                .out_connections
                .remove(&i);
        }
        for i in out_connections {
            self.components
                .get_mut(&i.to.id)
                .unwrap()
                .in_connections
                .remove(&i);
        }
        true
    }
}

struct Component {
    pub model: Box<dyn Model + Send + Sync>,
    in_connections: HashSet<Connection>,
    out_connections: HashSet<Connection>,
}

impl Component {
    fn construct_outputs(&self, buffer_size: usize) -> Output {
        let request = self.model.output_format();

        let mut new_outputs = Output::default();

        for (name, io_type) in request {
            match io_type {
                IOType::Voltage => {
                    new_outputs.voltages.insert(name, vec![0.; buffer_size]);
                }
            }
        }

        new_outputs
    }
}
