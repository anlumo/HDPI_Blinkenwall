use portaudio as pa;
use std::thread;
use atomic_ring_buffer::AtomicRingBuffer;
use std::sync::Arc;

const FRAMES: usize = 512;

pub struct AudioInput {
    portaudio: pa::PortAudio,
    stream: pa::Stream<pa::NonBlocking, pa::stream::Input<f32>>,
    ringbuffer: Arc<AtomicRingBuffer<[f32; FRAMES*2], [[f32; FRAMES*2]; 5]>>, // FRAMES*2 due to having two channels!
}

impl<'a> AudioInput {
    pub fn new() -> AudioInput {
        let portaudio = pa::PortAudio::new().unwrap();

        let input_device = portaudio.default_input_device().unwrap();
        let latency = portaudio.device_info(input_device).unwrap().default_low_input_latency;
        let input_params = pa::StreamParameters::<f32>::new(input_device, 2, true, latency);
        let settings = pa::stream::InputSettings::new(input_params, 44100.0, FRAMES as u32);
        let buffer = [[0.0; FRAMES*2]; 5];

        let ringbuffer = Arc::new(AtomicRingBuffer::new(buffer));

        let callback = {
            let ringbuffer = ringbuffer.clone();
            move |pa::stream::InputCallbackArgs { buffer, frames, flags, time }| {
                assert_eq!(frames, FRAMES);
                ringbuffer.enqueue(|x| {
                    x.clone_from_slice(buffer);
                });
                pa::Continue
            }
        };

        AudioInput {
            stream: portaudio.open_non_blocking_stream(settings, callback).unwrap(),
            portaudio: portaudio,
            ringbuffer: ringbuffer,
        }
    }

    pub fn start(&mut self) -> Result<(), pa::error::Error> {
        self.stream.start()
    }

    pub fn stop(&mut self) -> Result<(), pa::error::Error> {
        self.stream.stop()
    }

    pub fn poll(&mut self) -> Result<[f32; FRAMES*2], ()> {
        let mut result = try!(self.ringbuffer.dequeue_spin(|x| *x));
        loop {
            match self.ringbuffer.dequeue_spin(|x| *x) {
                Ok(buffer) => {
                    result = buffer;
                },
                Err(()) => {
                    return Ok(result);
                }
            }
        }
    }
}
