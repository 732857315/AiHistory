# Original composition and runtime synthesis

**A Small Step Through Time** is an original composition made for AiHistory.
Copyright (c) 2026 732857315. The composition, score and synthesis code are
licensed under this repository's MIT License.

The note-level score is `music-score.json`; `src/audio.rs` implements the same
48 bars as MIDI pitches and chords. The arrangement uses a pentatonic lead,
soft triad pads and bass in C major / A minor, 120 BPM, 4/4. Its 96-second cycle
accompanies the longer historical journey without assigning musical sections
to historical dates.

The browser synthesizes sine oscillators and an octave overtone with attack
and decay envelopes. Notes are scheduled with a short lookahead, disconnected
after use, resynchronized on seeking and suspended on pause. Narration lowers
the music volume. The score contains no borrowed melody, recordings, loops or
imported samples. There are no audio files to download or regenerate.

Optional narration uses the operating system/browser's Speech Synthesis API.
No third-party online TTS client or downloaded speech is part of the project.
Availability and licensing of device voices remain with their respective
providers; the project does not redistribute voice models.
