use nannou::prelude::*;

pub enum Sensitivity {
    HIGH,
    MED,
    LOW,
}
pub struct Scene<'a> {
    pub app: &'a App,
    pub draw: Draw,
    pub frame: Frame<'a>,
    pub sensitivity: Sensitivity,
}

#[derive(Clone, Copy)]
struct EQ {
    bass_amp: f32,
    bass_trigger: f32,
    mid_amp: f32,
    mid_trigger: f32,
    treb_amp: f32,
    treb_trigger: f32,
    total_amp: f32,
}

impl Scene<'_> {
    fn draw_rainbow_squares(self: &Self, fft_array: Vec<f32>, eq: EQ) {
        for i in -10..10 {
            for j in -10..10 {
                if i > -5 && i < 5 || j > -5 && j < 5 {
                    continue;
                }
                let color_factor = fft_array[random_range(0, fft_array.len() - 1)];

                let fract = i as f32 / 360.0;
                let r = (self.app.time + fract) % 1.0;
                let g = (self.app.time - 1.0 - fract) % 1.0;
                let b = (self.app.time + 0.5 + fract) % 1.0;
                let rgba = srgba(r, g, b, color_factor);
                self.draw
                    .rect()
                    .w_h(50.0, 50.0)
                    .x_y(
                        fft_array[random_range(0, fft_array.len() - 1)] * 120.0 * i as f32,
                        fft_array[random_range(0, fft_array.len() - 1)] * 120.0 * j as f32,
                    )
                    .color(rgba)
                    .rotate(random_range(-0.1, 0.1));
            }
        }
    }
    fn draw_rainbow_circle(self: &Self, fft_array: Vec<f32>, eq: EQ) {
        // Map over an array of integers from 0 to 360 to represent the degrees in a circle.
        // get middle 360 points
        let points = (0..360)
            .map(|i| {
                // Convert each degree to radians.
                let radian = deg_to_rad(i as f32);
                let diff = fft_array.len() - 360;
                let fft_val = fft_array[i + diff] as f32;
                // Get the sine of the radian to find the x co-ordinate of this point of the circle
                // and multiply it by the radius.
                let x = radian.sin() * 1024.0 * fft_val;
                // Do the same with cosine to find the y co-ordinate.
                let y = radian.cos() * 1024.0 * fft_val;
                // Construct and return a point object with a color.
                pt2(x, y)
            })
            .enumerate()
            .map(|(i, p)| {
                let fract = i as f32 / 360.0;
                let r = (self.app.time + fract) % 1.0;
                let g = (self.app.time - 1.0 - fract) % 1.0;
                let b = (self.app.time + 0.5 + fract) % 1.0;
                let rgba = srgba(r, g, b, random_range(0.3, 1.0));
                (p, rgba)
            });
        self.draw
            .polyline()
            .weight(self.app.time.sin() * 50.0 * eq.total_amp)
            .points_colored(points);
    }

    fn backgrounds(self: &Self, eq: EQ) {
        let r = (self.app.time) % 1.0;
        let g = (self.app.time + eq.total_amp) % 1.0;
        let b = (self.app.time - 0.5 + eq.total_amp) % 1.0;
        let rgba = srgba(r, g, b, eq.total_amp);
        if eq.total_amp < 0.1 {
            self.draw.background().color(BLACK);
        } else if eq.bass_amp > eq.bass_trigger {
            self.draw.background().color(rgba);
        } else if eq.treb_amp > eq.treb_trigger {
            self.draw.background().color(rgba);
        } else if random_range(0, 10) > random_range(0, 10) {
            self.draw.background().color(BLACK);
        }
    }

    fn find_3eq(self: &Self, fft_array: Vec<f32>) -> EQ {
        let mut bass_trigger = 0.0;
        let mut mid_trigger = 0.0;
        let mut treb_trigger = 0.0;
        match self.sensitivity {
            Sensitivity::HIGH => {
                bass_trigger = 0.5;
                mid_trigger = 0.5;
                treb_trigger = 0.5;
            }
            Sensitivity::MED => {
                bass_trigger = 0.7;
                mid_trigger = 0.7;
                treb_trigger = 0.7;
            }
            Sensitivity::LOW => {
                bass_trigger = 1.0;
                mid_trigger = 1.0;
                treb_trigger = 1.0;
            }
        }
        let bass_amp: f32 = fft_array[0..25].iter().sum::<f32>() / 25.0;
        let mid_amp: f32 = fft_array[(fft_array.len() / 2) - 15..(fft_array.len() / 2) + 15]
            .iter()
            .sum::<f32>()
            / 30.0;
        let treb_amp: f32 = fft_array[fft_array.len() - 25..].iter().sum::<f32>() / 25.0;
        let total_amp: f32 = fft_array.iter().sum::<f32>() / fft_array.len() as f32;
        return EQ {
            bass_amp,
            bass_trigger,
            mid_amp,
            mid_trigger,
            treb_amp,
            treb_trigger,
            total_amp,
        };
    }

    pub fn run(self: &Self, fft_array: Vec<f32>) {
        let eq = self.find_3eq(fft_array.clone());
        self.backgrounds(eq.clone());
        if eq.treb_amp < eq.treb_trigger {
            self.draw_rainbow_circle(fft_array.clone(), eq.clone());
        }
        if eq.mid_amp < eq.mid_trigger {
            self.draw_rainbow_squares(fft_array.clone(), eq.clone());
        }
        self.draw.to_frame(self.app, &self.frame).unwrap();
    }
}
