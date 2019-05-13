use portaudio as pa;
use std::thread;
use atomicring::AtomicRingBuffer;
use std::sync::Arc;

const FRAMES: usize = 1024;

pub struct AudioInput {
    portaudio: pa::PortAudio,
    stream: pa::Stream<pa::NonBlocking, pa::stream::Input<f32>>,
    ringbuffer: Arc<AtomicRingBuffer<[f32; FRAMES]>>,
}

impl<'a> AudioInput {
    pub fn new() -> AudioInput {
        let portaudio = pa::PortAudio::new().unwrap();

        let input_device = portaudio.default_input_device().unwrap();
        let latency = portaudio.device_info(input_device).unwrap().default_low_input_latency;
        let input_params = pa::StreamParameters::<f32>::new(input_device, 1, true, latency);
        let settings = pa::stream::InputSettings::new(input_params, 44100.0, FRAMES as u32);
        let ringbuffer = Arc::new(AtomicRingBuffer::<[f32; FRAMES]>::with_capacity(8));

        let callback = {
            let ringbuffer = ringbuffer.clone();
            move |pa::stream::InputCallbackArgs { buffer, frames, flags, time }| {
                assert_eq!(frames, FRAMES);
                let mut copy = [0.0; FRAMES];
                copy.copy_from_slice(buffer);
                ringbuffer.push_overwrite(copy);
                pa::Continue
            }
        };

        AudioInput {
            stream: portaudio.open_non_blocking_stream(settings, callback).unwrap(),
            portaudio,
            ringbuffer,
        }
    }

    pub fn start(&mut self) -> Result<(), pa::error::Error> {
        self.stream.start()
    }

    pub fn stop(&mut self) -> Result<(), pa::error::Error> {
        self.stream.stop()
    }

    pub fn poll(&mut self) -> Option<[f32; FRAMES]> {
        if let Some(mut result) = self.ringbuffer.try_pop() {
            loop {
                match self.ringbuffer.try_pop() {
                    Some(buffer) => {
                        result = buffer;
                    },
                    None => {
                        return Some(result);
                    }
                }
            }
        } else {
            None
        }
    }
}
