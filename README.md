# ICI Image Editor

Used to edit ICI images (`IndexedImage` and `AnimatedIndexedImage`) from [ici-files](https://github.com/emmabritton/ici-files)

Designed to be drawn with [Buffer graphics](https://github.com/emmabritton/buffer-graphics-lib) and in turn, [Pixel graphics](https://github.com/emmabritton/pixel-graphics-lib)

## Usage

Open the program normally

or set a default palette for all new images for this session with
`./image-editor /path/to/palette`

## Controls

* Undo - Ctrl+Z, Cmd+Z
* Redo - Shift+Ctrl+Z, Shift+Cmd+Z, Ctrl+Y, Cmd+Y
* Save single frame when timeline is visible - Hold Shift when saving
* Shift by 1px - Shift+Up/Down/Left/Right

## Screenshots

![Editor](https://github.com/emmabritton/ici-image-editor/raw/main/.github/screenshots/image.png)
![Editor with timeline](https://github.com/emmabritton/ici-image-editor/raw/main/.github/screenshots/animated.png)
![Menu](https://github.com/emmabritton/ici-image-editor/raw/main/.github/screenshots/menu.png)
![Palette editor](https://github.com/emmabritton/ici-image-editor/raw/main/.github/screenshots/palette.png)

## TODO
- Images larger than 64px
- Canvas scrolling/zoom