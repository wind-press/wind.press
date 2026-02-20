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

Note that this is not required to run in development and contribute to the WindPress website or documentation.

Build the application for production:

```bash
pnpm generate
```

NuxtHub Admin and the `nuxthub deploy` flow are deprecated. For deployment, use your hosting provider's native CI/CD (for example Vercel, Cloudflare Pages, or Cloudflare Workers with Wrangler) against the generated static output in `.output/public`.

### GitHub Actions (Cloudflare Pages)

This repository includes `.github/workflows/deploy-cloudflare-pages.yml` to build and deploy on every push to `main`.

If you use Cloudflare Pages Git integration from the dashboard, remove or disable this workflow to avoid duplicate deployments.

Set these GitHub settings before enabling the workflow:

- **Secrets**
  - `CLOUDFLARE_API_TOKEN`
  - `CLOUDFLARE_ACCOUNT_ID`
- **Variables**
  - `CLOUDFLARE_PAGES_PROJECT_NAME` (optional, defaults to repository name)
  - `NUXT_PUBLIC_SITE_URL` (optional but recommended)

### Cloudflare Pages Git Integration (no GitHub Action)

You can also connect the repository directly in Cloudflare Pages:

- Build command: `pnpm build`
- Build output directory: `.output/public`

## License

[MIT License](./LICENSE)
