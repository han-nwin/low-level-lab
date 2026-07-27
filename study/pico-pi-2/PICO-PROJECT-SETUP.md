# Raspberry Pi Pico CMake Project Setup

Starting with only `blinky.c`, each Pico project needs:

* A small `CMakeLists.txt`
* The Pico SDK import file
* One CMake configure command

## 1. Create `CMakeLists.txt`

```cmake

# cmake version
cmake_minimum_required(VERSION 3.13)

# include the sdk.cmake file
include(pico_sdk_import.cmake)

# give the project a name (anything you want)
project(blinky LANGUAGES C CXX ASM)

# set the C/C++ language versions
set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# initialize the sdk
pico_sdk_init()

add_executable(Blinky)

target_sources(Blinky PRIVATE blinky.c)

# Pull in our pico_stdlib which pulls in commonly used features
target_link_libraries(Blinky pico_stdlib pico_bootsel_via_double_reset)


# create map/bin/hex file etc.
pico_add_extra_outputs(Blinky)
```

## 2. Install and Copy the Pico SDK Import File
```bash
mkdir -p "$HOME/pico"

git clone --recurse-submodules \
    https://github.com/raspberrypi/pico-sdk.git \
    "$HOME/pico/pico-sdk"

export PICO_SDK_PATH="$HOME/pico/pico-sdk"
```

From the new project directory:

```bash
cp /Users/han/pico/pico-sdk/external/pico_sdk_import.cmake .
```

## 3. Download Toolchains and Configure the Project

### Install the Arm GNU Toolchain

Official installation guide:

[Arm GNU Toolchain Installation Guide](https://learn.arm.com/install-guides/gcc/arm-gnu/)

Everyone must choose a package ending in:

```text
arm-none-eabi
```

The beginning of the package name identifies the host machine.

| Host machine        | Package name contains                |
| ------------------- | ------------------------------------ |
| macOS Apple Silicon | `darwin-arm64-arm-none-eabi.pkg`     |
| macOS Intel         | `darwin-x86_64-arm-none-eabi`        |
| Linux x86-64        | `x86_64-arm-none-eabi.tar.xz`        |
| Linux ARM64         | `aarch64-arm-none-eabi.tar.xz`       |
| Windows x64         | `mingw-w64-x86_64-arm-none-eabi.msi` |

> Do not choose `aarch64-none-elf`. The Raspberry Pi Pico requires the `arm-none-eabi` target.

Arm recommends this toolchain for bare-metal embedded targets.

### 1. Determine Your Host OS and Architecture

#### macOS or Linux

```bash
uname -s
uname -m
```

#### Windows PowerShell

```powershell
$env:PROCESSOR_ARCHITECTURE
```

### 2. Download the Toolchain

Download the Arm GNU Toolchain from:

https://learn.arm.com/install-guides/gcc/arm-gnu/

### 3. Select the Correct Package

Choose the download for your host machine whose filename ends in:

```text
arm-none-eabi
```

Examples:

* Apple Silicon macOS: `darwin-arm64-arm-none-eabi.pkg`
* Intel macOS: `darwin-x86_64-arm-none-eabi`
* x86-64 Linux: `x86_64-arm-none-eabi.tar.xz`
* ARM64 Linux: `aarch64-arm-none-eabi.tar.xz`
* x86-64 Windows: `mingw-w64-x86_64-arm-none-eabi.msi`

### 4. Add the Toolchain to `PATH`

Add the toolchain's `bin` directory to your system's `PATH`.

For example, on macOS or Linux:

```bash
export PATH="/path/to/arm-gnu-toolchain/bin:$PATH"
```

To make the change permanent, add the command to your shell configuration file, such as:

```text
~/.zshrc
```

or:

```text
~/.bashrc
```

### 5. Verify the Installation

```bash
arm-none-eabi-gcc --version
```

If the toolchain is installed correctly, this command will print the compiler version.

### 6. Locate the Compiler

#### macOS or Linux

```bash
command -v arm-none-eabi-gcc
```

#### Windows

```powershell
where arm-none-eabi-gcc
```

## Run first cmake

```bash
  cmake -S . -B build-pico2-w \
    -DPICO_BOARD=pico2_w \
    -DPICO_TOOLCHAIN_PATH=/Applications/ArmGNUToolchain/15.2.rel1/arm-none-eabi \
    -DCMAKE_EXPORT_COMPILE_COMMANDS=ON
```


  To avoid typing the toolchain path every time, add this to ~/.zshrc or .bashrc et.c:

```bash

  export PICO_TOOLCHAIN_PATH=/Applications/ArmGNUToolchain/15.2.rel1/arm-none-eabi
```

## 4. Create the Compilation Database Symlink

This allows `clangd` and Neovim to find the generated compilation commands:

```bash
ln -sfn build-pico2-w/compile_commands.json compile_commands.json
```

## 5. Build the Project

```bash
cmake --build build --parallel
```

## Generated Files

After building, you will get:

```text
build/blinky.elf
build/blinky.bin
build/blinky.hex
build/blinky.uf2
```

The `.uf2` file is normally the file copied onto the Pico when it is in bootloader mode.

## 6. Open the Project in Neovim

```bash
nvim blinky.c
```

# Repeatable Pico Project Workflow

For every Pico project:

```bash
cp pico_sdk_import.cmake .

cmake -S . -B build \
  -DPICO_BOARD=pico2_w \
  -DPICO_TOOLCHAIN_PATH=/Applications/ArmGNUToolchain/15.2.rel1/arm-none-eabi \
  -DCMAKE_EXPORT_COMPILE_COMMANDS=ON

ln -sfn build/compile_commands.json compile_commands.json

cmake --build build
```

# Configure the Environment Variables

To avoid typing the Pico SDK and toolchain paths each time, add the following to `~/.zshrc`:

```bash
export PICO_SDK_PATH=/Users/han/pico/pico-sdk
export PICO_TOOLCHAIN_PATH=/Applications/ArmGNUToolchain/15.2.rel1/arm-none-eabi
```

Reload the shell configuration:

```bash
source ~/.zshrc
```

After that, the configure command becomes:

```bash
cmake -S . -B build
```

# How the Pieces Work Together

* `CMakeLists.txt` defines the project, source files, libraries, and C/C++ language versions.
* `pico_sdk_import.cmake` locates and imports the Raspberry Pi Pico SDK.
* `CMAKE_EXPORT_COMPILE_COMMANDS` generates `compile_commands.json`.
* `compile_commands.json` tells `clangd` which compiler flags, include directories, and definitions the project uses.
* The `compile_commands.json` symlink allows Neovim and `clangd` to find the compilation database from the project root.
* The global `~/.clangd` configuration can remain minimal because the project-specific compiler configuration comes from CMake.
