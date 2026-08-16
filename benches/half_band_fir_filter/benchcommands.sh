# baseline x86_64: SSE2 128 bit SIMD, 4 parallel f32s
RUSTFLAGS="-Awarnings -C target-cpu=x86-64 -C target-feature=+sse2" taskset -c 0 cargo bench -q

# AVX 256 bit SIMD, 8 parallel f32s
RUSTFLAGS="-Awarnings -C target-cpu=x86-64 -C target-feature=+avx" taskset -c 0 cargo bench -q
# AVX with hardware FMA
RUSTFLAGS="-Awarnings -C target-cpu=x86-64 -C target-feature=+avx,+fma" taskset -c 0 cargo bench -q

# AVX2
RUSTFLAGS="-Awarnings -C target-cpu=x86-64 -C target-feature=+avx2" taskset -c 0 cargo bench -q
# AVX2 with hardware FMA
RUSTFLAGS="-Awarnings -C target-cpu=x86-64 -C target-feature=+avx2,+fma" taskset -c 0 cargo bench -q

# native build
RUSTFLAGS="-Awarnings -C target-cpu=native" taskset -c 0 cargo bench -q
