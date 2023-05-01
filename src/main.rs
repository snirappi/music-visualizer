use std::{sync::{mpsc::{Sender, Receiver, self}}, thread};
use nannou::{prelude::*} ;

mod audio;
use crate::audio::{record, process_audio};
mod scene;
use crate::scene::*;

const BUFFER_WINDOW: usize = 50;
struct Model {
    buffer: Receiver<Vec<f32>>,
    transition_speed: Sensitivity,
    particle: Particle,
    particle2: Particle,
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
        transition_speed: Sensitivity::MED,
        particle: Particle::NONE,
        particle2: Particle::NONE,
    }
}

fn update(app: &App, model: &mut Model, update: Update){
    let mut main_particle_speed = 0;
    let mut secondary_particle_speed = 0;
    match model.transition_speed {
        Sensitivity::HIGH => {
            main_particle_speed = 15;
            secondary_particle_speed = 2;
        },
        Sensitivity::MED => {
            main_particle_speed = 30;
            secondary_particle_speed = 8;
        },
        Sensitivity::LOW => {
            main_particle_speed = 60;
            secondary_particle_speed = 15;
        },
    };
    if app.time as usize % main_particle_speed == 0 {
        model.particle = particle_selection();
    }
    if app.time as usize % secondary_particle_speed == 0 {
        model.particle2 = particle_selection();
    } else {
        model.particle2 = Particle::NONE;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let recv_buffer = &model.buffer.recv().unwrap();

    let fft_array = &recv_buffer[BUFFER_WINDOW..recv_buffer.len() - BUFFER_WINDOW];
    let scene = Scene {
        app,
        draw: app.draw(),
        frame,
        sensitivity: Sensitivity::HIGH, 
        particle: model.particle,
        particle2: model.particle2,
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