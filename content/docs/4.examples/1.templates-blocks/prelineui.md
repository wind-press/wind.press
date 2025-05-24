---
title: 'Preline UI'
description: 'Learn how to use Preline UI on WordPress websites.'
---

This guide will show you how to use Preline UI v3 on WordPress websites using WindPress.

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

## What is Preline UI?

[Preline UI](https://preline.co/) is an open-source library of prebuilt UI components designed to enhance web development with the utility-first Tailwind CSS framework. It offers over 840 accessible components, including elements like modals, buttons, alerts, and cards, enabling developers to swiftly design and customize responsive, mobile-first websites. Fully compatible with frameworks such as React and Vue, Preline UI supports dark mode variants and provides seamless integration with popular development tools. Additionally, it includes free templates and a comprehensive Figma design system, making it a valuable resource for developers aiming to expedite their project workflows.

### Why use Preline UI?

- **Extensive Component Library**: Over 840 prebuilt, accessible UI components tailored for Tailwind CSS.
- **Seamless Tailwind CSS Integration**: Designed to work effortlessly with utility-first styling.
- **Framework Compatibility**: Supports React, Vue, and other modern frameworks.
- **Dark Mode Support**: Built-in variants for light and dark themes.
- **Responsive & Mobile-First**: Ensures a great user experience across all devices.
- **Free & Open-Source**: Available for use without cost, with a Pro version offering additional features.
- **Figma Design System**: Includes a comprehensive design system for UI/UX designers.
- **Developer-Friendly**: Provides detailed documentation and examples for easy implementation.

## Using Preline UI on WordPress

Installing Preline UI on WordPress is easy with WindPress.

### Step 1: Navigate to the WindPress admin screen

Navigate to the `WindPress` menu and switch to `main.css` file editor.

### Step 2: Add the Preline UI variants

Add the following line of code to the `main.css` file:

```postcss [main.css]
/* ... */

@import "preline/variants.css";

@plugin "@tailwindcss/forms";

/* ... */
```

### Step 3: Use the Preline UI interactive elements (optional)

Preline UI comes with a set of interactive elements like modals, dropdowns, and more. To use these elements, you need to add the Preline UI JavaScript file to your WordPress website.

Add the following line of code to the `main.css` file:

```postcss [main.css]
/* ... */

@source "https://esm.sh/preline/dist/preline.js?raw";

/* ... */
```

Add the following code to your theme's `functions.php` file or a [Snippet plugin](https://wordpress.org/plugins/search/Snippet/).

```php [functions.php]
<?php

add_action('wp_enqueue_scripts', function () {
    wp_enqueue_script('prelineui', 'https://esm.sh/preline/dist/preline.js?raw');
});
```

### Step 4: Preline UI styles (optional)

Please note, Preline UI comes with some opinionated styles that are applied to components by default. If you want these styles in your project, you may include them into the `main.css` file.

```postcss [main.css]
/* ... */

/* Adds pointer cursor to buttons */
@layer base {
  button:not(:disabled),
  [role="button"]:not(:disabled) {
    cursor: pointer;
  }
}

/* Defaults hover styles on all devices */
@custom-variant hover (&:hover);

/* ... */
```