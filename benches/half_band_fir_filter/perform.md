# FIR filter performance optimisation
### Necessity
In "decimating" mode, the spectrum analyser repeatedly applies a half band FIR filter to samples as they move down "decimation" stages. The half band FIR filter is applied multiple times per frame on audio data being streamed in real time at 48,000hz (by default on a Linux system). Profiling using `samply` revealed a significant amount of processor time is spent within the half band filter function. Optimising the FIR filter kernel provides value in reduction of application latency and system load and complements other performance decisions like precomputing FIR filter tap values, and using decimation to keep FFT size small.

### Suitability
With precomputed tap values, a FIR filter suits parallelisation. Each subsample is computed by summing the products of component input samples with corresponding tap. This investigation tests parallelisation of independent multiplications (`v2` through `v5`), and of independent subsamples (`v6`). Fused multiply-add instructions (effectively performing the multiplication and the sum step for a component sample in a single instruction) are also investigated (`v1`, `v4`, `v5`), along with different approaches to summing final accumulators.

### Hypothesis

| Version | Change under test | Hypothesis |
| --- | --- | --- |
| [v0](implementations.rs#L28) | Iterator `map(sample * tap).sum()` (single accumulator) | Baseline. Single dependency chain. |
| [v1](implementations.rs#L58) | Iterator `fold(sample.mul_add(tap, accumulator)` (single accumulator) | Fewer instructions and one rounding step might make each inner iteration faster. |
| [v2](implementations.rs#L86) | Accumulator vector sized to target SIMD register width. Each accumulator using `v0` style map(). Final accumulators `.sum()`ed. | Independent accumulation should make loop vectorisation legal and break the scalar dependency chain. |
| [v3](implementations.rs#L219) | v2 with a balanced final reduction tree | A shorter reduction dependency chain may reduce the cost of using multiple accumulators. |
| [v4](implementations.rs#L130) | v2 using `mul_add` internally. Final accumulators `.sum()`ed. | Combined hypothetical performance improvement of `v1` and `v2`  |
| [v5](implementations.rs#L167) | v4 with a balanced final reduction tree | Combined hypothetical performance improvement of `v1` and `v3` |
| [v6](implementations.rs#L273) | Calculate four output subsamples concurrently | Independent FIR outputs may provide additional instruction-level parallelism (ILP). |

### Methodology
`Divan` was used to benchmark the FIR filter implementations running over 5000 iterations 50 times. The filters were tested in "decimation stages" as this is the best reflection of the real application usage.

All code is available:
- Filter implementations are in [`implementations.rs`](implementations.rs)
- The benchmark structure is in [`runner.rs`](runner.rs)
- The commands used to run benchmarks are in [`benchcommands.sh`](benchcommands.sh)
- Raw benchmark output is in [`results1.txt`](results1.txt) and [`results2.txt`](results2.txt)
<br/>

2 representative FIR filter sizes were used:
|taps|samples|decimations|
|-|-|-|
|131|1024|7|
|51|512|7|

<br/>
  
#### Environment
##### Rust
- Rust 1.97.0
- LLVM 22.1.6
##### Hardware
- Intel Core Ultra 7 265
  - `taskset -c 0;` (CPU 0 is a P-core)
- 32GB DDR5 6000 MT/s CMK32GX5M2E6000Z36
##### OS
- Fedora Linux 44 (Server Edition) x86_64
- Linux 7.1.3-201.fc44.x86_64


#### Notes:
1. Taps are precomputed, input and output space is preallocated, this reflects real usage and ensures benchmarks are testing core filter performance.
2. Odd tap counts are required to centre the FIR window on one sample. When SIMD instructions are being targetted, the tap vector is padded with 0s up to the nearest multiple of the SIMD width. This padding is implemented in the benchmarks as it reflects real FIR filter usage.
3. Inline attributes were set to `never` for assemply analysis.


### Results

##### Native build benchmark results
| Version | 51 taps, run 1 | 51 taps, run 2 | vs v0 | 131 taps, run 1 | 131 taps, run 2 | vs v0 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| v0 | 5.060 µs | 5.032 µs | 1.00x | 32.560 µs | 32.390 µs | 1.00x |
| v1 | 8.058 µs | 8.073 µs | 0.63x | 57.500 µs | 57.120 µs | 0.57x |
| v2 | 1.205 µs | 1.207 µs | 4.18x | 4.633 µs | 4.683 µs | **6.97x** |
| v3 | 1.146 µs | 1.130 µs | 4.43x | 4.481 µs | 4.883 µs | 6.94x |
| v4 | 1.054 µs | 1.053 µs | 4.79x | 4.768 µs | 4.772 µs | 6.81x |
| v5 | 1.043 µs | 1.051 µs | **4.82x** | 4.812 µs | 4.939 µs | 6.66x |
| v6 | 4.029 µs | 4.066 µs | 1.25x | 17.950 µs | 17.820 µs | 1.82x |

Most results reproduced closely. Within a single experiment, the average fastest sample was 2.1% below the median, whilst the slowest was 3.2% above it. Every mean lies within 1.6% of the median. 65 out of the 72 median benchmark times lie within 3% of each other between run 1 and run 2. v3 in the above native build results is a conspicuous outlier to this. Given the variance in the results, there is no single optimal approach, but there are real improvements and some conclusions can be made about the optimisation techniques used. 

##### SIMD instructions for tap-sample multiplication

The clearest improvement is encouraging parallelisation of tap-sample multiplication using a SIMD width array of accumulators. Since the baseline for x86_64 includes SSE2, parallel accumulation in 4 lanes is always possible. Benchmarking with `-C target-cpu=x86-64` and no `target-feature`s demonstrates SIMD optimised `v2` and `v3` each performing around 3.6 and 4.6 times faster for 51 and 131 tap filters respectively. Looking at the generated assembly confirms the SIMD "encouragment" is working as we see `v2` and `v3` compile to SSE packed single (f32) instructions whilst `v0` uses scalar single instructions:
|`v2` and `v3`|`v0`|
|-|-|
| <code>movups xmm2, xmmword ptr [rbp + rcx - 16]&#10;mulps xmm2, xmm0&#10;addps xmm2, xmm3</code>|<code>movss xmm0, dword ptr [r13 + 4*rax - 12]&#10;mulss xmm0, dword ptr [rbp + 4*rax]&#10;addss xmm0, xmm4</code>|

The 131 tap result is particuarly notable as we see a consistent 4.6x performance improvement with only 4 parallel SIMD lanes, despite the overhead of seperate accumulators which need a final reduction.

This performance improvement increases further when targetting AVX and AVX2, with an average speedup of 4.4x and 6.9x for 51 tap and 131 tap benchmarks. The generated assembly is consistent with the performance improvement. In `v2` and `v3` we see AVX the equivalents of SSE instructions: `vmovups`, `vmulps` and `vaddps`, using 256 bit `ymm` registers, where SSE used 128 bit `xmm`.


##### Fused Multiply-Add 
The primary benefit of using a fused multiply-add (FMA) instruction is increased accuracy. Only a single floating point round is required in the fused operation, where 2 rounds occur when multiplication and addition are performed seperately. This accuracy improvement is not visible at the resolution of the spectrum analyser TUI display.

When FMA is targetted, the generated assembly confirms `v4` and `v5` retain the SIMD parallelisation from `v2` and `v3`, whilst replacing the seperate multiply and add instructions with a packed fused multiply-add instruction:
|`v4` and `v5` in FMA builds|`v2` and `v3` in the same builds|
|-|-|
| <code>vmovups ymm0, ymmword ptr [r13 + rcx - 96]&#10;vfmadd132ps ymm0, ymm3, ymmword ptr [rbp + rcx - 96]</code>|<code>vmovups ymm0, ymmword ptr [r14 + rcx - 32]&#10;vmulps ymm0, ymm0, ymmword ptr [rbp + rcx - 32]&#10;vaddps ymm0, ymm0, ymm2</code>|

The use of 256 bit `ymm` registers confirms both approaches operate on 8 f32 values in parallel. `v2` and `v3` remain seperate `vmulps` and `vaddps` instructions whilst `v4` and `v5` perform the same arithmetic using a single `vfmadd132ps` or `vfmadd231ps` instruction.

Results show builds targetting hardware FMA have an average improvment of 11.7% for 51 tap benchmarks of `v3` and `v4` over `v1` and `v2`. For 131 tap benchmarks, this reverses and there is instead on average performance deterioration of around 3.23%, suggesting some fixed or nonlinear cost is reduced using FMA, resulting in better performance with lower tap counts, whilst processor time _per iteration_ is higher. This may be explained by each FMA instruction's dependency on its accumulator's prior result, whilst multiplication instructions are independent of prior results. `VADDPS` has lower latency ([2 cycles](https://www.uops.info/html-instr/VADDPS_YMM_YMM_YMM.html)) than `VMFADDXXXPS` ([4 cycles](https://uops.info/html-instr/VFMADD132PS_YMM_YMM_YMM.html)) meaning overlapping addition and multiplication instructions with a faster dependent instruction likely make better use of available execution units and increase throughput versus a single slower FMA instruction. I was unable to identify a source of the improved constant/nonlinear cost reducing execution time for FMA implementations processing 51 taps. A future benchmark with a high number of samples would strengthen the "less overhead but slower per iteration" theory.

Implementations of the filter using `mul_add` perform extremely poorly compared to their `map(...).sum()` counterparts when compiling for targets that do not support a fused multiply add instruction. The `v1` benchmarks take on average 5.9 times longer than `v0` on these targets, whilst SIMD optimised implementations `v4` and `v5` take on average 34 times longer than their counterpants `v2` and `v3`. Analysing the compiled assembly reveals the issue. `mul_add` computes `(self * a) + b` "**with only one rounding error**". When this is not available in hardware, the glibc [`fmaf`](https://sourceware.org/glibc/manual/2.33/html_node/Misc-FP-Arithmetic.html) function is called for each iteration in the hot loop. The documentation for this function states "fma can be very slow since it must avoid intermediate rounding". Even if glibc fmaf is using hardware fma under the hood, the function overhead and shuffling results in poor performance. As `fmaf` is a scalar function, the SIMD optimised implementations using `mul_add` that are compiled for non-fma targets no longer use any SIMD instructions, which is the source of their particularly bad slowdown.

##### Final accumulator reduction
The balanced reduction tree used by `v3` and `v5` provides a smaller, target-dependent improvement over the linear sum in `v2` and `v4`. It is most useful for the 51 tap AVX2 builds, where it reduces execution time by up to 11%; the improvement is generally smaller for 131 taps because the final reduction forms less of the total work. Native results range from a 5.6% improvement (`v3`, 51 taps) to small regressions for 131 taps, so the reduction strategy is secondary to enabling SIMD parallelisation and should not be expected to improve every build.
