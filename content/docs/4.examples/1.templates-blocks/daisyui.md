---
title: 'daisyUI'
description: 'Learn how to use daisyUI on WordPress websites.'
---

This guide will show you how to use daisyUI 5 on WordPress websites using WindPress.

::u-alert
---
class: not-prose ring-1 ring-amber-500 dark:ring-amber-400
color: amber
icon: line-md:construction
variant: subtle
title: Tailwind CSS version
---
#description
<div class="leading-6">
This guide's instructions are based on the Tailwind CSS 4 version.
</div>
::

## What is daisyUI?

[daisyUI](https://daisyui.com/){target="_blank"} is a plugin for Tailwind CSS that simplifies UI development by providing pre-designed components with semantic class names. Instead of writing long utility class lists, you can use intuitive classes like `btn`, `card`, or `alert` to quickly style your elements.

## Why use daisyUI?

- **Faster Development**: Build beautiful interfaces with minimal code.
- **Consistent Design**: Get access to a well-structured design system.
- **Customizable Themes**: Easily switch between themes, including dark mode.
- **Lightweight & Flexible**: Works seamlessly with Tailwind CSS without adding extra bloat.

## Using daisyUI on WordPress

Installing daisyUI on WordPress is easy with WindPress. It's as simple as adding a single line of code to the `main.css` file.

### Step 1: Navigate to the WindPress admin screen

Navigate to the `WindPress` menu and switch to `main.css` file editor.

### Step 2: Add the daisyUI plugin

Add the following line of code to the `main.css` file:

```postcss [main.css]
/* ... */

@plugin "daisyui"; /* or via jsDelivr CDN: "https://esm.run/daisyui" */

/* ... */
```

Now you can use daisyUI class names!

## daisyUI themes

daisyUI comes with 35 built-in themes that instantly transform your website's entire look and feel. You can also create your own custom themes or customize built-in themes.

::callout{icon="i-heroicons-light-bulb"}
Learn more about daisyUI themes in the [daisyUI documentation](https://daisyui.com/docs/themes/){target="_blank"} and see the full list of available themes.
::

It's so easy to switch between themes by adding the `data-theme` attribute to your HTML element.

```html
<html data-theme="cupcake">
    <!-- ... -->
</html>
```
