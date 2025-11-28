# Copilot Instructions for Popcorn Hero

## Project Overview

This project is a **Tauri desktop application** that uses **Iroh** for establishing direct peer-to-peer connections with fallback to relay servers when necessary. All connections are authenticated and encrypted end-to-end using QUIC.

The frontend is built with **Svelte** and TypeScript, while the backend leverages **Rust** through Tauri's architecture.

## Iroh Documentation Reference

### What is Iroh?

Iroh lets you establish direct peer-to-peer connections whenever possible, falling back to relay servers if necessary. This gives you fast, reliable connections that are authenticated and encrypted end-to-end using QUIC.

**Official Documentation**: https://www.iroh.computer/docs

### Key Resources

- **[Quickstart Guide](https://www.iroh.computer/docs/quickstart)** - Get started with Iroh quickly
- **[Overview](https://www.iroh.computer/docs/overview)** - High level look at the problems Iroh solves
- **[Protocol Registry](https://www.iroh.computer/proto)** - Iroh Protocols are pluggable extensions that build on direct connections
- **[Awesome List](https://github.com/n0-computer/awesome-iroh)** - Projects & resources building with Iroh
- **[FAQ](https://www.iroh.computer/docs/faq)** - Frequently asked questions
- **[GitHub Repository](https://github.com/n0-computer/awesome-iroh)** - Iroh source code and community
- **[Discord Community](https://iroh.computer/discord)** - Join the Iroh community

## Tauri Documentation Reference

### What is Tauri?

Tauri is a framework for building tiny, fast binaries for all major desktop and mobile platforms. Developers can integrate any frontend framework that compiles to HTML, JavaScript, and CSS for building their user experience while leveraging Rust, Swift, and Kotlin for backend logic.

**Official Documentation**: https://v2.tauri.app/start/

### Key Features

- **Secure Foundation**: Built on Rust for memory, thread, and type-safety benefits. Undergoes security audits for major/minor releases.
- **Smaller App Size**: Tauri apps use the system's native webview, resulting in minimal Tauri apps being less than 600KB.
- **Flexible Architecture**: Compatible with any frontend framework (Svelte, React, Vue, etc.). JavaScript-to-Rust bindings via `invoke` function.

### Key Resources

- **[Prerequisites](https://v2.tauri.app/start/prerequisites/)** - Dependencies required to build Tauri apps
- **[Create a Project](https://v2.tauri.app/start/create-project/)** - Step-by-step project creation guide
- **[Project Structure](https://v2.tauri.app/start/project-structure/)** - Understanding Tauri project layout
- **[Frontend Configuration](https://v2.tauri.app/start/frontend/)** - Setup for Svelte, React, Vue and other frameworks
- **[Plugins](https://v2.tauri.app/plugin/)** - Extend Tauri functionality with official and community plugins
- **[Security Policy](https://github.com/tauri-apps/tauri/security/policy)** - Tauri security information
- **[GitHub Repository](https://github.com/tauri-apps/tauri)** - Tauri source code
- **[Discord Community](https://discord.com/invite/tauri)** - Tauri community support

## Svelte Documentation Reference

### What is Svelte?

Svelte is a framework for building user interfaces on the web. It uses a compiler to turn declarative components written in HTML, CSS and JavaScript into lean, tightly optimized JavaScript that runs in the browser.

**Official Documentation**: https://svelte.dev/docs

### Key Features

- **Compiler-based**: Converts components to optimized JavaScript at build time
- **Reactive by default**: Reactive statements and stores make state management simpler
- **Scoped styles**: CSS is scoped to components automatically
- **Small bundle size**: Minimal overhead compared to other frameworks
- **Interactive tutorial**: Built-in learning resource for getting started

### Key Resources

- **[Interactive Tutorial](https://svelte.dev/tutorial)** - Best place to start learning Svelte
- **[Playground](https://svelte.dev/playground)** - Try Svelte online instantly
- **[API Reference](https://svelte.dev/docs/svelte)** - Complete component and store documentation
- **[FAQ](https://svelte.dev/docs/faq)** - Frequently asked questions
- **[Discord Community](https://svelte.dev/chat)** - Get help from the community

## SvelteKit Documentation Reference

### What is SvelteKit?

SvelteKit is a framework for rapidly developing robust, performant web applications using Svelte. It's similar to Next.js (for React) or Nuxt (for Vue). SvelteKit handles routing, SSR, build optimizations, and modern development practices.

**Official Documentation**: https://svelte.dev/docs/kit

### Key Features

- **File-based routing**: Automatic routing based on file structure in `src/routes/`
- **SSR, CSR, and Prerendering**: Configurable rendering modes for different use cases
- **Build optimizations**: Automatic code splitting and minimal code loading
- **Hot Module Replacement (HMR)**: Instant code reflection during development via Vite
- **Offline support**: Service worker integration for offline functionality
- **Image optimization**: Automatic image optimization and responsive images
- **Link preloading**: Automatic data preloading for faster navigation

### Key Resources

- **[Creating a Project](https://svelte.dev/docs/kit/creating-a-project)** - Project setup guide
- **[Project Structure](https://svelte.dev/docs/kit/project-structure)** - Understanding SvelteKit layout
- **[Routing](https://svelte.dev/docs/kit/routing)** - File-based routing documentation
- **[Page Options](https://svelte.dev/docs/kit/page-options)** - SSR, CSR, Prerendering configuration
- **[API Routes](https://svelte.dev/docs/kit/routing#Server)** - Creating backend endpoints
- **[Stores](https://svelte.dev/docs/kit/state-management)** - State management with SvelteKit
- **[Interactive Tutorial](https://svelte.dev/tutorial/kit)** - SvelteKit-specific learning
- **[Discord Community](https://svelte.dev/chat)** - Get help from the community

## shadcn-svelte Documentation Reference

### What is shadcn-svelte?

shadcn-svelte is an unofficial, community-led Svelte port of shadcn/ui. It's not a traditional component library - it's how you build your component library. You get the actual component code with full control to customize, modify, and extend components to your needs.

**Official Documentation**: https://shadcn-svelte.com/docs

### Key Principles

- **Open Code**: Full transparency - you see and control all component code
- **Composition**: Every component uses a common, composable interface
- **Distribution**: Schema-based CLI tool for distributing components across projects
- **Beautiful Defaults**: Carefully chosen default styles using Tailwind CSS
- **AI-Ready**: Open code makes it straightforward for LLMs to read and improve components

### Key Features

- **Bits UI**: Headless component library for composable UI building
- **Tailwind CSS**: Utility-first CSS framework for styling
- **Full Customization**: Modify any component directly to fit your needs
- **Large Component Collection**: Pre-built, well-designed components ready to use
- **Cross-framework Support**: CLI distribution system supporting multiple frameworks

### Key Resources

- **[Installation](https://shadcn-svelte.com/docs/installation)** - Setup and initialization
- **[Components](https://shadcn-svelte.com/docs/components)** - Browse and install components
- **[Customization](https://shadcn-svelte.com/docs/customization)** - Theming and style customization
- **[Theme Generator](https://shadcn-svelte.com/themes)** - Create custom color themes
- **[GitHub Repository](https://github.com/huntabyte/shadcn-svelte)** - Source code and issues
- **[Discord Community](https://svelte.dev/chat)** - Svelte/shadcn community support

## Development Guidelines

When working on this project, keep in mind:

### Tauri & Desktop Development
1. Backend logic should be implemented in Rust (in `src-tauri/src/`)
2. Use the `invoke` function in JavaScript/Svelte to call Rust functions
3. Configure Tauri in `src-tauri/tauri.conf.json`
4. Ensure all desktop capabilities are properly defined in `src-tauri/capabilities/`
5. Follow Tauri best practices for security and performance

### Iroh & P2P Networking
1. P2P connections should use Iroh for establishing direct connections when possible
2. Implement proper error handling for relay server fallback scenarios
3. Ensure all communications are encrypted end-to-end using QUIC
4. Refer to the Iroh documentation for specific protocol implementations

### Frontend (Svelte)
1. Frontend code is in `src/` directory
2. Use SvelteKit routing in `src/routes/` - file-based routing is automatic
3. Maintain TypeScript type safety throughout the frontend
4. Use shadcn-svelte components for UI consistency and customization
5. Keep components small and reusable
6. Use Svelte stores for state management when needed
7. Leverage Tailwind CSS for styling (configured with shadcn-svelte)
