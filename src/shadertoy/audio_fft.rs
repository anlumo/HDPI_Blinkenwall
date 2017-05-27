use rustfft;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FFTplanner;
use std::sync::Arc;
use std::slice;

pub struct AudioFFT {
    size: usize,
    fft: Arc<rustfft::FFT<f32>>,
}

impl AudioFFT {
    pub fn new(size: usize) -> AudioFFT {
        let mut planner = FFTplanner::new(false);
        AudioFFT {
            size: size,
            fft: planner.plan_fft(size),
        }
    }

    pub fn process(&self, data: &Vec<f32>) -> Vec<f32> {
        assert!(data.len() == self.size);
        let mut input: Vec<Complex<f32>> = data.iter().map(|x| Complex::<f32>::new(*x, 0.)).collect();
        let mut output: Vec<Complex<f32>> = vec![Complex::zero(); self.size];

        self.fft.process(&mut input, &mut output);
        let output = &output[0..self.size/2]; // only take the first half
        let scale = (2.0 * self.size as f32).recip();

        let mut output: Vec<Complex<f32>> = output.iter().map(|x| Complex::<f32>::new(x.re * scale, x.im * scale)).collect();
        output[0].im = 0.0; // Zero out the nyquist value
        output.iter().map(|x| (5.0 * (x.norm_sqr().log10().mul_add(10.0, 70.0))).min(255.0).max(0.0) / 255.0).collect()
    }
}
