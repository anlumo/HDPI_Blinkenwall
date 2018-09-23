# HDPI_Blinkenwall

Blinkenwall v2 is our current project for replacing our old Blinkenwall out of glass bricks of our local Viennese Hackspace Metalab .

Shoutout and thanks to all this awesome people, all of whom helped out to make this project reality.

Special thanks goes to our member ripper, who invested a lot of his personal time for this project

More information at https://metalab.at/wiki/Blinkenwall


## Setup

Run the backend:

```
$ git init --bare shadertoy.git
$ cargo build
$ cargo run
```

in a 2nd terminal, follow the README.d in `blinkenwall` to start the frontend.
