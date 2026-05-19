# FinRadar
[![License: MIT](https://img.shields.io/badge/License-MIT-purple.svg)](https://opensource.org/licenses/MIT)

This app combines my previous projects, the [Finance Tracker](https://github.com/Stenberg-N/finance-tracker) and the [FocusBoard](https://github.com/Stenberg-N/focusboard), into a single coherent app where I apply all the lessons learned.

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
<img width="1918" height="1081" alt="fin-radar3" src="https://github.com/user-attachments/assets/e2aab92b-d255-4e0d-8525-de1d612cff36" />
<img width="1767" height="980" alt="fin-radar4" src="https://github.com/user-attachments/assets/17eeb834-4219-41b7-a82b-b548626d2d13" />
<img width="1915" height="1058" alt="fin-radar6" src="https://github.com/user-attachments/assets/9f149688-c710-4212-8038-b59447049f26" />


## Acknowledgements
This app uses icons for its UI from Uicons by <a href="https://www.flaticon.com/uicons">Flaticon</a>

## License
This project is licensed under the MIT license. See the LICENSE file for details.
