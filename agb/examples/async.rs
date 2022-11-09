#![no_std]
#![no_main]

use agb::{executor, println};

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
fn main(gba: agb::Gba) -> ! {
    executor::async_main(gba, |_gba| async move {
        let a = executor::spawn(count_frames("A"));

        let wait = executor::spawn(wait_for_n_frames(10));

        let value = executor::spawn(get_value());

        executor::spawn(async {
            wait.await;
            agb::println!("waited for 10 frames!");
            a.abort();
        });

        executor::spawn(async {
            let value = value.await;
            agb::println!("The value was {}", value);
        });

        let (mut reader, mut writer) = executor::channel::new_with_capacity(16);

        executor::spawn(async move {
            executor::vblank_async().await;
            for i in 0..256 {
                let _ = writer.write(i);
                executor::yeild().await;
            }
        });

        executor::spawn(async move {
            let start = executor::CURRENT_VBLANK.read();
            while (reader.read().await).is_ok() {}
            let end = executor::CURRENT_VBLANK.read();
            agb::println!(
                "Writer associated with this reader has closed, took {} frames to read all",
                end - start
            );
        });
    });
}
