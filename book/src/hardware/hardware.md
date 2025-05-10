# The Game Boy Advance hardware

The Game Boy Advance is a handheld gaming console released by Nintendo in March 2001 in Japan and in North America in June of the same year.
It features a 2.9 inch screen with a 240x160 pixel resolution and is powered by a 32-bit 16.8MHz ARM CPU.
The console was developed as a successor to the Game Boy Color and was internally codenamed the 'Advanced Game Boy' (agb), which is where this crate gets its name.

# What makes the GBA unique?

The GBA was developed at a time when processors were not powerful enough to push an entire screen of pixels to the screen every frame.
As a result, it features a special Pixel Processing Unit (PPU) that is similar to a modern-day graphics card, but is optimized for gaming.
The console has a concept of "hardware sprites" and "hardware backgrounds," which we will explain in more detail in the next section.
These hardware 2D capabilities give the GBA its unique characteristics.

Despite being a retro console, the GBA is still compatible with modern tools and programming languages thanks to the ARM CPU it contains.
The CPU is modern enough to be supported by LLVM and Rust, which provide a reasonably trouble-free experience.
This allows developers to take advantage of modern tooling while experiencing what it was like to program for retro consoles at the time.

The combination of this weak hardware and retro PPU with support of modern tooling makes the GBA fairly unique among retro consoles.

# Capabilities of the hardware

The GBA is fundamentally a 2D system, and a lot of the hardware accelerated graphics is designed to support this.
The relevant features for this book are:

- 256 sprites which can be from 8x8 to 64x64 pixels in size
- 4 background layers which are enabled / disabled depending on the graphics mode
- Background tiles, 8x8 pixel tiles are used in the background layers if they are in tile mode.
- 8-bit sound. You have the ability to send 8-bit raw audio data to the speakers, optionally stereo.

You can read more about the specifics of the GBA on [gbatek](https://rust-console.github.io/gbatek-gbaonly/).
To simplify the development process, agb abstracts some of the GBA's hardware away from the developer, which reduces the number of things to remember and lessens the chance of something going wrong.
If you wish to experiment with the hardware directly, the best place to look is [tonc](https://www.coranac.com/tonc/text/).
