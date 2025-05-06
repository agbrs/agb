# Background music

In this section we're going to add some music and sound effects to the game to make it feel more alive.

First we'll put some sound effects when the ball hits a paddle, and then we'll add some background music.

## Audio in agb

In `agb`, audio is managed through the [`Mixer`](https://docs.rs/agb/latest/agb/sound/mixer/struct.Mixer.html).
Create a mixer from the `gba` struct, passing through the frequency you intend to use.
For this section, we'll use 32768Hz.

Get yourself a mixer by adding this near the beginning of your `main` function.

```rust
use agb::sound::mixer::Frequency;

let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
```

Just before the loop, you'll want to enable the mixer.
It is best not to enable it too soon, because as soon as you enable the mixer, you should start calling
[`mixer.frame()`](https://docs.rs/agb/latest/agb/sound/mixer/struct.Mixer.html#method.frame).
Failing to do so will cause the audio to skip.
It is best to call this right before `frame.commit()`.
So let's do this now and add

```rust
mixer.frame(); // new code here
frame.commit();
```

## Generating the wav files

`agb` can only play `wav` files.
You can download the file from [here](ball-paddle-hit.wav), or generate the same sound yourself on [sfxr](https://sfxr.me/#57uBnWbcktkrVgQNCAgSRbsJfYTWqQacVxoPWQ2mduecQZiZfcMwFF6jp4vQs185AwzxKsDDp4dc4p5fLGnQfNpA7dHvnZYBDDWPuH34JrhczFyZq74yWYW3H).

The final file should go in the `sfx` directory in your game.

The file must be a 32768Hz wav file.
Any other frequency will result in the sound being played at a different speed than what you would expect.
You can use `ffmpeg` to convert to a file with the correct frequency with a command similar to this:

```sh
ffmpeg -i ~/Downloads/laserShoot.wav -ar 32768 sfx/ball-paddle-hit.wav
```

## Importing the sound effect

Import the wav file using [`include_wav`](https://docs.rs/agb/latest/agb/macro.include_wav.html).

```rust
use agb::{include_wav, mixer::SoundData};

static BALL_PADDLE_HIT: SoundData = include_wav!("sfx/ball-paddle-hit.wav");
```

## Playing the sound effect

To play a sound effect, you need to create a [`SoundChannel`](https://docs.rs/agb/latest/agb/sound/mixer/struct.SoundChannel.html).

```rust
use agb::sound::mixer::SoundChannel;

let hit_sound = SoundChannel::new(BALL_PADDLE_HIT);
mixer.play_sound(hit_sound);
```

We'll do this in a separate function:

```rust
fn play_hit(mixer: &mut Mixer) {
    let hit_sound = SoundChannel::new(BALL_PADDLE_HIT);
    mixer.play_sound(hit_sound);
}
```

and add the `play_hit(&mut mixer)` call where you handle the ball paddle hits.

## Background music

Because the GBA doesn't have much spare CPU to use, we can't store compressed audio as background music and instead have to store it as uncompressed.
Uncompressed music takes up a lot of space, and the maximum cartridge size is only 32MB, so that's as much as you can use.

Therefore, most commercial games for the GBA use tracker music for the background music instead.
These work like MIDI files, where rather than storing the whole piece, you instead store the instruments and which notes to play and when.
With this, you can reduce the size of the final ROM dramatically.

Composing tracker music is a topic of itself, but you can use the example [here](bgm.xm) for this example.
Copy this to `sfx/bgm.xm` and we'll see how to play this using agb.

Firstly you'll need to add another crate, `agb-tracker`.

```sh
cargo add agb_tracker
```

Then import the file:

```rust
use agb_tracker::{Track, include_xm};

static BGM: Track = include_xm!("sfx/bgm.xm");
```

You can create the tracker near where you enable the mixer:

```rust
use agb_tracker::Tracker;

let mut tracker = Tracker::new(&BGM);
```

and then to actually play the tracker, every frame you need to call `.step(&mut mixer)`.
So put the call to `tracker.step()` above the call to `mixer.frame()`

```rust
tracker.step(&mut mixer);
mixer.frame();
```

You will now have background music playing.

## Exercise

- Add a new sound effect for when the ball hits the wall rather than a paddle.
- Add another new sound effect for when the ball hits the back wall and the player looses.
