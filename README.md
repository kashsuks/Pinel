A somewhat fast text editor written in Rust

## Docs Site

The documentation site lives in `src/`. It uses a simple Node.js build script (no Jekyll/Ruby required).

```bash
cd src
npm install
npm run build    # generates _site/
npm run dev      # rebuilds on file changes
```

### Adding docs

Drop a `.md` file in `src/docs/` with frontmatter:

```markdown
---
title: My Page
description: What this page covers.
category: Basics
order: 2
---

Your markdown content here...
```

Run `npm run build` and the page appears at `/docs/my-page/`.
