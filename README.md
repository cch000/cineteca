# movies-tui \[WIP\]

Simple tui-based movie library to keep track of watched movies in a directory.

![](img/screenshot.png)

This screenshot shows a library with two movies. One of them is selected, and the other one has been marked as watched.

## How does it work?

The movie library is generated automatically, filtering files that might not be movies.
This is achieved by recursively reading all the files in the directory and discarding those
that do not have a video extension or are short videos. The program also checks every time
it is opened for changes in the directory, calculating a hash of the directory and comparing
it to the previous one. If there are changes, the library will be updated 
(e.g., if a new movie has been added). 

## Usage

- Press 'w' toggle watched 
- Press 'p' to play the movie (requires `mpv`)
- Press '?' to show all keybinds

More options will be added in the future
...
