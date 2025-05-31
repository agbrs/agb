# Using a debugger with VSCode

`agb::println` can get you quite far with debugging, but sometimes you will want a debugger.
VSCode and mGBA can work together to provide a debugger experience.
The template is configured for this by default on linux.
You'll need the recommended extensions along with the debugger `arm-none-eabi-gdb` installed.
Pressing `F5` will start the game with the debugger attached to it.
You can then add breakpoints to the code and step through.
This can, however, be rather difficult as we need to enable optimisations to have passable performance which causes vast areas of code to be optimised out and generally be reordered.

This works because of the `launch.json` and `tasks.json` files.
The `tasks.json` file specifies how to build the game, which is simply by calling `cargo build`.
While the `launch.json` specifies exactly how to launch the game and configure the debugger.

> The `launch.json` file specifies the name of the binary directly, if you change the name of your crate you will need to change the names of `agb_template` in the `launch.json` file.

# Recommended extensions

There are two recommended extensions, `rust-analyzer` for a language server for Rust, and `cpptools` the C/C++ extension.
We want the C/C++ extension because it is what contains the support needed for launching the debugger.

# `tasks.json`

We define a task to build the game in debug mode.
We explicitly state the target dir here because otherwise specifying your own would break things in the `launch.json`.

```json
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Rust build: debug",
            "command": "cargo",
            "args": [
                "build"
            ],
            "options": {
                "cwd": "${workspaceFolder}",
                "env": {
                    "CARGO_TARGET_DIR": "${workspaceFolder}/target"
                }
            }
        },
    ],
}
```

# `launch.json`

Here we use the C/C++ extension to define a debugger that runs the game using mGBA with a gdb server (the `-g` option) and attaches `arm-none-eabi-gdb`.
If you do get this working for other platforms, please reach out to us so we can include the configuration in the template by default.
You'll see in here we refer to the name of the crate directly, `agb_template`, so if you change the name of the crate you'll need to change it in here too.

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "(gdb) Launch",
            "type": "cppdbg",
            "request": "launch",
            "targetArchitecture": "arm",
            "args": [],
            "stopAtEntry": false,
            "environment": [
                {
                    "name": "CARGO_TARGET_DIR",
                    "value": "${workspaceFolder}/target",
                },
            ],
            "externalConsole": false,
            "MIMode": "gdb",
            "miDebuggerServerAddress": "localhost:2345",
            "preLaunchTask": "Rust build: debug",
            "program": "${workspaceFolder}/target/thumbv4t-none-eabi/debug/agb_template",
            "cwd": "${workspaceFolder}",
            "linux": {
                "miDebuggerPath": "arm-none-eabi-gdb",
                "setupCommands": [
                    {
                        "text": "shell \"mgba-qt\" -g \"${workspaceFolder}/target/thumbv4t-none-eabi/debug/agb_template\" &"
                    }
                ]
            },
        },
    ],
}
```