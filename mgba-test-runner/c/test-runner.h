#pragma once

#include <stdint.h>

struct MGBA;

struct video_buffer {
    uint32_t width;
    uint32_t height;
    uint32_t* buffer;
};

struct MGBA* new_runner(char filename[]);
void free_runner(struct MGBA* mgba);
void set_logger(void logger(char[]));
void advance_frame(struct MGBA* mgba);
struct video_buffer get_video_buffer(struct MGBA* mgba);