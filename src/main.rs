use clap::{Parser, clap_derive::ArgEnum};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use serialport::SerialPort;
use std::{
    error::Error,
    io::{self, Read, Write},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};


#[derive(Debug, Clone, ArgEnum)]
enum ArgsFlowControl {
    Soft,
    Hard,
    None
}

#[derive(Debug, Clone, ArgEnum)]
enum ArgsParity {
    Even,
    Odd,
    None
}

/// Really Simple Communication application
/// Much like picocom.
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
    #[clap(short, long, arg_enum, default_value_t = ArgsFlowControl::None)]
    flowcontrol: ArgsFlowControl,

    /// Set parity
    #[clap(short, long, arg_enum, default_value_t = ArgsParity::None)]
    parity: ArgsParity,
}

struct Application {
    port: Box<dyn SerialPort>
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let flowcontrol = match args.flowcontrol {
        ArgsFlowControl::Soft => serialport::FlowControl::Software,
        ArgsFlowControl::Hard => serialport::FlowControl::Hardware,
        ArgsFlowControl::None => serialport::FlowControl::None
    };

    let parity = match args.parity {
        ArgsParity::Even => serialport::Parity::Even,
        ArgsParity::Odd => serialport::Parity::Odd,
        ArgsParity::None => serialport::Parity::None
    };

    let port = serialport::new(args.port, args.baudrate)
        .flow_control(flowcontrol)
        .parity(parity)
        .open()
        .expect("Failed to open");

    let mut app = Application::new(port);
    app.run();
    Ok(())
}



impl Application {

    fn new(port: Box<dyn SerialPort>) -> Self {
        _ = terminal::enable_raw_mode();
        Self {port}
    }


    fn run(&mut self) {
        let (tx, rx): (Sender<char>, Receiver<char>) = mpsc::channel();

        self.launch_input_reader(tx);

        self.launch_serial_comms(rx);

    }

    fn launch_serial_comms(&mut self, rx: Receiver<char>) {
        let mut serial_buf = vec![0; 1000];
        loop {
            match self.port.read(serial_buf.as_mut_slice()) {
                Ok(size) => io::stdout().write_all(&serial_buf[..size]).unwrap(),
                Err(ref error) if error.kind() == io::ErrorKind::TimedOut => (),
                Err(error) => eprintln!("Write Error {:?}", error),
            }
            match rx.try_recv() {
                Ok(key) => {
                    _ = self.port
                        .write(key.to_string().as_bytes())
                        .expect("Write failed")
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }
            std::thread::sleep(Duration::from_millis(100))
        }
    }

    fn launch_input_reader(&mut self, tx: Sender<char>) {
        _ = thread::spawn(move || loop {
            if let Event::Key(event) = event::read().expect("Failed to read line") {
                match event {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => break,
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => {
                        tx.send(c).unwrap_or(());
                    }
                    _ => (),
                }
            }
        });
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        _ = terminal::disable_raw_mode();
    }

}
