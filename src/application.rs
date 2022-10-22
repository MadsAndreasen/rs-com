use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use serialport::SerialPort;
use std::{
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

static BAUD_RATES: [u32; 6] = [9600, 19200, 33400, 57600, 115_200, 230_400];

pub struct Application {
    port: Box<dyn SerialPort>,
    state: Arc<RwLock<ApplicationState>>,
    baud_index: usize,
}

impl Application {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        _ = terminal::enable_raw_mode();
        Self {
            port,
            state: Arc::new(RwLock::new(ApplicationState::Io)),
            baud_index: 0,
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
                    let state = self.state.clone();
                    let mut lock = state.write().unwrap();
                    match *lock {
                        ApplicationState::Io => {
                            _ = self
                                .port
                                .write(key.to_string().as_bytes())
                                .expect("Write failed")
                        }
                        ApplicationState::Command => {
                            match key {
                                'q' => break,
                                'u' => self.baud_up(),
                                'd' => self.baud_down(),
                                _ => ()
                            }
                            *lock = ApplicationState::Io;

                        },
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }
            std::thread::sleep(Duration::from_millis(100))
        }
    }

    fn baud_down(&mut self)
    {
        self.baud_index = self.baud_index.checked_sub(1).unwrap_or(BAUD_RATES.len() - 1);
        self.set_baud_rate();
    }

    fn baud_up(&mut self) {
        self.baud_index += 1;
        self.set_baud_rate();

    }

    fn set_baud_rate(&mut self) {
        match BAUD_RATES.get(self.baud_index) {
            Some(baud_rate) => {
                print_info_line(&format!("Baud rate: {baud_rate}"));
                self.port.set_baud_rate(*baud_rate).unwrap();
            }
            None => {
                self.baud_index = 0;
                self.set_baud_rate();
            }
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

fn print_info_line(line: &str) {
    println!("\r");
    println!("*** {line} ***\r")
}

impl Drop for Application {
    fn drop(&mut self) {
        _ = terminal::disable_raw_mode();
    }
}
