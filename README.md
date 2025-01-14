# Tauri + Vanilla

This template should help get you started developing with Tauri in vanilla HTML, CSS and Javascript.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)



# State and Exchanged Objects

* `RevealState` 
  * Current list of images
  * "Active" image

* `RevealSettings`
  * Image source (directory, gallery, ...)
  * Covering type (triangles, rectangles, ...)
  * Uncovering strategy (manual, automatic, ...)
  * Show control buttons

* `RevealObject`
  * Image
  * Covering
  * Question
    * Answer options
    * Correct answer
  * Settings
