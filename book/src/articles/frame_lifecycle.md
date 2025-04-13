# Frame lifecycle

Games written using `agb` are typically follow the ['update-render loop'](https://gameprogrammingpatterns.com/game-loop.html).
The way your components update will be very dependent on the game you are writing, but each frame you would normally do the following:

```rust
loop {
    my_game.update();

    let mut frame = gfx.frame();
    my_game.show(&mut frame);
    frame.commit();
}
```

This document is goes into detail about the correct usage of the [`GraphicsFrame`](https://docs.rs/agb/latest/agb/display/GraphicsFrame.html)
(the `frame` variable you see above) and how to make the most of it.
Further articles (e.g. [Blending, windows and DMA]()) will go into more detail about other effects you can apply once you've mastered the content of this article.

## `.show(frame: &mut GraphicsFrame)`

The most common pattern involving `GraphicsFrame` you'll see in the `agb` library is a `.show()` method which typically accepts a mutable
reference to a `GraphicsFrame`.
Due to this naming convention, it is also conventional in games written using `agb` to name the `render` method `show()`.

As an example, you could have your player object as follows:

```rust
use agb::{
    fixnum::{Num, Vector2D},
    display::{GraphicsFrame, object::Object}
};

struct Player {
    sprite: Object,
    world_position: Vector2D<Num<i32, 4>>,
}

impl Player {
    // ...
    pub fn update(&mut self, camera_position: Vector2D<Num<i32, 4>>) {
        // move the player based on input and the world position. In a real game,
        // the update method may take a `ButtonController` or some other struct
        // you've built around that to control the players position based on
        // buttons pressed.

        // Remember that the objects's position here is relative to the top left
        // of the screen, so the update function should ensure that it is
        // translated to be relative to the camera position.
        self.sprite.set_position((self.world_position - camera_position).floor());
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        self.sprite.show(frame);
    }
}
```
