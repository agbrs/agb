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

You can find pixel fonts with many under permissive licenses on the [https://www.pentacom.jp/pentacom/bitfontmaker2/gallery/](bitfontmaker2)'s website.

# Layout

The [`Layout`](https://docs.rs/agb/latest/agb/display/font/struct.Layout.html) is an `Iterator` over [`LetterGroup`](https://docs.rs/agb/latest/agb/display/font/struct.LetterGroup.html)s.
A `LetterGroup` is a set of letters to be drawn at once.
The `Layout` handles correctly positioning the letter groups including performing line breaks where required and correctly aligning the text.
It does this incrementally, doing as little work as possible to generate the next groups position.


```rust
let text_layout = Layout::new(
    "Hello, this is some text that I want to display!",
    &FONT,
    AlignmentKind::Left,
    32,
    200,
);
```

## Palette changes

To have multiple colours in your text, you can use [`ChangeColour`](https://docs.rs/agb/latest/agb/display/font/struct.ChangeColour.html).

```rust
const COLOUR_1: ChangeColour = ChangeColour::new(1);
const COLOUR_2: ChangeColour = ChangeColour::new(2);

let text = format!("Hey, {COLOUR_2}you{COLOUR_1}!",);
```

You might want to use static text rather than using Rust's text formatting, in that case see the documentation for [`ChangeColour`](https://docs.rs/agb/latest/agb/display/font/struct.ChangeColour.html) where it documents the exact code points you need to use.

## Tags

You might want to treat certain parts of your text differently to other parts.
Maybe some text should wiggle around, maybe some text should be delayed in the time taken to display it.
You can encode this user state using the tag system.

The tag system gives 16 user controllable bits that you can set and unset during processing of text.
Here's a simple example that shows how this works with `LetterGroup`s.

```rust
const MY_TAG: Tag = Tag::new(0);
// set the tag with `set` and unset with `unset`.
let text = alloc::format!("#{}!{}?", MY_TAG.set(), MY_TAG.unset());
let mut layout = Layout::new(&text, &FONT, AlignmentKind::Left, 32, 100);

// get whether the tag is set with `has_tag` on `LetterGroup`.
assert!(!layout.next().unwrap().has_tag(MY_TAG));
assert!(layout.next().unwrap().has_tag(MY_TAG));
assert!(!layout.next().unwrap().has_tag(MY_TAG));
```

which can be extended within your text display system.
A complete example of this can be seen in the [advanced object text rendering example](https://agbrs.dev/examples/object_text_render_advanced).
If you want to use Tags without using Rust's text formatting, the documentation for [`Tag`]((https://docs.rs/agb/latest/agb/display/font/struct.Tag.html)) documents the exact code points you need to use.

# Renderers

The groups that come from the `Layout` can be used in the render backends.
There is a backend for displaying text using `Object`s and another for using background tiles.

## ObjectTextRenderer

The `ObjectTextRenderer` takes in a `LetterGroup` and gives back an `Object` that represents that group.
To create one, you need to provide a palette and the size of sprites to use.
It is important that the size of sprite is greater than or equal to the maximum group size that is specified in the `Layout`.

A simple example of the `ObjectTextRender` would look like
```rust
let text_layout = Layout::new(
    "Hello, this is some text that I want to display!",
    &FONT,
    AlignmentKind::Left,
    16, // minimum group size is 16, so the sprite size I use should be at least 16 wide
    200,
);

// using an appropriate sprite size, palette should come from somewhere
let text_render = ObjectTextRenderer::new(PALETTE.into(), Size::S16x16);
let objects: Vec<_> = text_layout.map(|x| text_render.show(&x, vec2(16, 16))).collect();

// then show the objects in the usual way
```
The full example can be found in the [`object_text_render_simple`](https://agbrs.dev/examples/object_text_render_simple) example.

One of the main reasons to use objects for your text is to be able to individually manipulate your objects to create special effects.
The [`object_text_render_advanced`](https://agbrs.dev/examples/object_text_render_advanced) example showcases this use case.

## RegularBackgroundTextRenderer

