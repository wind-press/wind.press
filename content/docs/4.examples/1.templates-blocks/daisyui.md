---
title: 'daisyUI'
description: 'Learn how to use daisyUI on WordPress websites.'
---

This guide will show you how to use daisyUI 5 on WordPress websites using WindPress.

## What is daisyUI?

[daisyUI](https://daisyui.com/) is a plugin for Tailwind CSS that simplifies UI development by providing pre-designed components with semantic class names. Instead of writing long utility class lists, you can use intuitive classes like `btn`, `card`, or `alert` to quickly style your elements.

## Why use daisyUI?

- **Faster Development**: Build beautiful interfaces with minimal code.
- **Consistent Design**: Get access to a well-structured design system.
- **Customizable Themes**: Easily switch between themes, including dark mode.
- **Lightweight & Flexible**: Works seamlessly with Tailwind CSS without adding extra bloat.

## Using daisyUI on WordPress

Installing daisyUI on WordPress is easy with WindPress. It's as simple as adding a single line of code to the `main.css` file.

### Step 1: Navigate to the WindPress admin screen

Navigate to the `WindPress` menu and switch to `main.css` file editor.

![The `main.css` file editor](/img/content/docs/guide/configuration/tw-4/file-main-css/screenshot-1.png){width="1440px" height="866px"}

### Step 2: Add the daisyUI plugin

Add the following line of code to the `main.css` file:

```postcss [main.css]
/* ... */

@plugin "daisyui"; /* or via jsDelivr CDN: "https://esm.run/daisyui" */

/* ... */
```

Now you can use daisyUI class names!