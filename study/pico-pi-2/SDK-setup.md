```bash
mkdir -p "$HOME/pico"
git clone --recurse-submodules https://github.com/raspberrypi/pico-sdk.git "$HOME/pico/pico-sdk"

export PICO_SDK_PATH="$HOME/pico/pico-sdk"
cp "$PICO_SDK_PATH/external/pico_sdk_import.cmake" .

# Then configure explicitly for Pico 2 and build:

cmake -S . -B build \
  -DPICO_BOARD=pico2 \
  -DPICO_SDK_PATH="$PICO_SDK_PATH"

cmake --build build -j
```
