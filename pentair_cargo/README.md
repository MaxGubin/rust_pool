Rust port of https://github.com/tagyoureit/nodejs-poolController


[Protocol documentation](https://docs.google.com/document/d/1M0KMfXfvbszKeqzu6MUF_7yM6KDHk8cZ5nrH1_OUcAc/edit?usp=sharing)

https://github.com/michaelusner/pentair-pool-controler


# Setting up access to ports

Add the current user to the dialout group: 

```bash
sudo usermod -a -G dialout $USER
```

You will need to log out and back in for this to take effect.

# Degugging Tool

A simple tool that just sends data over serial.

Building:


```bash
cargo build --bin port_debug

```
Example of sending a correct packet over the port:

```bash
target/debug/port_debug --hex-string=FF00FFA501104886020101018
```

# Firebase

We use firebase to communicate with the Android application (keep configuration).
The project name 

Here is the command line to setip npm function tools:
```bash

```


# Build on Raspberry Pi

It's needed to install libssl-dev

```bash
sudo apt-get install libssl-dev

```
