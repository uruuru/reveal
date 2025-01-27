<div align="center">
  <img src="src-tauri/icons/Logo_200.png" width="100px" alt="Reveal!">
</div>

# Reveal!

Reveal is a simple interactive game where participants have to identify
a partly visible image as quick as possible.

## Controls

The game can either be controlled via the control buttons in the top-right,
via hotkeys on desktops, or via touch and swipe gestures on mobile devices.

Action | Desktop | Mobile
-- | -- | --
Further uncover the image | `u` / `⎵`  | touch
Fully reveal the image | `c` / `↓` | swipe down
Next image | `n` / `→` | swipe left
Previous image | `p` / `←` | swipe right
Reset the covering | `r` / `↑` |  swipe up

## Settings

Further settings and the ability to select different images are available via the `⚙` in the top-right. Changed settings are persisted.

## Image Sources

The game tries to load images from a couple of default locations before asking the user for manual selection.
Not every platform supports every location. Access to files on Android and iOS differs from desktops.
* Desktops (in this order)
  * folder `reveal` in the user's pictures folder,
  * folder `reveal_images` next to the executable.
* Android
  * folder `reveal` in the user's pictures folder.
* iOS
  * folder `reveal` in "On My iPhone/iPad".

The user is able to manually select images via the settings,
either selecting a folder from which all images are loaded
or selecting individual images.
Due to current limitations in the underlying framework,
Android and iOS only support selecting individual images.
On iOS it is possible to either select images from files
or from Photos.

## Examples
TODO


# Trivia
The motivation to implement the game was getting to know Rust and Tauri,
and understanding to which extent development for multiple platforms (including mobile) is possible
from a single code base.


# Development

* [Tauri](https://tauri.app/)
* Frontend in vanilla HTML, JS, and CSS
* Backend in [Rust](https://www.rust-lang.org/)

## Building

```
cargo tauri dev/build/...
```

```
npx @biomejs/biome check
```

## Distribution

macOS `.app` bundle
```
cargo tauri build --bundles app --target universal-apple-darwin
```

iOS `.ipa`
```
cargo tauri ios build
```

Windows (cross-compiled on linux)
```
cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
```
