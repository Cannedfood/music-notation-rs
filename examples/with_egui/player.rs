use std::collections::VecDeque;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use music_notation::note::rhythm::{Duration, Time};
use music_notation::note::Note;

pub enum PlayerCommands {
    SetBuffer(Vec<Note>),
    SetTime(Time),
    Start,
    Pause,
}

pub enum PlayerEvents {
    Time(Time),
}

pub struct PlayerState {
    pub upcoming_notes: VecDeque<Note>,
    pub active_notes: VecDeque<Note>,
    pub commands: std::sync::mpsc::Receiver<PlayerCommands>,
    pub events: std::sync::mpsc::Sender<PlayerEvents>,
    pub current_time_in_beats: f64,
    pub beats_to_seconds: f64,
    pub playing: bool,
}
impl PlayerState {
    pub fn new() -> (
        Self,
        std::sync::mpsc::Sender<PlayerCommands>,
        std::sync::mpsc::Receiver<PlayerEvents>,
    ) {
        let (command_send, command_receive) = std::sync::mpsc::channel();
        let (event_send, event_receive) = std::sync::mpsc::channel();
        (
            Self {
                upcoming_notes: VecDeque::new(),
                active_notes: VecDeque::new(),
                commands: command_receive,
                events: event_send,
                current_time_in_beats: 0.0,
                beats_to_seconds: 60.0 / 120.0,
                playing: false,
            },
            command_send,
            event_receive,
        )
    }

    pub fn update(&mut self, buffer: &mut [f32], sample_rate: u32, channels: usize) {
        let sample_rate = sample_rate as f64;
        let num_samples = buffer.len() / channels;

        self.handle_commands();

        if !self.playing {
            return;
        }

        let buffer_end_time =
            self.current_time_in_beats + num_samples as f64 / sample_rate / self.beats_to_seconds;

        // Add notes that have started to the active notes
        {
            let buffer_len_beats = num_samples as f64 / sample_rate * self.beats_to_seconds;
            let buffer_end_musical_time = Time::ZERO
                + Duration::from_beats_f64(self.current_time_in_beats + buffer_len_beats);

            while !self.upcoming_notes.is_empty()
                && self.upcoming_notes[0].time <= buffer_end_musical_time
            {
                self.active_notes
                    .push_back(self.upcoming_notes.pop_front().unwrap());
            }
        }

        // Remove notes that have ended from the active notes
        self.active_notes.retain(|note| {
            let note_start_beats = (note.time - Time::ZERO).beats() - self.current_time_in_beats;
            let note_end_beats =
                (note.time + note.duration - Time::ZERO).beats() - self.current_time_in_beats;
            if note_end_beats < 0.0 {
                return false;
            }

            generate_audio(
                buffer,
                channels,
                sample_rate,
                (note_start_beats * self.beats_to_seconds) * sample_rate,
                (note_end_beats * self.beats_to_seconds) * sample_rate,
                note.pitch.frequency_hertz() as f64,
                note.velocity.to_f64(),
            );

            true
        });

        self.current_time_in_beats = buffer_end_time;
        self.events
            .send(PlayerEvents::Time(
                Time::ZERO + Duration::from_beats_f64(self.current_time_in_beats),
            ))
            .unwrap();
    }

    fn handle_commands(&mut self) {
        while let Ok(command) = self.commands.try_recv() {
            match command {
                PlayerCommands::SetBuffer(notes) => {
                    self.active_notes.clear();
                    self.upcoming_notes = notes.into_iter().collect()
                }
                PlayerCommands::SetTime(time) => {
                    self.current_time_in_beats = (time - Time::ZERO).beats();
                }
                PlayerCommands::Start => self.playing = true,
                PlayerCommands::Pause => self.playing = false,
            }
        }
    }
}

fn generate_audio(
    buffer: &mut [f32],
    channels: usize,
    sample_rate: f64,
    start_samples: f64,
    end_samples: f64,
    freq_hz: f64,
    velocity: f64,
) {
    let buffer_len = (buffer.len() / channels) as i64;

    let buf_start = (start_samples.ceil() as i64).clamp(0, buffer_len) as usize;
    let buf_end = (end_samples.ceil() as i64).clamp(0, buffer_len) as usize;
    if buf_end <= buf_start {
        return;
    }

    let attack_duration = sample_rate * (0.02 * velocity + 0.06 * (1.0 - velocity));
    let decay_duration = sample_rate * 0.02;

    let step = 2.0 * std::f64::consts::PI * freq_hz / sample_rate;
    let mut w = (buf_start as f64 - start_samples) * step;
    for i in buf_start..buf_end {
        let decay = ((end_samples - i as f64) / decay_duration).clamp(0.0, 1.0);
        let attack = ((i as f64 - start_samples) / attack_duration).clamp(0.0, 1.0);
        let val = (velocity * attack * decay * w.sin()) as f32;
        for sample in buffer[i * channels..(i + 1) * channels].iter_mut() {
            *sample += val;
        }
        w += step;
    }
}

pub struct Player {
    stream: cpal::Stream,
    pub commands: std::sync::mpsc::Sender<PlayerCommands>,
    pub events: std::sync::mpsc::Receiver<PlayerEvents>,
}

pub fn start_player() -> Player {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let config = device.default_output_config().unwrap();

    let (mut state, commands, events) = PlayerState::new();

    let channels = config.channels() as usize;
    let sample_rate = config.sample_rate().0;
    let stream = device
        .build_output_stream(
            &config.config(),
            move |buf: &mut [f32], _info| {
                buf.fill(0.0);
                state.update(buf, sample_rate, channels);
                for sample in buf.iter_mut() {
                    *sample = (*sample * 0.1).tanh();
                }
            },
            |e| {
                eprintln!("an error occurred on the output audio stream: {}", e);
            },
            None,
        )
        .unwrap();

    let player = Player {
        stream,
        commands,
        events,
    };
    player.stream.play().unwrap();

    player
}
