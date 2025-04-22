---
title: 'Preline UI'
description: 'Learn how to use Preline UI on WordPress websites.'
---

This guide will show you how to use Preline UI on WordPress websites using WindPress.

::u-alert
---
class: not-prose ring-1 ring-amber-500 dark:ring-amber-400
color: amber
icon: line-md:construction
variant: subtle
title: Tailwind CSS 4 compatibility
---
#description
<div class="leading-6">
At the time of writing, Preline UI is currently available for Tailwind CSS 3 only. This guide's instructions are based on the Tailwind CSS 3 version. It will be updated as the Tailwind 4 compatible version is released.
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

Navigate to the `WindPress` menu and switch to `tailwind.config.js` file editor.

![The `tailwind.config.js` file editor](/img/content/docs/guide/configuration/tw-3/file-tailwind-config-js/screenshot-1.png){width="1440px" height="866px"}

### Step 2: Add the Preline UI plugin

Add the following line of code to the `tailwind.config.js` file:

```js [tailwind.config.js] {5}
export default {
    /* ... */

    plugins: [
        require('preline/plugin'),
    ],

    /* ... */
}
```

### Step 3: Use the Preline UI interactive elements

Preline UI comes with a set of interactive elements like modals, dropdowns, and more. To use these elements, you need to add the Preline UI JavaScript file to your WordPress website.


Add the following code to your theme's `functions.php` file or a [Snippet plugin](https://wordpress.org/plugins/search/Snippet/).

```php [functions.php]
<?php

add_action('wp_enqueue_scripts', function () {
    wp_enqueue_script('prelineui', 'https://esm.sh/preline/dist/preline.js?raw');
});

function register_prelineui_provider(array $providers): array {
    $providers[] = [
        'id' => 'prelineui',
        'name' => 'Preline UI Scanner',
        'description' => 'Scans the Preline UI interactive elements',
        'callback' => 'scanner_cb_prelineui_provider',
        'enabled' => \WindPress\WindPress\Utils\Config::get(sprintf(
            'integration.%s.enabled',
            'prelineui'
        ), true),
    ];

    return $providers;
}
add_filter('f!windpress/core/cache:compile.providers', 'register_prelineui_provider');

function scanner_cb_prelineui_provider(): array {
    return [
        [
            'name' => 'prelineui.min.js',
            'content' => file_get_contents('https://esm.sh/preline/dist/preline.js?raw'),
        ],
    ]
}
```