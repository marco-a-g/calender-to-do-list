# Planify 🦀  
**A collaboration desktop App written in Rust and the Dioxus framework**

Planify is a full-stack app for personal and collaborative organization of .  
It combines a **calendar**, **to-do management**, **group collaboration**, **file sharing**, and a **dashboard** in a desktop interface.

---

## ⚠️ Notes/Disclaimers

*This section is intended for individual remarks from team members to the reviewers/graders.*

- **Arno:**
- **Marco:** 
- **Max:** 
- **Paul:** 

---

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Tech Stack](#tech-stack)
4. [Architecture](#architecture)
5. [Local Database & Synchronization](#local-database--synchronization)
6. [System Requirements & Dioxus Setup (IMPORTANT)](#system-requirements--dioxus-setup-important)
7. [Setup & Run](#setup--run)
8. [Project Structure](#project-structure)
9. [Current Development Status / Notes](#current-development-status--notes)

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
- News widget (Dev.to articles with the #rust tag)
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
- **Routing:** dioxus-router
- **HTTP/API:** reqwest
- **Auth/Backend Connection:** Supabase
- **Local Data Storage:** SQLite (sqlx)
- **Async Runtime:** Tokio
- **Date/Time:** chrono
- **Desktop Integration:** Dioxus Desktop + native file dialogs (`rfd`)

---

## Architecture

The app is organized into modular components:

- `auth` – Login, registration, session logic
- `dashboard` – Home view and widgets
- `calendar` – Calendar frontend and event logic
- `todos` – Task management including recurrence handling
- `groups` – Groups, members, roles, files, invites
- `database/local` – SQLite initialization, local fetches, sync
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

## System Requirements & Dioxus Setup (IMPORTANT)

Since Planify is a native desktop app, Dioxus relies on the web rendering engine of the respective operating system. Before the project can be built or run, the corresponding OS libraries must be installed.

### 1. Install Dioxus CLI (All Operating Systems)
The Dioxus Command Line Interface is the core tool for serving and bundling the app.
```bash
cargo install dioxus-cli