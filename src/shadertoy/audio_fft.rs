use rustfft::{num_complex::Complex, FftPlanner};
use std::sync::Arc;

pub struct AudioFFT {
    size: usize,
    fft: Arc<dyn rustfft::Fft<f32>>,
    scratch: Vec<Complex<f32>>,
}

impl AudioFFT {
    pub fn new(size: usize) -> AudioFFT {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft(size, rustfft::FftDirection::Forward);
        AudioFFT {
            size,
            scratch: vec![Complex::new(0., 0.); fft.get_inplace_scratch_len()],
            fft,
        }
    }

    pub fn process(&mut self, data: &[f32]) -> Vec<f32> {
        assert!(data.len() == self.size);
        let mut buffer: Vec<Complex<f32>> =
            data.iter().map(|x| Complex::<f32>::new(*x, 0.)).collect();

        self.fft
            .process_with_scratch(&mut buffer, &mut self.scratch);
        let output = &buffer[0..self.size / 2]; // only take the first half
        let scale = (2.0 * self.size as f32).recip();

        let mut output: Vec<Complex<f32>> = output
            .iter()
            .map(|x| Complex::<f32>::new(x.re * scale, x.im * scale))
            .collect();
        output[0].im = 0.0; // Zero out the nyquist value
        output
            .iter()
            .map(|x| {
                (5.0 * (x.norm_sqr().log10().mul_add(10.0, 70.0)))
                    .min(255.0)
                    .max(0.0)
                    / 255.0
            })
            .collect()
    }
}
