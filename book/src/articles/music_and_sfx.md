# Music and sound effects

No game is complete without music and sound effects.
The Game Boy Advance doesn't have built-in hardware support for sound mixing, so in order to play more than one sound at once, you'll need to use a software mixer.

`agb`'s built-in software mixer allows for up to 8 simultaneous sounds to be played at once at various speeds and volumes.
It also (through the [`agb-tracker` crate](https://crates.io/crates/agb_tracker)) can play basic [tracker music](https://en.wikipedia.org/wiki/Music_tracker). Usage of both will be covered in this article.

# Choice of frequency

`agb`'s mixer works at a fixed frequency which you choose when creating it.
Once chosen, you cannot change the frequency during the game without first dropping the mixer.

There are 3 supported frequencies, with higher frequencies having noticeably better sound quality but using significantly more CPU.
The following is just an indication as to how much CPU time per frame will be used by audio, actual results will vary greatly depending on the number of channels currently playing and what they are playing.

One thing to note here is that the actual hardware has a very poor speaker, and even through the headphones it has quite a lot of noise.
And with how little CPU time there is, and the fact that the audio hardware produces 8-bit audio[^9-bit], don't expect amazing sound.

[^9-bit]:
    Technically there's a trick you can use to get 9-bit audio out of the Game Boy Advance.
    You will be limited to mono only if you use that trick, and it uses large quantities of ROM space to store the extra information that it generally isn't worth it.

| Frequency | Audio quality                                                       | Approximate CPU usage |
| --------- | ------------------------------------------------------------------- | --------------------- |
| 10,512Hz  | Poor - even bad out of the speakers                                 | ~5% per frame         |
| 18,157Hz  | Low - speakers sound fine but headphones are still a little crunchy | ~10% per frame        |
| 32,768Hz  | Medium - speakers sound great and headphones are fine               | ~20% per frame        |

# Preparing the samples

The CPU on the Game Boy Advance isn't powerful enough to decompress audio while also being able to play a game at the same time.
So all audio is stored uncompressed, making them quite big.
For a lot of games, most of the space in the ROM is taken up by music and sound effects.

`agb` only supports `wav` files for uncompressed audio.
And they are _not_ resampled before loading, so you must ensure that the `wav` files are at the sample rate you've chosen for your game.

You can use [`ffmpeg`](https://ffmpeg.org/) to resample any audio to your chosen frequency (and convert to a wav file) like follows:

```sh
ffmpeg -i path/to/audio.mp3 -ar 18157 sfx/audio.wav
```

# Loading the sample

You can load the sample by using the [`include_wav!`](https://docs.rs/agb/latest/agb/macro.include_wav.html) macro.
This returns a [`SoundData`](https://docs.rs/agb/latest/agb/sound/mixer/struct.SoundData.html) which you can later pass to the mixer to play.

```rust
use agb::{
    include_wav,
    sound::mixer::SoundData,
};

static BACKGROUND_MUSIC: SoundData = include_wav!("sfx/audio.wav");
```

# Managing the mixer

In order to actually play the music, you'll need a sound mixer which you can get from the `Gba` struct.
This is where you pass your chosen frequency.

```rust
use agb::sound::mixer::Frequency;

let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
```

Now that you have the mixer, you need to call [`.frame()`](https://docs.rs/agb/latest/agb/sound/mixer/struct.Mixer.html#method.frame) at least once per frame.
If you don't do that, then the audio will 'skip', which is very noticeable for players.

```rust
loop {
    let mut frame = gfx.frame();
    // do your per-frame game update stuff

    mixer.frame();
    frame.commit();
}
```

# Playing sounds

Music and sound effects are treated in the same way.
The mixer manages a number of concurrent channels which will all play at once.
There can be at most 8 channels playing.

Create a new channel by constructing a new [`SoundChannel`](https://docs.rs/agb/latest/agb/sound/mixer/struct.SoundChannel.html).

```rust
let mut background_music = SoundChannel::new(BACKGROUND_MUSIC);
background_music.stereo();
```

There are various methods you can use to change how the sound channel is played.
For example, you can change its volume, or the speed at which it is played (effecting the pitch).

Then, play the sound with:

```rust
mixer.play_sound(background_music);
```

This function returns an `Option<ChannelId>`.
You will get `Some` if there was a free space for this channel to be played, and you can later retrieve this same channel using `mixer.channel(channel_id)` in case you want to change how it is being played, or to stop it.

# Sound playback settings

There are few things you can tweak about how the sound effect is played which is useful in games.
Note that if you are playing stereo sound, you _cannot_ change any of these properties, and any attempt to do so will be ignored.

1. Pitch.
   You can change the pitch by using the `.playback()` method, which takes a speed as a fixnum for how fast this sample should be played.
   `1` is unchanged, `2` is double speed etc.
2. Volume.
   You can change the volume by using the `.volume()` method, which takes the new volume as a fixnum.
   `1` is unchanged, `0.5` is half volume etc.
   Setting this too high will cause clipping in the final audio.
3. Panning.
   This will change the volume on a per-side basis, and is changed using the `.panning()` method.
   On actual Game Boy Advance hardware, there is only 1 speaker, so this only works on emulators or if the player has headphones.
   `-1` is fully to the left, `1` is fully to the right and `0` is the default and plays centrally.

# Modifying playing sounds

With a given `ChannelId` retrieved from the call to `play_sound()`, you can alter how the sound effect is played.
The `mixer.channel(channel_id)` method will return the `SoundChannel` and then you can apply the effects mentioned above to change how it is played.

The `.stop()` method will cause this channel to stop playing and free it up for a different one.
This is useful for level transitions to stop the background music from playing once you're done with it.

And there is `.pause()` and `.resume()` which doesn't free up the current channel, and allows you to resume from where you left off at a later point.

# Sound priorities

By default, sounds are 'low priority'.
These will _not_ play if there are already 8 sounds playing at once.
You can also state that your channel is 'high priority'.
These will always play (and `.play_sound()` will panic if it can't find a slot to play this sound effect), and will remove low priority sounds from the playing list if there isn't currently space.

You should only use high priority sounds for important things, like required sound effects and your background music (if you're not using a tracker).

Create a high priority channel with [`SoundChannel::new_high_priority()`](https://docs.rs/agb/latest/agb/sound/mixer/struct.SoundChannel.html#method.new_high_priority).

```rust
let mut background_music = SoundChannel::new_high_priority(BACKGROUND_MUSIC);
background_music.stereo();
```

# Tracker music

You'll find you run out of ROM space very quickly if you start including high quality audio for all the background music you want.
For example, even just 4 minutes of music at 32,768Hz will take up about 10MB of space (maximum cartridge size is 32MB with most being 16MB).
So ideally you'd want to make your music take up less space.

For that, you use [tracker music](https://en.wikipedia.org/wiki/Music_tracker).
This stores individual samples of each instrument, along with instructions on what volume and pitch to play each note.
These take up much less space than full, uncompressed audio.
Using tracker music, you could reduce the same 4 minutes of music to just a few kilobytes.

Creating tracker files is outside the scope of this book, but often you would use a tool like [Milkytracker](https://milkytracker.org/) or [OpenMPT](https://openmpt.org/) to compose your music.
`agb` has good support for the `xm` file format (native to milkytracker and an option for OpenMPT).

To get tracker support, include the `agb-tracker` crate as follows:

```bash
cargo add agb_tracker
```

Then import your xm background music

```rust
use agb_tracker::{Track, include_xm};

static BGM: Track = include_xm!("sfx/bgm.xm");
```

For each track you want to play at once, you need an instance of the [`Tracker`](https://docs.rs/agb_tracker/latest/agb_tracker/type.Tracker.html).

```rust
use agb_tracker::Tracker;

let mut bgm_tracker = Tracker::new(&BGM);
```

You can now play this background music using the `step()` function which you would call at some point before the `.frame()` function on the mixer.

```rust
bgm_tracker.step(&mut mixer);
mixer.frame();
```

The `Tracker` will manage playing the various samples from the `xm` file at the right time, pitch and volume.

Because it uses the mixer under-the-hood, the `xm` file can play at most 8 samples at once, and each of those samples take up a slot for sound effects.
This is also more CPU intensive then just playing a single sound effect as the background music because more channels are being used at once.
The actual book-keeping that the Tracker needs to do on a per-frame basis to play the music is fairly lightweight.
