use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use serialport::SerialPort;
use std::{
    io::{self, Read, Write},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

pub struct Application {
    port: Box<dyn SerialPort>,
}

impl Application {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        _ = terminal::enable_raw_mode();
        Self { port }
    }

    pub fn run(&mut self) {
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
                    _ = self
                        .port
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
