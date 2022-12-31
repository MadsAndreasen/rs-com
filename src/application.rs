use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use serialport::SerialPort;
use std::{
    collections::HashMap,
    io::{self, Read, Write},
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc, RwLock,
    },
    thread,
    time::Duration,
};

enum ApplicationState {
    Command,
    Io,
}

struct Command<Cmd: FnOnce() -> String> {
    character: char,
    command: Cmd,
}

pub struct Application {
    port: Box<dyn SerialPort>,
    state: Arc<RwLock<ApplicationState>>,
}

impl Application {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        _ = terminal::enable_raw_mode();
        Self {
            port,
            state: Arc::new(RwLock::new(ApplicationState::Io)),
        }
    }

    pub fn run(&mut self) {
        let (tx, rx): (Sender<char>, Receiver<char>) = mpsc::channel();
        self.launch_input_reader(tx);
        self.launch_serial_comms(rx);
    }

    fn launch_serial_comms(&mut self, rx: Receiver<char>) {
        let mut serial_buf = vec![0; 100];
        loop {
            match self.port.read(serial_buf.as_mut_slice()) {
                Ok(size) => {
                    io::stdout().write_all(&serial_buf[..size]).unwrap();
                    io::stdout().flush().unwrap();
                }
                Err(ref error) if error.kind() == io::ErrorKind::TimedOut => (),
                Err(error) => eprintln!("Write Error {:?}", error),
            }
            match rx.try_recv() {
                Ok(key) => {
                    let mut lock = self.state.write().unwrap();
                    match *lock {
                        ApplicationState::Io => {
                            _ = self
                                .port
                                .write(key.to_string().as_bytes())
                                .expect("Write failed")
                        }
                        ApplicationState::Command => {
                            *lock = ApplicationState::Io;
                            println!("Quitters")
                        },
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }
            std::thread::sleep(Duration::from_millis(100))
        }
    }

    fn launch_input_reader(&mut self, tx: Sender<char>) {
        let state: Arc<RwLock<ApplicationState>> = Arc::clone(&self.state);
        _ = thread::spawn(move || loop {
            if let Event::Key(event) = event::read().expect("Failed to read line") {
                match event {
                    KeyEvent {
                        code: KeyCode::Char('a'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => *state.write().unwrap() = ApplicationState::Command,
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
