use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use serialport::SerialPort;
use std::{
    io::{self, Read, Write},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration, collections::HashMap,
};


enum ApplicationState {
    Command,
    Io
}

struct Command<Cmd: FnOnce() -> String> {
    character: char,
    command: Cmd
}

pub struct Application {
    port: Box<dyn SerialPort>,
    state: ApplicationState
}

impl Application {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        _ = terminal::enable_raw_mode();
        Self { port, state: ApplicationState::Io }
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
                },
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
        _ = thread::spawn(move || {
                let commands = HashMap::from(
                    [('q', || {println!("No quitters!")})]
                );
                let mut state = ApplicationState::Io;
                loop {
                    if let Event::Key(event) = event::read().expect("Failed to read line") {
                        match state {
                            ApplicationState::Io => {
                                match event {
                                    KeyEvent {
                                        code: KeyCode::Char('a'),
                                        modifiers: KeyModifiers::CONTROL,
                                        ..
                                    } => state = ApplicationState::Command,
                                    KeyEvent {
                                        code: KeyCode::Char(c),
                                        ..
                                    } => {
                                        tx.send(c).unwrap_or(());
                                    }
                                    _ => (),
                                }
                            },
                            ApplicationState::Command => {
                                match event {
                                    KeyEvent {
                                        code: KeyCode::Char('x'),
                                        modifiers: KeyModifiers::CONTROL,
                                        ..
                                    } => { state = ApplicationState::Io; break},
                                    KeyEvent {
                                        code: KeyCode::Char(code),
                                        modifiers: KeyModifiers::CONTROL,
                                        ..
                                    } => {
                                        match commands.get(&code) {
                                            Some(func) => func(),
                                            _ => ()
                                        }
                                        state = ApplicationState::Io;
                                    }
                                    _ => state = ApplicationState::Io
                                }
                            }
                        }
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
