use std::sync::{Arc, Mutex, mpsc::{Sender, channel, Receiver}};

use cpal::{traits::{HostTrait, DeviceTrait, StreamTrait}, Host, Device};
use rustfft::{FftPlanner, num_complex::{Complex, ComplexFloat}};


const BUFFER_SIZE : usize = 512;
const SMOOTHING : f32 = 0.5;
pub fn playback(clip: Vec<f32>) -> Result<(), String>{
    let host: Host = cpal::default_host();
    let device: Device = host.default_output_device().expect("could not find default output device");
    println!("Output Device: {:?}", device.name());
    let mut supported_configs_range = device.supported_output_configs().expect("error while querying configs");
    let supported_config = supported_configs_range.next().expect("no supported config").with_max_sample_rate();
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    println!("Sample Rate: {:?}, Sample Format: {}", supported_config.sample_rate(), supported_config.sample_format());
    type StateHandle = Arc<Mutex<Option<(usize, Vec<f32>, Sender<()>)>>>;
    let (done_tx, done_rx) = channel::<()>();
    let state = (0, clip, done_tx);
    let state = Arc::new(Mutex::new(Some(state)));
    fn write_output_data<T>(output: &mut [f32], writer: &StateHandle)
        {
            if let Ok(mut guard) = writer.try_lock() {
                if let Some((i, clip_samples, done)) = guard.as_mut() {
                    for frame in output.chunks_mut(2) {
                        for sample in frame.iter_mut() {
                            *sample = *clip_samples.get(*i).unwrap_or_else(|| &0.0);
                        }
                        *i += 1;
                    }
                    if *i >= clip_samples.len() {
                        if let Err(_) = done.send(()) {
                            // Playback has already stopped. We'll be dead soon.
                        }
                    }
                }
            }
        }
        let stream = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &supported_config.into(),
                move |data, _: &_| write_output_data::<f32>(data, &state),
                err_fn,
                None
            ),
            _ => todo!()
        }.unwrap();

        stream.play();
        done_rx.recv();
        Ok(())
}

pub fn record(sender: Sender<f32>){

    let host: Host = cpal::default_host();
    let device: Device = host.default_input_device().expect("no input device available");
    println!("Input Device: {:?}", device.name());
    let mut supported_configs_range = device.supported_input_configs().expect("error while querying configs");
    let supported_config = supported_configs_range.next().expect("no supported config").with_max_sample_rate();
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    fn read_input_data<T>(input: &[f32], sender: &Sender<f32>)
    {
        for s in input.chunks(2) {
            sender.send(s[0]).expect("Failed to send");
        }
    }

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(&supported_config.into(), move |data, _: &_| read_input_data::<f32>(data, &sender), err_fn,None),
        _ => todo!(),

    };
    let unwrapped_stream = stream.unwrap();
    unwrapped_stream.play().expect("play fucked up");
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel.")).expect("ctrc set handler failed");

    println!("Waiting for Ctrl-C...");
    rx.recv().expect("recv failed");
    println!("Got it! Done Recording...");
    drop(unwrapped_stream);
}

pub fn process_audio(rx: Receiver<f32>, tx: Sender<Vec<f32>>){
    let mut i : usize = 0;
    let mut buffer = vec![Complex{re: 0.0f32, im: 0.0f32}; BUFFER_SIZE * 2];
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(BUFFER_SIZE * 2);
    for recieved in rx {
        //handle recieved f32
        buffer[i] = recieved.into();
        i = i + 1;
        if i > buffer.len() - 1 {
            fft.process(& mut buffer);
            let mut graph1 = vec![0.0; BUFFER_SIZE];
            let mut graph2 = vec![0.0; BUFFER_SIZE];
            for (i, freq) in buffer.iter().enumerate() {
                if i < BUFFER_SIZE {
                    graph1[i] = make_freq_array(*freq);
                } else {
                    graph2[i - BUFFER_SIZE] = make_freq_array(*freq);
                }
            }
            i = 0;

            tx.send(smoothing(graph1, graph2, SMOOTHING));
        }
    }
}

fn make_freq_array(num: Complex<f32>) -> f32{
    let value = num.abs();
    return value;
}

// Smoothing param should be 1.0 - 0.0
fn smoothing(graph1: Vec<f32>, graph2: Vec<f32>, smoothing: f32) -> Vec<f32>{
    if graph1.len() != graph2.len() {
        panic!("FFT Buffers not equal size for smoothing!");
    }
    let mut graph = vec![0.0; graph1.len()];
    for (i, freq) in graph1.iter().enumerate() {
       graph[i] = graph1[i] * smoothing + (1.0 - smoothing) * graph2[i];
    }
    graph
}