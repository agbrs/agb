#include "test-runner.h"

#include <mgba/core/core.h>
#include <mgba/feature/commandline.h>
#include <stdio.h>

_Static_assert(BYTES_PER_PIXEL == 4, "bytes per pixel MUST be four");

void log_output(struct mLogger* _log, int category, enum mLogLevel level,
                const char* format, va_list args);
char* log_level_str(enum mLogLevel);

static void (*EXTERNAL_LOGGING)(char*);

struct mLogger LOGGER = {.log = log_output};

struct MGBA {
    struct mCore* core;
    struct video_buffer videoBuffer;
    char* filename;
};

struct MGBA* new_runner(char* filename) {
    struct MGBA* mgba = malloc(sizeof(struct MGBA));

    mLogSetDefaultLogger(&LOGGER);

    char* filename_new = strdup(filename);
    mgba->filename = filename_new;

    struct mCore* core = mCoreFind(mgba->filename);
    if (!core) {
        printf("failed to find core\n");
        return NULL;
    }

    core->init(core);

    unsigned width, height;
    core->desiredVideoDimensions(core, &width, &height);
    ssize_t videoBufferSize = width * height * BYTES_PER_PIXEL;

    uint32_t* videoBuffer = malloc(videoBufferSize * sizeof(*videoBuffer));

    core->setVideoBuffer(core, videoBuffer, width);

    // load rom
    mCoreLoadFile(core, mgba->filename);

    mCoreConfigInit(&core->config, NULL);

    core->reset(core);

    mgba->core = core;
    mgba->videoBuffer = (struct video_buffer){
        .buffer = videoBuffer,
        .width = width,
        .height = height,
    };

    return mgba;
}

void set_logger(void logger(char*)) { EXTERNAL_LOGGING = logger; }

void free_runner(struct MGBA* mgba) {
    mgba->core->deinit(mgba->core);
    free(mgba->filename);
    free(mgba->videoBuffer.buffer);
    free(mgba);
}

void advance_frame(struct MGBA* mgba) { mgba->core->runFrame(mgba->core); }

struct video_buffer get_video_buffer(struct MGBA* mgba) {
    return mgba->videoBuffer;
}

void log_output(struct mLogger* _log, int category, enum mLogLevel level,
                const char* format, va_list args) {
    UNUSED(_log);
    if (level & 31) {
        int32_t size = 0;

        size += snprintf(NULL, 0, "[%s] %s: ", log_level_str(level),
                         mLogCategoryName(category));
        va_list args_copy;
        va_copy(args_copy, args);
        size += vsnprintf(NULL, 0, format, args_copy);
        va_end(args_copy);
        size += 1;

        char* str = calloc(size, sizeof(*str));

        int32_t offset = snprintf(str, size, "[%s] %s: ", log_level_str(level),
                                  mLogCategoryName(category));
        size -= offset;
        vsnprintf(&str[offset], size, format, args);

        if (EXTERNAL_LOGGING != NULL) {
            EXTERNAL_LOGGING(str);
        } else {
            printf("%s\n", str);
        }
        free(str);
    }
}

char* log_level_str(enum mLogLevel level) {
    switch (level) {
        case mLOG_FATAL:
            return "FATAL";
            break;
        case mLOG_ERROR:
            return "ERROR";
            break;
        case mLOG_WARN:
            return "WARNING";
            break;
        case mLOG_INFO:
            return "INFO";
            break;
        case mLOG_DEBUG:
            return "DEBUG";
            break;
        case mLOG_STUB:
            return "STUB";
            break;
        case mLOG_GAME_ERROR:
            return "GAME ERROR";
            break;
        default:
            return "Unknown";
    }
}