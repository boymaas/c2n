use std::future::Future;

pub trait Storage: Future<Output = ()> {
    fn write(&mut self, data: String);
    fn read(&mut self) -> String;
}

