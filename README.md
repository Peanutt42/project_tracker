# Project Tracker
personal project todo list tracker

![](Screenshot.png)

## Platform support
- Linux, Windows, Macos (untested, but should work)
- Installation scripts are linux only for now
- Server can run on Raspberrypi! (GUI has weird artifacts...)

## Prerequisites
- Rust (programing lanuage): https://www.rust-lang.org/tools/install

## Installation - Project Tracker GUI (also Client) (Linux)
```bash
cd scripts
./install_linux.sh
```

## Installation - Project Tracker Server (Linux)
Headless:
```bash
cd scripts
./install_server_linux.sh
```
With dashboard ui:
```bash
cd scripts
./install_server_with_dashboard_linux.sh
```