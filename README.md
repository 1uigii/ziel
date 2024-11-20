A simple and modular implementation of the popular battleship game.

## How to run the game
The executable can be work as server or as client.
```bash
# run the server
ziel server --addr <ADDR> default: 127.0.0.1:8080

# run the client
ziel client --addr <ADDR> default: 127.0.0.1:8080
```

To run it directly from the source code use cargo.
```bash
# run the server
cargo r -r -- server --addr <ADDR> default: 127.0.0.1:8080

# run the client
cargo r -r -- client --addr <ADDR> default: 127.0.0.1:8080
```
The Keybinds:
- `q` exit
- when in ship placement mode:
  - `<space>` lift ship up/place ship
  - `<arrow keys/wasd>` move the cursor/ship
  - `<enter>` play a match
- when in battle mode
  - `<space>` select a target

 ## Create a Custom UI
 - Implement the UI trait in `client::ui::UI`
 - Rewrite the main funtion to use your UI

 Documentation will follow for the implementation of the trait.
