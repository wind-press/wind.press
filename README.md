# wind.press

Welcome to the WindPress website repository available on [wind.press](https://wind.press).

> This repository is derived from the Nuxt website repository with modifications.

## Setup

Make sure to enable corepack and install the dependencies:

```bash
corepack enable
pnpm install
```

Copy the `.env.example` file to `.env`:

```bash
cp .env.example .env
```

## Development

Start the development server:

```bash
pnpm dev
```

### Add a Showcase

To list a showcase, add a file in the [./content/showcase](./content/showcase) directory.

Make sure to start the development server in order to generate the screenshot for the showcase and go to http://localhost:3000/showcase to see the result.

If you want to use your own custom screenshot, use the `screenshotUrl` property.

To regenerate the image, delete the generated one in `public/assets/templates`.

### Add a Tailwind CSS Template

To list a Tailwind CSS template, add a file in the [./content/templates](./content/templates) directory.

Make sure to start the development server in order to generate the screenshot for the template and go to http://localhost:3000/templates to see the result.

If you want to update the url where we take the automated screenshot, use the `screenshotUrl` property.

To regenerate the image, delete the generated one in `public/assets/templates`.

## Production

In order to build the application for production, you need to have a [Nuxt UI Pro](https://ui.nuxt.com/pro?aff=GZ5Zd) license and set the `NUXT_UI_PRO_LICENSE` variable in the `.env` file.

Note that this is not required to run in development and contribute to the WindPress website or documentation.

Build the application for production:

```bash
pnpm generate
```

## License

[MIT License](./LICENSE)
