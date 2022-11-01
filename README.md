# Getting Farted On

A game created for [LD51 game jam](https://ldjam.com/events/ludum-dare/51/getting-farted-on) in 48 hours.

This is the jam version source, checkout the main branch for updates.

![farts](images/farts.gif)

## Play

Jam version can be played [here](https://kuviman.github.io/getting-farted-on/jam/).

**Multiplayer has been removed** from that build so server is no longer needed.

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

You are farting every 10 seconds. A fart forces you in the direction opposite of your back.

**You can force a fart by pressing W/Up/Space**. This does not stop or delay the autofart. You may only force another fart in 5 seconds. Your stomach growls when this ability recharges.

Will you be able to make the ascend to stop this every 10 seconds nonsense

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

- [Trailer (cringe)](https://www.youtube.com/watch?v=91N8bYAOuKg)
- [Creation timelapse](https://www.youtube.com/watch?v=zxApycDzn78)
- [Streamers compilation](https://www.youtube.com/watch?v=dd9-6KY7-6k)

## Building & running from source

To run the game locally all you have to do is:

- [Install Rust compiler](https://rustup.rs/)
- Clone this repository - `git clone https://github.com/kuviman/getting-farted-on && cd getting-farted-on`
- For jam version `git checkout jam-version`
- Build the project, run local server & connect to it - `cargo run`

There are more options you can specify as command line agruments, e.g. `cargo run -- --server 127.0.0.1:1155` will only run a server, and `cargo run -- --connect 127.0.0.1:1155` will only run a client connecting to it.

For more options check `cargo run -- --help` or the source code.
