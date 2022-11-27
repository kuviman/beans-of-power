# Getting Farted On

![farts](images/farts.gif)

## Play

You can play the game on [itch.io](https://kuviman.itch.io/getting-farted-on).

*The game was initially created for [Ludum Dare 51](https://ldjam.com/events/ludum-dare/51/getting-farted-on) game jam in 48 hours (the theme was "Every 10 Seconds"), you can check out the [jam version here](https://kuviman.github.io/getting-farted-on/jam/), jam version source is available at [jam-version branch](https://github.com/kuviman/getting-farted-on/tree/jam-version)*

## Description

*MMO foddian farting simulator*.

\- dude this burger was delicious im gona have second one

\- dude you already had 10 "seconds" are you sure

\- dude you right brb i need to poo real quick

Oh no the toilet is out of order

And you really need it very badly, you are already farting every 10 seconds

Looks like your only hope is that shiny one at the top

You are really trying to hold it so you cant even walk like a human. All you can do is roll.

**Use A/D or Left/Right to roll in desired direction**.

~~You are farting every 10 seconds~~. Ok, the jam is over, so no more every 10 seconds nonsense. Your chamber is building up pressure, if it reaches the limit, you are going to auto fart.

**You can force a fart by pressing W/Up/Space**. This, same as auto fart, releases some of the pressure. Although, you have to build a minimum to be able to do it, otherwise you are only going to accelerate pressure building up. If you continue forcing a fart by not releasing the button, you are going to do a long fart, until your inner pressure reaches zero.

Fart your way to the top as soon as possible

## Controls

- A/D or Left/Right - roll in desired direction
- W/Up/Space - force a fart (5 second cooldown)
- H - show/hide player names
- Ctrl-R - quick restart
- 1/2/3/4 - emotes

## Tools used to make this

Tools used to make this

- [Rust programming language](https://www.rust-lang.org/)
- [Custom engine](https://github.com/kuviman/geng)
- [VS Code](https://code.visualstudio.com/)
- [Paint.Net](https://getpaint.net/)
- [Audacity](https://www.audacityteam.org/)
- Microphone
- Guitar
- Mouth

## Some videos

- [Trailer](https://www.youtube.com/watch?v=91N8bYAOuKg) (cringe)
- [Creation timelapse](https://www.youtube.com/watch?v=zxApycDzn78) (meh)
- [Streamers compilation](https://www.youtube.com/watch?v=dd9-6KY7-6k) (pog)

## Building & running from source

To run the game locally all you have to do is:

- [Install Rust compiler](https://rustup.rs/)
- Clone this repository - `git clone https://github.com/kuviman/getting-farted-on && cd getting-farted-on`
- Build the project, run local server & connect to it - `cargo run`

There are more options you can specify as command line agruments, e.g. `cargo run -- --server 127.0.0.1:1155` will only run a server, and `cargo run -- --connect 127.0.0.1:1155` will only run a client connecting to it.

For more options check `cargo run -- --help` or the source code.
