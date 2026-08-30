# FinRadar
[![License: MIT](https://img.shields.io/badge/License-MIT-purple.svg)](https://opensource.org/licenses/MIT)

FinRadar is a local desktop application combining finance tracking, data visualization, notes, calendar and alerts. It is built with Svelte and Typescript on the frontend and Tauri/Rust for the application logic.<br/>
It is a continuation of my previous projects, where it combines them, the [Finance Tracker](https://github.com/Stenberg-N/finance-tracker) and the [FocusBoard](https://github.com/Stenberg-N/focusboard), into a single coherent app where I apply all the lessons learned.
It is built for my own needs much like the previous projects and it will continue to get more features if and when I have a need for something.

## Architecture
```mermaid
flowchart TD
  subgraph FRONTEND["Frontend"]
    UI["UI<br/>Svelte + TypeScript"]
    LIB["Frontend logic<br/>lib/"]
    UI --> LIB
  end

  subgraph BACKEND["Tauri"]
    RUST["Tauri commands"]
    DB["SQLite (sqlx)"]
    PY["Python / ML<br/>(Planned)"]
    RUST --> DB
    RUST --> PY
  end

  LIB --> RUST
```
FinRadar uses a Svelte + TypeScript frontend with Tauri/Rust handling application logic and SQLite for persistence. Shared frontend modules (lib/) communicate with Tauri commands. Python-based ML functionality is planned for forecasting finances.

### Planned ML prediction flow
```mermaid
sequenceDiagram
  participant UI as Svelte UI
  participant RUST as Tauri
  participant DB as SQLite
  participant PY as Python

  UI ->> RUST: Request prediction
  RUST ->> DB: Query data
  DB -->> RUST: Data
  RUST ->> PY: Pass data to prediction function
  PY -->> RUST: Prediction data
  RUST -->> UI: Return prediction data
  UI ->> UI: Render chart
```

## Installation
### App installer
Currently, there is no installer available.

### Manual
#### Requirements
1. Rust. You can install it [here](https://rust-lang.org/tools/install/)
2. Microsoft Visual Studio C++ Build Tools, found [here](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
3. Node.js for the dev environment, found [here](https://nodejs.org)

#### Next steps
##### Clone the repository
```sh
git clone https://github.com/Stenberg-N/fin-radar.git
```
```sh
cd fin-radar
```
##### Install the dependencies
```sh
npm install
```
##### Run the dev environment
```sh
npm run tauri dev
```
##### Alternatively, to build the app
```sh
npm run tauri build
```

## Other
The app saves its database and logs inside your user's local app data:<br>
C:\Users\Your_username\AppData\Local\com.stenberg.fin-radar

The app saves other data to the user's roaming app data, e.g. preferences and other settings set by the user:<br>
C:\Users\Your_username\AppData\Roaming\com.stenberg.fin-radar

## Screenshots
<img width="1188" height="739" alt="fin-radar1" src="https://github.com/user-attachments/assets/30780e0e-69cf-4b39-90c5-4b25602d6a58" />
<img width="1919" height="1079" alt="fin-radar3" src="https://github.com/user-attachments/assets/8365f594-2aff-4c3c-a68e-f9804637faea" />
<img width="1917" height="1081" alt="fin-radar4" src="https://github.com/user-attachments/assets/ccb17b0a-e450-4ccd-9581-1c5802b3dd89" />
<img width="1915" height="1058" alt="fin-radar6" src="https://github.com/user-attachments/assets/9f149688-c710-4212-8038-b59447049f26" />
<img width="802" height="441" alt="fin-radar8" src="https://github.com/user-attachments/assets/b01e7bb8-00bc-49ac-bc5b-94ca6121416c" />
<img width="1919" height="1077" alt="fin-radar11" src="https://github.com/user-attachments/assets/39cb439c-49b7-4f8c-92e3-e60d14f6ac7a" />
<img width="1918" height="1077" alt="fin-radar9" src="https://github.com/user-attachments/assets/0b362624-b5ed-4ac1-83c2-8f222e24a5c8" />
<img width="779" height="659" alt="fin-radar10" src="https://github.com/user-attachments/assets/53eebe3a-6ec2-48e0-a615-7103962abbcc" />

## Acknowledgements
This app uses icons for its UI from Uicons by <a href="https://www.flaticon.com/uicons">Flaticon</a>

## License
This project is licensed under the MIT license. See the LICENSE file for details.
