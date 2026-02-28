# Project: Planify 🦀

Planify is a collaboration tool, developed as a fullstack application using **Rust** and the **Dioxus Framework**.

## 🛠️ Prerequisites

Since this project uses WebAssembly (WASM) for the frontend, the **Dioxus CLI** is required.

### Installing the Dioxus CLI
To install the tool, run the following command in your terminal once:

`cargo install dioxus-cli`

Detailed info: [official Dioxus installation guide](https://dioxuslabs.com/learn/0.6/getting_started/).

## Installation on Macos (might not work on every device)

The normal installation has to go through the following steps:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    rustup default 1.85.0
    curl -sSL https://dioxus.dev/install.sh | bash

Steps you might need to do in case the view is not loading correctly:
    it might occur that the ciew is looking faulty with white edges and some views after login not working correctly
    this might happen because the tailwind didnt finish building bevor the application starts, in this case you need
    to rebuild the app after serving with 'r' in the terminal or click ctrl + c and serve again

If you see that the Tailwind.css is not generated at all, it could be that you're not on the latest dioxus version