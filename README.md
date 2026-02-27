# Planify 🦀  
**A collaboration desktop App written in Rust and the Dioxus framework.**

Planify is a full-stack app for personal and collaborative organization. The application uses Supabase as its central database provider. 
It combines a **calendar**, **to-do management**, **group collaboration**, **file sharing**, and a **dashboard** in a desktop interface.

---

## ⚠️ Notes/Disclaimers for Team-LMU

- **Arno:**
- **Marco:** 
   - Used LLMs (Gemini) for research, troubleshooting, refactoring, cumbersome Tasks like reworking SQL Query. All html/CSS Styling elements (typically found in the rsx! Blocks within the #[component] functions), due to no experience with frontend.
   - The two Placeholders in the Dashboard for future expansion (Newsfeed and Chat) are explicitly created fully by a LLM.
   - Communica
- **Max:** 
- **Paul:** 

- **General:**
   - server_fn, #[server], #[cfg(not(feature = "server"))]: Initially, we planned to offer two ways to access the application: as a desktop app and via a web browser using a standalone server. Hence, we began structuring our functions logic using Dioxus's  server_fn type. We later narrowed the project's scope to desktop-only and decided to keep these function signatures to facilitate an easyer transition to the web in the future.
   - Rust stable update llm
   - In some of our Pull-Requests on Github we used

---

## 🛠 System Requirements & Setup


> **Recommendation:** We highly recommend building and running this project on **Windows** due to a significantly simpler setup process and fewer OS-level dependency conflicts.

**Prerequisites:** It is assumed that **Git** and **Rust (version 1.85.0)** are already installed on your system. 

Since Planify is a native desktop app, Dioxus relies on the web rendering engine of the respective operating system. This first guide focuses on the Windows setup. 

### 1. Windows-Specific Requirements (Mandatory)
Rust requires the Microsoft C++ Linker (`link.exe`) to compile native `.exe` binaries on Windows. 

If not already installed, please ensure the **Visual Studio Build Tools** are present on your system. You must have the **"Desktop development with C++"** workload installed, specifically including:
* **MSVC v143**
* **Windows 11 SDK** (or Windows 10 SDK, matching your OS)

### 2. Install Dioxus CLI & WebAssembly Target
The Dioxus Command Line Interface is the core tool for serving the app. It also requires the WebAssembly target for asset management. 

To install the CLI, please follow the instructions provided by Dioxus. Below is an excerpt from the official Dioxus documentation:

> **Install the Dioxus CLI** > Dioxus ships with its own build tool that leverages `cargo` to provide integrated hot-reloading, bundling, and development servers for web and mobile. Follow the instructions provided by the Dioxus Devlopers:
> 
> You can download the cli with `cargo-binstall`:
> ```bash
> cargo binstall dioxus-cli --force
> ```
> 
> If you want to build the CLI from source, you can install it with the following command:
> ```bash
> cargo install dioxus-cli
> ```
> 📣 *Installing from source can take up to 10 minutes and requires several dependencies. We strongly recommend downloading the prebuilt binaries.* > *If you get an OpenSSL error on installation, ensure the dependencies listed here are installed.*

Make sure to also add the WebAssembly target via your terminal:
```bash
cargo install dioxus-cli
rustup target add wasm32-unknown-unknown
```

### 3. Setup & Run
Follow these steps to run the app locally:

1. **Clone the repository (if not already done):**
   ```bash
   git clone [https://github.com/marco-a-g/calender-to-do-list.git](https://github.com/marco-a-g/calender-to-do-list.git)
   ```

2. **Start the app in debug mode:**
   ```bash
   dx serve --desktop
   ```

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Tech Stack](#tech-stack)
4. [Architecture](#architecture)
5. [Local Database & Synchronization](#local-database--synchronization)
6. [Project Structure](#project-structure)
7. [Current Development Status / Notes](#current-development-status--notes)

---

## Overview

Planify is designed for productive team and/or self-organization:

- Create and manage calendar events
- Create, manage, assign and prioritize To-Do's
- Create groups, share files and manage members
- Local SQLite data storage with synchronization to a remote DB (Supabase)

---

## Features

### Authentication & Profile
- Login / Registration
- Profile creation (choose a username)
- Username availability check (with delayed input validation)
- Profile page to view and change the username

### Dashboard
- Home screen with a central overview
- To-do widget showing upcoming tasks this week
- Calendar widget showing events this week
- News widget (Dev.to articles with the #rust tag), clickable to open Dev.to in Browser
- Chat section (not yet implemented)

### To-Dos
- To-do dashboard with filtering options
- Filter To-Do's by Group, To-Do-List and due date
- Detailed view for single To-Do's 
- Create, edit, delete, and complete To-Do's
- Recurring to-dos and series handling
- History view of completed tasks

### Calendar
- Month/View controls
- Load calendar from local data
- Create, edit, and delete events
- Recurring events (including expansion/display)
- Link to group/private context via calendar data

### Groups
- Group overview with color coding and member count
- Create a group (including color selection)
- Group detail page with tabs: **Members**, **Files**, **Roles**
- Invitation workflow (user search, invite handling)
- Leave / delete group (depending on permissions)

### Files in Groups
- File selection via native desktop file picker
- Upload in a group context
- File list per group
- Download (open URL)
- Delete files

### Roles & Permissions
- Role model including: `owner`, `admin`, `member`, `invited`
- Member list displaying roles
- Actions based on role (e.g., Promote/Demote, Ownership transfer, Kick)

---

## Tech Stack

- **Language:** Rust
- **UI/Frontend:** Dioxus
- **HTTP/API:** reqwest
- **Auth/Backend Connection:** Supabase
- **Local Data Storage:** SQLite (sqlx)


---

## Modules

The app is organized into modular components:

- `auth` – Login, registration, session logic
- `dashboard` – Home view and widgets
- `calendar` – Calendar frontend and event logic
- `todos` – Task management including recurrence handling
- `groups` – Groups, members, roles, files, invites
- `database` – SQLite initialization, local fetches, sync
- `user` – Profile functions
- `utils` – Shared data types, helper functions, date conversion

---

## Local Database & Synchronization

Planify uses a local SQLite database as its working foundation and synchronizes with Supabase.

**Sync Principle:**
- A sync is triggered when a user is authenticated.
- Tables are mirrored locally (`...Light` structs).
- A re-sync is triggered after specific write actions.

**Important Note (Current State):**
In the current implementation, the local DB is reset during the initialization process (file deletion before rebuild). This is convenient for development, but not yet final for production offline use.

---

## Project Structure

*(Add a brief overview of your directory tree here if required by the grading rubric)*

---

## Current Development Status / Notes

*(Add any final notes on missing features, known bugs, or next steps here)*