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



