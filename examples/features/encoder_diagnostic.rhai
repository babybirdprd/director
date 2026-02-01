// Quick Encoder Diagnostic
// Run this to verify hardware encoding detection

let movie = new_director(100, 100, 30);

print("=== Director Hardware Encoding Diagnostic ===\n");

// Get initial auto-detect info
let info = movie.get_encoder_info();
print("Auto-detected encoder:");
print(info);
print("");

// Test all modes
let modes = ["auto", "software", "nvenc", "qsv", "amf"];

for mode in modes {
    movie.configure_encoder(mode);
    let info = movie.get_encoder_info();
    print("Mode '" + mode + "': " + info);
}

print("\n=== Expected Results on NVIDIA 1050 ===");
print("auto:    Should show h264_nvenc (NVIDIA NVENC) - Hardware");
print("nvenc:   Should show h264_nvenc (NVIDIA NVENC) - Hardware");
print("software: Should show libx264 (Software) - Software");
print("qsv:     May fail if no Intel GPU available");
print("amf:     May fail if no AMD GPU available");

print("\nIf 'auto' shows NVENC, hardware acceleration is working!");

movie
