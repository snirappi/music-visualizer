use std::{sync::{mpsc::{Sender, Receiver, self}}, thread};
use nannou::{prelude::*} ;

mod audio;
use crate::audio::{record, process_audio};
mod scene;
use crate::scene::*;
struct Model {
    buffer: Receiver<Vec<f32>>
}

fn model(_app: &App) -> Model {
    let (audio_tx, audio_rx): (Sender<f32>, Receiver<f32>) = mpsc::channel();
    let (video_tx, video_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = mpsc::channel();
    thread::spawn(move || {
        record(audio_tx);
    });
    thread::spawn(move || {
        process_audio(audio_rx, video_tx);
    });
    Model {
        buffer: video_rx
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update){

}

fn view(app: &App, _model: &Model, frame: Frame) {
    let fft_array = &_model.buffer.recv().unwrap()[256..768];
    let scene = Scene {
        app,
        draw: app.draw(),
        frame,
        sensitivity: Sensitivity::HIGH, 
    };
    scene.run(fft_array.to_vec());
}

pub fn main() {
        nannou::app(model)
            .update(update) 
            .simple_window(view)
            .fullscreen()
            .run();
}