mod application;

use application::Application;

use clap::{ValueEnum, Parser};
use std::{error::Error};

use serde::{Deserialize};
use serde_yaml::{self};

#[derive(Debug, Clone, ValueEnum)]
enum ArgsFlowControl {
    Soft,
    Hard,
    None,
}

#[derive(Debug, Clone, ValueEnum)]
enum ArgsParity {
    Even,
    Odd,
    None,
}

#[derive(Debug, Deserialize)]
pub struct Macro {
    name: String,
    content: String
}

#[derive(Debug, Deserialize)]
struct Macros {
    macros: Vec<Macro>
}

/// Really Simple Communication application
/// Much like picocom.
/// Press C-a C-h for help with commands
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to device to use (i.e. /dev/ttyS1, COM1)
    #[clap()]
    port: String,

    /// Baudrate to use with the serial device
    #[clap(short, long, default_value_t = 115_200)]
    baudrate: u32,

    /// Set flow control
    #[clap(short, long, value_enum, default_value_t = ArgsFlowControl::None)]
    flowcontrol: ArgsFlowControl,

    /// Set parity
    #[clap(short, long, value_enum, default_value_t = ArgsParity::None)]
    parity: ArgsParity,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let flowcontrol = match args.flowcontrol {
        ArgsFlowControl::Soft => serialport::FlowControl::Software,
        ArgsFlowControl::Hard => serialport::FlowControl::Hardware,
        ArgsFlowControl::None => serialport::FlowControl::None,
    };

    let parity = match args.parity {
        ArgsParity::Even => serialport::Parity::Even,
        ArgsParity::Odd => serialport::Parity::Odd,
        ArgsParity::None => serialport::Parity::None,
    };

    let port = serialport::new(args.port, args.baudrate)
        .flow_control(flowcontrol)
        .parity(parity)
        .open()
        .expect("Failed to open");

    let macros = load_macrosj();
    let mut app = Application::new(port, macros);
    app.run();
    Ok(())
}


fn load_macrosj() -> Vec<Macro>
{
    if let Ok(f) = std::fs::File::open("macros.rscom") {
        let macros: Macros = serde_yaml::from_reader(f).expect("Macros file could not be parsed");
        macros.macros
    }
    else {
        Vec::<Macro>::new()
    }

}