
Add changes you make to the changelog including tests you run.

After making changes to Rust source in ktop-rs/, rebuild with:
  cd ktop-rs && cargo build --release
Then copy the binary to install locally:
  cp ktop-rs/target/release/ktop ~/.local/bin/ktop

**CRITICALLY IMPORTANT INSTRUCTIONS BELOW THIS LINE**

**You must NEVER PUSH TO GITHUB without explicitly asking the user!**

## Changelog

### v1.0.5
- Add footer power estimate segment before OOM status when live sensors are available
- Estimate uses CPU package power from Linux powercap/hwmon plus NVIDIA NVML and AMD hwmon GPU power
- Hide the power field when the host exposes no usable power sensors instead of showing a fake value
- Tested: `cargo build --release`

### v1.0.4
- Switch from musl static linking to glibc dynamic linking for NVIDIA GPU compatibility
- musl binaries can't dlopen glibc-linked .so files (like libnvidia-ml.so), causing GPU detection to silently fail
- CI now uses `cross` tool to build in older-glibc Docker containers for portability
- install.sh updated to fetch gnu-targeted binaries
- Tested: local build, version check, ldd confirms dynamic linking
