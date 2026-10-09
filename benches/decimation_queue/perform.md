# Queue implementation performance optimisation

DSP in this project uses circular queues to store samples during processing. These benchmarks test the performance of different implementations of queues with the FIR filter decimation component of the spectrum analyser.

### Hypothesis

| Version | Change under test | Hypothesis |
| --- | --- | --- |
| [v0]() | VecDequeue `make_contiguous` | Baseline, standard library vecdequeue, made contiguous befor FIR filter |
| [v1]() | VecDequeue, FIR filter handles queue wraparound | standard library vecdeque, FIR filter written to handle 2 sources internally |
| [v3]() | Custom double memory mapped queue | Double memory mapped queue. Negates need for O(n) make contiguous or additional internal wraparound logic. |
| [v4]() | Custom double memory mapped queue without internal wrap checking | Double memory mapped queue. Only updates internal head and tail pointers when requested by caller. Saves a branch condition on every push. |


### Methodology
`Divan` was used to benchmark the queue implementations for 500 iterations 50 times. The queues were benchmarked via use as source and target circular buffers for an optimised FIR filter across multiple "decimation stages".  as this is the best reflection of the real application usage.

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
