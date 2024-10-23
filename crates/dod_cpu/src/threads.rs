use flume::{Receiver, Sender};
use indicatif::{MultiProgress, ProgressStyle};
use std::thread;

pub fn get_multi_progress<T>() -> (MultiProgress, ProgressStyle, Sender<T>, Receiver<T>) {
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    let (tx, rx) = flume::unbounded::<T>();

    (m, sty, tx, rx)
}

pub fn get_available_threads() -> u32 {
    if thread::available_parallelism().unwrap().get() == 1 {
        1u32
    } else {
        thread::available_parallelism().unwrap().get() as u32
    }
}

pub fn get_single_progreses<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = flume::unbounded::<T>();
    (tx, rx)
}
