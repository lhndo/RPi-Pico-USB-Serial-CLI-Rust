## Note:
* This project is in an early WIP stage and I'm using it for learning and experimenting with Rust/Embedded development

* The intent is to develop an easy prototyping workflow by using the **Raspberry Pi Pico** over **Serial USB** though a **CL** like **interface**

* The "**device**" is set up in **device.rs** and encapsulated in a **Device** struct which is then borrowed to various **CLI** **commands**/**programs** 
* **CLI commands** are implemented in **simple_cli/commands.rs**

## Setup:

* Clone this repository
* Connect the Pico in **BOOTSEL** mode and execute `cargo run` (*assuming you have the rust toolchain installed*)
* Connect to the Pico though a COM port using a **Serial Monitor tool** and follow the instructions