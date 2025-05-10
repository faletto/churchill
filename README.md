# Churchill, a DHCP Server's Worst Nightmare
Churchill is a cross-platform command-line tool written in Rust meant to generate DHCP Discover messages from random source MAC addresses. This results in what's called a DHCP Starvation attack, where a DHCP server runs out of IP addresses and can no longer accomodate new clients.

## Requirements

## Usage 
```
usage: churchill [OPTIONS]
```

## Options:
```
Options:
  -n, --number <NUMBER>    Number of packets to send (0 = unlimited) [default: 0]
  -d, --delay <DELAY>      Delay between packets in milliseconds (defaults to instantaneous) [default: 0]
  -a, --address <ADDRESS>  Address to send DHCP packets on (if a valid address isn't specified, a list of interfaces/addresses will be shown) [default: 0.0.0.0]
  -l, --list               List available network interfaces/addresses
  -h, --help               Print help
  -V, --version            Print version
```
Example:
```
churchill -a 127.0.0.1 -n 256 -d 10
```
Sends 256 packets on the address **127.0.0.1** (the loopback interface) with a delay of **10ms** between each packet.

## Disclaimer
This program was designed for cybersecurity research purposes. Do not use this tool on any network unless you have the explicit permission from the network owner. This program serves as a demonstration for how specific exploits can operate and as a learning tool for cybersecurity, and is not intended for any malicious purpose.