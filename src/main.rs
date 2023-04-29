use std::{sync::{mpsc::{Sender, Receiver, self}}, thread};
use nannou::{prelude::*} ;

mod audio;
use crate::audio::{record, process_audio};
mod scene;
use crate::scene::*;

const BUFFER_WINDOW: usize = 50;
struct Model {
    buffer: Receiver<Vec<f32>>,
    particle: Particle,
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
        buffer: video_rx,
        particle: Particle::NONE,
    }
}

fn update(app: &App, _model: &mut Model, _update: Update){
    if app.time as usize % 15 == 0 {
        _model.particle = particle_selection();
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let recv_buffer = &_model.buffer.recv().unwrap();

    let fft_array = &recv_buffer[BUFFER_WINDOW..recv_buffer.len() - BUFFER_WINDOW];
    let scene = Scene {
        app,
        draw: app.draw(),
        frame,
        sensitivity: Sensitivity::HIGH, 
        particle: _model.particle,
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