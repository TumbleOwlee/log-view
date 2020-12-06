use crate::error::Error;
use std::io::BufRead;

pub struct Source<A: Send> {
    handle: Box<dyn TryRead<A> + 'static + Send>,
}

impl<A: Send> Source<A> {
    pub fn from(handle: impl TryRead<A> + 'static + Send) -> Source<A> {
        Source {
            handle: Box::new(handle),
        }
    }
}

impl<A: Send> TryRead<A> for Source<A> {
    fn try_read(&self) -> Option<A> {
        self.handle.try_read()
    }
}

pub trait TryRead<A> {
    fn try_read(&self) -> Option<A>;
}

pub struct AsyncPipeIn {
    handle: Option<std::thread::JoinHandle<()>>,
    terminate: std::sync::mpsc::Sender<()>,
    recv: std::sync::mpsc::Receiver<String>,
}

impl Drop for AsyncPipeIn {
    fn drop(&mut self) {
        self.terminate.send(()).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

impl AsyncPipeIn {
    pub fn start() -> Result<Self, Error> {
        if !atty::is(atty::Stream::Stdin) {
            let (tx, rx) = std::sync::mpsc::channel();
            let (ty, ry) = std::sync::mpsc::channel();
            Ok(AsyncPipeIn {
                handle: Some(std::thread::spawn(move || {
                    let stdin = std::io::stdin();
                    let mut stdin = stdin.lock();
                    let mut buf = String::with_capacity(1024);
                    loop {
                        if stdin.read_line(&mut buf).is_ok()
                            && !buf.is_empty()
                            && tx.send(buf.clone()).is_err()
                        {
                            break;
                        }
                        match ry.try_recv() {
                            Err(std::sync::mpsc::TryRecvError::Empty) => {}
                            _ => break,
                        }
                        buf.clear();
                    }
                })),
                terminate: ty,
                recv: rx,
            })
        } else {
            Err(Error::NoPipeIn)
        }
    }
}

impl TryRead<String> for AsyncPipeIn {
    fn try_read(&self) -> Option<String> {
        if let Ok(s) = self.recv.try_recv() {
            Some(s)
        } else {
            None
        }
    }
}

pub struct AsyncFileIn {
    handle: Option<std::thread::JoinHandle<()>>,
    terminate: std::sync::mpsc::Sender<()>,
    recv: std::sync::mpsc::Receiver<String>,
}

impl Drop for AsyncFileIn {
    fn drop(&mut self) {
        self.terminate.send(()).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

impl AsyncFileIn {
    pub fn start(file: &str) -> Result<Self, Error> {
        match std::fs::File::open(file)
            .map(std::io::BufReader::new)
            .map_err(|_| Error::FileOpenFailed(file.to_owned()))
        {
            Ok(mut f) => {
                let (tx, rx) = std::sync::mpsc::channel();
                let (ty, ry) = std::sync::mpsc::channel();
                Ok(AsyncFileIn {
                    handle: Some(std::thread::spawn(move || {
                        let mut buf = String::with_capacity(1024);
                        loop {
                            if f.read_line(&mut buf).is_ok()
                                && !buf.is_empty()
                                && tx.send(buf.clone()).is_err()
                            {
                                break;
                            }
                            match ry.try_recv() {
                                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                                _ => break,
                            }
                            buf.clear();
                        }
                    })),
                    terminate: ty,
                    recv: rx,
                })
            }
            Err(e) => Err(e),
        }
    }
}

impl TryRead<String> for AsyncFileIn {
    fn try_read(&self) -> Option<String> {
        if let Ok(s) = self.recv.try_recv() {
            Some(s)
        } else {
            None
        }
    }
}
