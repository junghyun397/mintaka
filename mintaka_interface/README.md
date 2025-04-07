# mintaka-protocol

## text-protocol

### Configuration Commands

* **config time match [milliseconds]**: Set the time limit for the entire match in milliseconds
* **config time turn [milliseconds]**: Set the time limit for each turn in milliseconds
* **config time increment [milliseconds]**: Set the time increment for each turn in milliseconds
* **config workers [number]**: Set the number of worker threads
* **config workers auto**: Set the number of worker threads automatically based on physical CPU cores
* **config memory [kilobytes]**: Set the maximum memory usage in KiB

### Board Control Commands

* **parse board [board-data]**: Load a board from a string representation
* **parse history [history-data]**: Load a board from move history
* **clear**: Clear the current board
* **board**: Display the current board state
  * **response**: in format board-data
* **history**: Display the move history
  * **response**: in format history-data
* **version**: Display the engine version information
* **set [position] [color]**: Place a stone of specified color at the given position
* **unset [position] [color]**: Remove a stone of specified color from the given position
* **play [position]**: Make a move at the specified position
* **undo**: Undo the last move

#### Tokens

* **position**: A position on the board, represented as a string (e.g., "h8," "a1," etc.)
* **color**: The color of the stone, either "black" or "white"
* **board-data**: A string visualization of the board state
* **history-data**: A string representation of the move history sequence (e.g., "h8,h7,g9...")

### Game Control Commands

* **gen**: Generate a move (launches the engine to search)
  * **response**: in format best move, evaluation (e.g., "h8, 0.5")
* **abort**: Abort the current search
