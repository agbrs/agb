# Text rendering

There are many techniques for displaying text using agb with varying levels of support.
Perhaps the simplest is to produce sprites or background tiles that contain text that you show using the usual means.
This technique can get you very far!
For instance: text in title screens, options in menus, and any text in the HUD could (and as we will discuss, should) all be pre-rendered.
For detail on how to do these, see the [backgrounds](./backgrounds.md) or [objects](./objects_deep_dive.md) articles.

What we will discuss here is *dynamic* rendering of text. Where the text can be decided at runtime.

# Agb text rendering principles

The text rendering system in agb has support for:
* Unicode support
* Variable size letters
* Kerning
* Left, right, centre, and justified alignments
* Left to right text only

Maybe for this reason, text rendering on the GBA is slow.
Even just laying the text out, deciding where to put each character, is slow.
For this reason, the API for text rendering is designed to spread work over multiple frames as much as possible.
This naturally results in the effect of only adding a couple characters each frame that is so common in games even today.
If you instead want to display text instantly, consider pre-rendering it.

The text rendering system is split into the layout and the backend renderers.
The `Layout` is an iterator of `LetterGroup` which stores a group of characters and where they should be displayed providing a `pixels` iterator which gives an iterator over all the pixels to render.
This includes any detail around kerning, alignment, etc.

With these letter groups, you can pass them to the Object or Tile based renderers.
The tile based renderer takes a reference to a regular background and displays the letters given by the group on it.
The object based render takes the letter group and gives back an object that represents that letter group.

# Font

Importing a font is done using the [`include_font`](https://docs.rs/agb/latest/agb/macro.include_font.html) macro.
This takes a path to a ttf and the font size to use to import. For example:

```rust
static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);
```

If you have created your own pixel font, you can convert it to ttf using [YAL's Pixel Font Converter!](https://yal.cc/tools/pixel-font/)
This tool lets you define a font from an image including variable sized letters and kerning pairs.
It also lets you export the settings which we encourage you to keep in version control.

# Layout

The [`Layout`](https://docs.rs/agb/latest/agb/display/font/struct.Layout.html) is an `Iterator` over [`LetterGroup`](https://docs.rs/agb/latest/agb/display/font/struct.LetterGroup.html)s.
A `LetterGroup` is a set of letters to be drawn at once.
The `Layout` handles correctly positioning the letter groups including performing line breaks where required and correctly aligning the text.

```rust
let mut text_layout = Layout::new(
    "Hello, this is some text that I want to display!",
    &FONT,
    AlignmentKind::Left,
    32,
    200,
);
```

