# Audio System

The audio system ensures high-quality, synchronized audio mixing for video export.

## Components

### `AudioMixer` (`src/audio.rs`)
The mixer holds a list of `AudioTrack`s. It does not run a realtime audio thread (unlike a game engine). Instead, it mixes "chunks" of audio on demand for each video frame.

### `AudioTrack`
*   **Data**: Stores the *entire* decoded audio file in memory as `Vec<f32>` (Interleaved Stereo).
*   **Properties**: `start_time`, `volume` (Animated), `trim_start`.
*   **Memory Usage**: Since video rendering is offline, we prioritize simplicity over streaming. A 3-minute song is ~30MB in float32, which is acceptable.

## Pipeline

1.  **Loading**:
    *   `symphonia` decodes the input file (MP3, WAV, AAC).
    *   **Resampling**: If the source sample rate != 48kHz (our internal standard), `rubato` is used to resample it to 48kHz high-quality.
    *   Data is converted to Stereo Float32.

2.  **Mixing (`Director::mix_audio`)**:
    *   Called every frame during export.
    *   Calculates the number of samples needed for the current frame duration (`samples = sample_rate / fps`).
    *   Iterates all active tracks.
    *   Slices the relevant samples based on `current_time`.
    *   Applies volume/fades.
    *   Sums into a mix buffer.

3.  **Nested Compositions**:
    *   If a `CompositionNode` is active, the mixer recurses into the child Director.
    *   The child Director mixes its own audio for the requested duration.
    *   The result is added to the parent's mix buffer.

## Synchronization

*   **Sample-Exact**: We calculate exactly how many samples correspond to the video frame duration.
*   **Drift Prevention**: We track the *total* samples written so far rather than calculating per-frame delta errors, ensuring long videos stay in sync.
