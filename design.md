- captor requirements
  - initialise
    - provide sample rate, buffer size, and channel count to controller
    -  load options 
      - channel count
      - sample rate 
      - buffer size
  - main loop
    - write new audio samples to ring buffer
      - perform averaging here to minimise buffer size for average modes
    - notify controller thread when next analysis window is ready
      - window is ready when number of new samples > sample rate / analysis sample rate

- controller requirements
  - initialise
    - take console args
    - init captor
    - verify captor options are compatible with console args
      - e.g. selected channel exists
  - maintain analysis window buffers
    - circular
    - size of analysis window size
    - seperate buffer for each channel (unless averaged)
  - potentially, different buffers for different notes (e.g. a 20hz base note will not be observable in a 1/100th of a second window )

- transformer requirements
  - perform FFT across analysis window buffer
  - output frequency spectrum
  - potentially normalise output to 0 <= x <= 1
    - this might actually be hard as it is impossible to know what audio will be played in the future
    - the GUI would be poor if playing at 20% volume meant that bars were compressed to 20% height

- console display requirements
  - iniialise
    - take number of output samples
    - build ascii table structure
  - take array of floats 0 <= x <=1
    - display bars

- console arg
  - analysis window size in number of samples
  - analysis sample rate in Hz
  - channel modes
    - average
    - display seperately
    - pick 1
  - output
    - number of output samples (how many k will be in the FFT output)
    - output
      - musical notes
      - octaves and semitons

- overall requirements
  - modularity
    - may want to swap console display for rendered GUI in the future
  - real-time processing
  - lighweight as possible

- console arg
  - analysis window size in number of samples
  - analysis sample rate in Hz
  - channel modes
    - average
    - display seperately
    - pick 1



audio samples
- start in 