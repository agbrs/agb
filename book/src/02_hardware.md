# The Game Boy Advance hardware

The Game Boy Advance was released by Nintendo in Japan in March 2001 and in North America in the following June.
It has a 2.9 inch screen with a 240x144 pixel resolution, and contains a 32-bit 16.8MHz ARM CPU.
It was developed to be the successor to the Game Boy Color and internally codenamed the 'Advanced Game Boy' (agb) which is where this crate gets its name.

# What makes the GBA unique?

The Game Boy Advance is (fairly) unique amongst retro handheld consoles.
It was developed at a time where processors weren't powerful enough to be able to push an entire screen of pixels to the screen every frame.
Therefore, it has a special Pixel Processing Unit (PPU) which is sort of similar to a modern day graphics card, except it is very games focused.
For example, the GBA has a concept of 'hardware sprites' and 'hardware backgrounds' which we'll go in to more detail in the next section.
This hardware 2d capabilities gives the GBA the unique characteristics with the games developed for it.

However, despite this, it is possible to write code for it using modern tools and programming languages thanks to the ARM CPU it contains.
The CPU is modern enough to be supported by LLVM and rust to give a reasonably trouble free experience.

So the GBA lets you take advantage of modern tooling while also giving you the ability to see what programming for retro consoles was like at the time!

# Capabilities of the hardware

The GBA is fundamentally a 2D system, and a lot of the hardware accelerated graphics is designed to support this.
The relevant features for this book are:

* 256 sprites which can be from 8x8 to 64x64 pixels in size
* 4 background layers which are enabled / disabled depending on the graphics mode
* Background tiles, 8x8 pixel tiles are used in the background layers if they are in tile mode.
* 8-bit sound. You have the ability to send 8-bit raw audio data to the speakers (optionally stereo).

You can read more about the GBA on [gbatek](https://rust-console.github.io/gbatek-gbaonly/).

agb tries to abstract some of this away from you to give you less to remember and less that can go wrong.
If you want to try playing around directly with the hardware, the best place to look is [tonc](https://www.coranac.com/tonc/text/).