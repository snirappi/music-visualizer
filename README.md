# music-visualizer
Rust Music Visualizer

## Usage
Run `cargo build` and run the .exe or `cargo run` to see the visualizer.
The visualizer will use whatever is the current default input device for your system.

## Details
This application uses `cpal` to retrieve sound data from Input devices which then sends to `rustfft` to process (Fast Fourier Transform) and finally uses `nannou` to draw.
