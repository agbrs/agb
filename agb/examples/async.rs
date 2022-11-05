#![no_std]
#![no_main]

use agb::{
    executor::{self, Executor},
    println,
};

async fn count_frames(task_id: &str) {
    let mut count = 0;
    loop {
        executor::vblank_async().await;
        count += 1;
        println!("Task {} at count {}", task_id, &count);
    }
}

async fn wait_for_n_frames(n: usize) {
    for _ in 0..n {
        executor::vblank_async().await;
    }
}

async fn get_value() -> u32 {
    42
}

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    executor::spawn(count_frames("A"));
    executor::spawn(count_frames("B"));

    let wait = executor::spawn(wait_for_n_frames(10));

    let value = executor::spawn(get_value());

    executor::spawn(async {
        wait.await;
        agb::println!("waited for 10 frames!");
        executor::spawn(count_frames("C"));
    });

    executor::spawn(async {
        let value = value.await;
        agb::println!("The value was {}", value);
    });

    let mut executor = unsafe { Executor::new() };

    executor.run();

    unreachable!();
}
