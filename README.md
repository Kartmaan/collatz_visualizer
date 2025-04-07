# Collatz Conjecture Visualization Application

This application allows you to graphically visualize the sequences of the Collatz Conjecture (also known as the Syracuse Conjecture) for one or two given integers.

## The Collatz Conjecture

The Collatz Conjecture is defined by the following rule:
- If n is even, the next term is n/2
- If n is odd, the next term is 3n+1

The sequence stops when the value 1 is reached.

## Screenshots

**Window**

![cap_win](/img/win_sc.png)

**Saved chart**

![saved_chart](/img/saved.png)

## Features

- Graphical visualization of sequences for one or two integers
- Detailed statistics display:
  - Flight time (number of steps)
  - Maximum altitude (highest value reached)
  - Count of even/odd values
  - Downtime
- Random value generation
- Save the graph image
- Copy sequences to the clipboard

## Installation

### Prerequisites

- Rust and Cargo (https://www.rust-lang.org/tools/install)

### Installation

1. Extract the zip archive
2. Open a terminal in the extracted folder
3. Run the following command to compile and launch the application:

```bash
cargo run
```

To create an optimized version:

```bash
cargo build --release
```

The executable will be located in the `target/release/` folder.

## Usage

1. Enter one or two integers in the input fields
2. Click "Visualize" to display the graph
3. Use "Randomize" to generate random values
4. Use "Save" to save the graph image
5. Use "Copy" to copy the sequences to the clipboard

## Code Structure

- `src/main.rs`: User interface and main application logic
- `src/collatz.rs`: Implementation of the Collatz Conjecture algorithm

## Dependencies

- iced: Graphical user interface
- plotters: Graph visualization
- rand: Random number generation
- clipboard: Clipboard access
- chrono: Date and time management
- image: Image manipulation

## License

This project is distributed under the MIT license.