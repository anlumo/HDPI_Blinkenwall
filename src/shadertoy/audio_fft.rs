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

    /*
    ^(UInt32 channelsPerFrame, UInt32 bytesPerFrame, UInt32 frameCount, void* frames) {
        int log2n = log2f(frameCount);
        frameCount = 1 << log2n;

        SInt16* samples16 = (SInt16*)frames;
        SInt32* samples32 = (SInt32*)frames;

        if (bytesPerFrame / channelsPerFrame == 2)
        {
            for (int i = 0, j = 0; i < frameCount * channelsPerFrame; i+= channelsPerFrame, j++)
            {
                originalReal[j] = samples16[i] / 32768.0;
            }
        }
        else if (bytesPerFrame / channelsPerFrame == 4)
        {
            for (int i = 0, j = 0; i < frameCount * channelsPerFrame; i+= channelsPerFrame, j++)
            {
                originalReal[j] = samples32[i] / 32768.0;
            }
        }

        vDSP_ctoz((COMPLEX*)originalReal, 2, &fftInput, 1, frameCount);

        const float one = 1;
        float scale = (float)1.0 / (2 * frameCount);

        //Take the fft and scale appropriately
        vDSP_fft_zrip(setupReal, &fftInput, 1, log2n, FFT_FORWARD);
        vDSP_vsmul(fftInput.realp, 1, &scale, fftInput.realp, 1, frameCount/2);
        vDSP_vsmul(fftInput.imagp, 1, &scale, fftInput.imagp, 1, frameCount/2);

        //Zero out the nyquist value
        fftInput.imagp[0] = 0.0;

        //Convert the fft data to dB
        vDSP_zvmags(&fftInput, 1, obtainedReal, 1, frameCount/2);


        //In order to avoid taking log10 of zero, an adjusting factor is added in to make the minimum value equal -128dB
        //      vDSP_vsadd(obtainedReal, 1, &kAdjust0DB, obtainedReal, 1, frameCount/2);
        vDSP_vdbcon(obtainedReal, 1, &one, obtainedReal, 1, frameCount/2, 0);

        // min decibels is set to -100
        // max decibels is set to -30
        // calculated range is -128 to 0, so adjust:
        float addvalue = 70;
        vDSP_vsadd(obtainedReal, 1, &addvalue, obtainedReal, 1, frameCount/2);
        scale = 5.f; //256.f / frameCount;
        vDSP_vsmul(obtainedReal, 1, &scale, obtainedReal, 1, frameCount/2);

        float vmin = 0;
        float vmax = 255;

        vDSP_vclip(obtainedReal, 1, &vmin, &vmax, obtainedReal, 1, frameCount/2);
        vDSP_vfixu8(obtainedReal, 1, buffer, 1, MIN(256,frameCount/2));

        addvalue = 1.;
        vDSP_vsadd(originalReal, 1, &addvalue, originalReal, 1, MIN(256,frameCount/2));
        scale = 128.f;
        vDSP_vsmul(originalReal, 1, &scale, originalReal, 1, MIN(256,frameCount/2));
        vDSP_vclip(originalReal, 1, &vmin, &vmax, originalReal, 1,  MIN(256,frameCount/2));
        vDSP_vfixu8(originalReal, 1, &buffer[256], 1, MIN(256,frameCount/2));
    }
    */

    pub fn process(&self, data: &Vec<f32>) -> Vec<f32> {
        let mut input: Vec<Complex<f32>> = data.iter().map(|x| Complex::<f32>::new(*x, 0.)).collect();
        let mut output: Vec<Complex<f32>> = vec![Complex::zero(); self.size];

        self.fft.process(&mut input, &mut output);

        output.iter().map(|x| (x.norm()+1.0).ln()).collect()
    }
}
