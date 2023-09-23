use proto::{
    audio_io::{AudioInput, AudioOutput},
    model_utils::Tone,
    Connection, Engine, IOType, OutputDevice, Port,
};
use std::{thread, time};

fn main() {
    let output = OutputDevice::default().unwrap();
    println!("using {} for output", output.name);
    let (mut engine, _) = Engine::new(&output, 256, 48000);
    let input = AudioInput::new(&engine.stream_config);
    let input_id = engine.add_model(input.clone());
    let con = Connection {
        from: Port {
            id: input_id,
            io: IOType::Voltage,
            name: String::from("Audio"),
        },
        to: Port {
            id: 0,
            io: IOType::Voltage,
            name: String::from("Audio"),
        },
    };
    engine.add_connection(con).unwrap();

    let ten_millis = time::Duration::from_secs(10);
    let now = time::Instant::now();

    thread::sleep(ten_millis);
}
