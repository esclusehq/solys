# Quick Task: Fix Console Layout

## Problem

1. **Page not scrollable** — Every wrapper in Console.jsx has `overflow-hidden`, preventing the App's scroll container from working. If viewport is small, content is cut off.

2. **Over-nested layout** — Extra `<div>` wrapper creates unnecessary depth: outer flex-col → flex row → content wrapper → terminal wrapper. The `px-8 pb-6 pt-5` padding is on an intermediate wrapper, not the outer container.

3. **Terminal minHeight** — Inline `minHeight: '300px'` on the xterm div forces a minimum size, causing layout to overflow or incorrectly size when space is tight.

4. **"Split" appearance** — Sidebar creates a visual split with no page-level scrolling, making the layout feel broken.

## Tasks

### Task 1: Fix Console.jsx layout

- Remove `overflow-hidden` from outer wrapper — let parent scroll container handle overflow
- Remove `overflow-hidden` from flex row — ditto
- Merge `px-8 pb-6 pt-5` padding from nested content wrapper into the flex row
- Remove the intermediate `<div className="flex-1 flex px-8 pb-6 pt-5 overflow-hidden min-h-0">`
- Direct child of flex row becomes just `<div className="flex-1 flex min-h-0">` wrapping the content
- Remove `overflow-hidden` from this terminal wrapper too

### Task 2: Fix Terminal.jsx layout

- Remove `minHeight: '300px'` style from the xterm container div
- Keep `h-full w-full p-0` — let the flex layout determine height
- Ensure terminal-area flex-1 fills properly

## Verification

1. `npm run build` passes
2. Visual: no cut-off at any viewport height
3. Page scrolls when content exceeds viewport
4. Terminal still fills available space
