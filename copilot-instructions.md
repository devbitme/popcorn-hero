# Copilot Instructions for Popcorn Hero

## Project Overview

This project is a **Tauri desktop application** that uses **Iroh** for establishing direct peer-to-peer connections with fallback to relay servers when necessary. All connections are authenticated and encrypted end-to-end using QUIC.

The frontend is built with **Svelte** and TypeScript, while the backend leverages **Rust** through Tauri's architecture.

## Iroh Documentation Reference

### What is Iroh?

Iroh establishes direct peer-to-peer connections whenever possible, falling back to relay servers if necessary. All connections are authenticated and encrypted end-to-end using QUIC. It is built in Rust and used natively in this Tauri application.

**Official Documentation**: https://docs.iroh.computer/

### Key Resources

- **[Quickstart Guide](https://docs.iroh.computer/quickstart)** - Build a ping program connecting two endpoints using tickets over DNS
- **[Overview](https://docs.iroh.computer/overview)** - High level look at the problems Iroh solves
- **[Protocol Registry](https://www.iroh.computer/proto)** - Iroh Protocols are pluggable extensions that build on direct connections
- **[Awesome List](https://github.com/n0-computer/awesome-iroh)** - Projects & resources building with Iroh
- **[FAQ](https://docs.iroh.computer/faq)** - Frequently asked questions
- **[GitHub Repository](https://github.com/n0-computer/iroh)** - Iroh source code and community
- **[Discord Community](https://iroh.computer/discord)** - Join the Iroh community

### Core Concepts

#### Endpoints
An `iroh::Endpoint` is the main entry point. It manages networking, maintains a relay connection, and finds ways to address devices by `EndpointId`.

```rust
let endpoint = Endpoint::bind().await?;
endpoint.online().await; // Wait until reachable by others
```

#### Tickets (iroh-tickets)
An `EndpointTicket` wraps an endpoint's address (node ID, relay URL, direct addresses) into a serializable string for sharing. Users copy-paste this ticket to connect to each other.

```rust
use iroh_tickets::{Ticket, endpoint::EndpointTicket};

// Generate a ticket from your endpoint
let ticket = EndpointTicket::new(endpoint.addr());
println!("{ticket}"); // Share this string

// Parse a ticket received from another user
let ticket = EndpointTicket::deserialize(&ticket_str)?;
let addr = ticket.endpoint_addr().clone();
```

**When to use tickets**: Bootstrapping peer connections without a central server, QR codes, copy/paste flows, short-lived sessions.

**When NOT to use tickets**: Long-lived connections where dialing details change. Prefer caching `EndpointID`s and let iroh resolve at runtime via discovery.

#### Discovery
Discovery resolves `EndpointID`s to addresses. Iroh supports:
- **DNS Discovery** (enabled by default) — resolves via `dns.iroh.link`
- **Pkarr** (enabled via DNS) — public-key addressable DNS records
- **Local/mDNS** (disabled) — local network discovery
- **DHT** (disabled) — BitTorrent Mainline DHT

#### Protocols & Router
Protocols define how two endpoints exchange messages, identified by an ALPN string. The `Router` dispatches incoming connections to the right protocol handler.

```rust
use iroh::protocol::Router;
use iroh_ping::Ping;

let ping = Ping::new();
let router = Router::builder(endpoint)
    .accept(iroh_ping::ALPN, ping)
    .spawn();
```

#### Ping Protocol (iroh-ping)
Lightweight diagnostic protocol to prove connectivity and measure round-trip latency between two endpoints.

```rust
let pinger = Ping::new();
let rtt = pinger.ping(&endpoint, remote_addr).await?;
```

### Rust Crates Used

| Crate | Purpose |
|-------|---------|
| `iroh` | Core endpoint, connections, discovery |
| `iroh-ping` | Diagnostic ping/pong protocol |
| `iroh-tickets` | Ticket creation and parsing |
| `tokio` | Async runtime required by iroh |

### Security Notes

- Tickets contain IP addresses — sharing a ticket means sharing your IPs
- Tickets are reusable, not single-use tokens
- All connections are end-to-end encrypted via QUIC
- EndpointIDs are elliptic curve public keys

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

> shadcn-svelte is a collection of beautifully-designed, accessible components for Svelte and SvelteKit. It is built with TypeScript, Tailwind CSS, and Bits UI primitives. Open Source. Open Code. AI-Ready. It also comes with a command-line tool to install and manage components and a registry system to publish and distribute code.

### Overview

- [About](https://shadcn-svelte.com/docs/about.md): Powered by amazing open source projects.
- [Changelog](https://shadcn-svelte.com/docs/changelog.md): Latest updates and announcements.
- [shadcn-svelte](https://shadcn-svelte.com/docs/cli.md): Use the shadcn-svelte CLI to add components to your project.
- [components.json](https://shadcn-svelte.com/docs/components-json.md): Configuration for your project.
- [JavaScript](https://shadcn-svelte.com/docs/javascript.md): How to use shadcn-svelte with JavaScript.
- [Legacy Docs](https://shadcn-svelte.com/docs/legacy.md): View the legacy docs for shadcn-svelte and Tailwind v3.
- [Theming](https://shadcn-svelte.com/docs/theming.md): Use CSS Variables to customize the look and feel of your application.

### Installation

- [Astro](https://shadcn-svelte.com/docs/installation/astro.md): How to setup shadcn-svelte in an Astro project.
- [Manual Installation](https://shadcn-svelte.com/docs/installation/manual.md): How to setup shadcn-svelte manually.
- [SvelteKit](https://shadcn-svelte.com/docs/installation/sveltekit.md): How to setup shadcn-svelte in a SvelteKit project.
- [Vite](https://shadcn-svelte.com/docs/installation/vite.md): How to setup shadcn-svelte in a Vite project.

### Components

#### Form & Input

- [Button](https://shadcn-svelte.com/docs/components/button.md): Displays a button or a component that looks like a button.
- [Button Group](https://shadcn-svelte.com/docs/components/button-group.md): A container that groups related buttons together with consistent styling.
- [Calendar](https://shadcn-svelte.com/docs/components/calendar.md): A calendar component that allows users to select dates.
- [Checkbox](https://shadcn-svelte.com/docs/components/checkbox.md): A control that allows the user to toggle between checked and not checked.
- [Combobox](https://shadcn-svelte.com/docs/components/combobox.md): Autocomplete input and command palette with a list of suggestions.
- [Date Picker](https://shadcn-svelte.com/docs/components/date-picker.md): A date picker component with range and presets.
- [Field](https://shadcn-svelte.com/docs/components/field.md): Combine labels, controls, and help text to compose accessible form fields and grouped inputs.
- [Formsnap](https://shadcn-svelte.com/docs/components/form.md): Building forms with Formsnap, Superforms, & Zod.
- [Input](https://shadcn-svelte.com/docs/components/input.md): Displays a form input field or a component that looks like an input field.
- [Input Group](https://shadcn-svelte.com/docs/components/input-group.md): Display additional information or actions to an input or textarea.
- [Input OTP](https://shadcn-svelte.com/docs/components/input-otp.md): Accessible one-time password component with copy paste functionality.
- [Label](https://shadcn-svelte.com/docs/components/label.md): Renders an accessible label associated with controls.
- [Native Select](https://shadcn-svelte.com/docs/components/native-select.md): A styled native HTML select element with consistent design system integration.
- [Radio Group](https://shadcn-svelte.com/docs/components/radio-group.md): A set of checkable buttonsâ€”known as radio buttonsâ€”where no more than one of the buttons can be checked at a time.
- [Select](https://shadcn-svelte.com/docs/components/select.md): Displays a list of options for the user to pick fromâ€”triggered by a button.
- [Slider](https://shadcn-svelte.com/docs/components/slider.md): An input where the user selects a value from within a given range.
- [Switch](https://shadcn-svelte.com/docs/components/switch.md): A control that allows the user to toggle between checked and not checked.
- [Textarea](https://shadcn-svelte.com/docs/components/textarea.md): Displays a form textarea or a component that looks like a textarea.

#### Layout & Navigation

- [Accordion](https://shadcn-svelte.com/docs/components/accordion.md): A vertically stacked set of interactive headings that each reveal a section of content.
- [Breadcrumb](https://shadcn-svelte.com/docs/components/breadcrumb.md): Displays the path to the current resource using a hierarchy of links.
- [Navigation Menu](https://shadcn-svelte.com/docs/components/navigation-menu.md): A collection of links for navigating websites.
- [Resizable](https://shadcn-svelte.com/docs/components/resizable.md): Accessible resizable panel groups and layouts with keyboard support.
- [Scroll Area](https://shadcn-svelte.com/docs/components/scroll-area.md): Augments native scroll functionality for custom, cross-browser styling.
- [Separator](https://shadcn-svelte.com/docs/components/separator.md): Visually or semantically separates content.
- [Sidebar](https://shadcn-svelte.com/docs/components/sidebar.md): A composable, themeable and customizable sidebar component.
- [Tabs](https://shadcn-svelte.com/docs/components/tabs.md): A set of layered sections of contentâ€”known as tab panelsâ€”that are displayed one at a time.

#### Overlays & Dialogs

- [Alert Dialog](https://shadcn-svelte.com/docs/components/alert-dialog.md): A modal dialog that interrupts the user with important content and expects a response.
- [Command](https://shadcn-svelte.com/docs/components/command.md): Fast, composable, unstyled command menu for Svelte.
- [Context Menu](https://shadcn-svelte.com/docs/components/context-menu.md): Displays a menu to the user â€” such as a set of actions or functions â€” triggered by right click.
- [Dialog](https://shadcn-svelte.com/docs/components/dialog.md): A window overlaid on either the primary window or another dialog window, rendering the content underneath inert.
- [Drawer](https://shadcn-svelte.com/docs/components/drawer.md): A drawer component for Svelte.
- [Dropdown Menu](https://shadcn-svelte.com/docs/components/dropdown-menu.md): Displays a menu to the user â€” such as a set of actions or functions â€” triggered by a button.
- [Hover Card](https://shadcn-svelte.com/docs/components/hover-card.md): For sighted users to preview content available behind a link.
- [Menubar](https://shadcn-svelte.com/docs/components/menubar.md): A visually persistent menu common in desktop applications that provides quick access to a consistent set of commands.
- [Popover](https://shadcn-svelte.com/docs/components/popover.md): Displays rich content in a portal, triggered by a button.
- [Sheet](https://shadcn-svelte.com/docs/components/sheet.md): Extends the Dialog component to display content that complements the main content of the screen.
- [Tooltip](https://shadcn-svelte.com/docs/components/tooltip.md): A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.

#### Feedback & Status

- [Alert](https://shadcn-svelte.com/docs/components/alert.md): Displays a callout for user attention.
- [Badge](https://shadcn-svelte.com/docs/components/badge.md): Displays a badge or a component that looks like a badge.
- [Empty](https://shadcn-svelte.com/docs/components/empty.md): Use the Empty component to display a empty state.
- [Progress](https://shadcn-svelte.com/docs/components/progress.md): Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.
- [Skeleton](https://shadcn-svelte.com/docs/components/skeleton.md): Use to show a placeholder while content is loading.
- [Sonner](https://shadcn-svelte.com/docs/components/sonner.md): An opinionated toast component for Svelte.
- [Spinner](https://shadcn-svelte.com/docs/components/spinner.md): An indicator that can be used to show a loading state.

#### Display & Media

- [Aspect Ratio](https://shadcn-svelte.com/docs/components/aspect-ratio.md): Displays content within a desired ratio.
- [Avatar](https://shadcn-svelte.com/docs/components/avatar.md): An image element with a fallback for representing the user.
- [Card](https://shadcn-svelte.com/docs/components/card.md): Displays a card with header, content, and footer.
- [Carousel](https://shadcn-svelte.com/docs/components/carousel.md): A carousel with motion and swipe built using Embla.
- [Chart](https://shadcn-svelte.com/docs/components/chart.md): Beautiful charts. Built using LayerChart. Copy and paste into your apps.
- [Data Table](https://shadcn-svelte.com/docs/components/data-table.md): Powerful table and datagrids built using TanStack Table.
- [Item](https://shadcn-svelte.com/docs/components/item.md): A versatile component that you can use to display any content.
- [Kbd](https://shadcn-svelte.com/docs/components/kbd.md): Used to display textual user input from keyboard.
- [Table](https://shadcn-svelte.com/docs/components/table.md): A responsive table component.
- [Typography](https://shadcn-svelte.com/docs/components/typography.md): Styles for headings, paragraphs, lists...etc

#### Misc

- [Collapsible](https://shadcn-svelte.com/docs/components/collapsible.md): An interactive component which expands/collapses a panel.
- [Pagination](https://shadcn-svelte.com/docs/components/pagination.md): Pagination with page navigation, next and previous links.
- [Range Calendar](https://shadcn-svelte.com/docs/components/range-calendar.md): A calendar component that allows users to select a range of dates.
- [Toggle](https://shadcn-svelte.com/docs/components/toggle.md): A two-state button that can be either on or off.
- [Toggle Group](https://shadcn-svelte.com/docs/components/toggle-group.md): A set of two-state buttons that can be toggled on or off.

### Dark Mode

- [Astro](https://shadcn-svelte.com/docs/dark-mode/astro.md): Adding dark mode to your Astro site.
- [Svelte](https://shadcn-svelte.com/docs/dark-mode/svelte.md): Adding dark mode to your Svelte site.

### Migration

- [Svelte 5](https://shadcn-svelte.com/docs/migration/svelte-5.md): How to migrate from Svelte 4 and Tailwind 3 to Svelte 5.
- [Tailwind v4](https://shadcn-svelte.com/docs/migration/tailwind-v4.md): How to use shadcn-svelte with Tailwind v4 and Svelte 5.

### Registry

- [Examples](https://shadcn-svelte.com/docs/registry/examples.md): Examples of registry items: styles, components, css vars, etc.
- [FAQ](https://shadcn-svelte.com/docs/registry/faq.md): Frequently asked questions about running a registry.
- [Getting Started](https://shadcn-svelte.com/docs/registry/getting-started.md): Learn how to get setup and run your own component registry.
- [registry-item.json](https://shadcn-svelte.com/docs/registry/registry-item-json.md): Specification for registry items.
- [registry.json](https://shadcn-svelte.com/docs/registry/registry-json.md): Schema for running your own component registry.

## TMDB API Documentation Reference

**Full Documentation**: https://developer.themoviedb.org/docs/getting-started
**API Reference**: https://developer.themoviedb.org/reference

### Authentication

Two methods are available (both provide the same access level):

1. **API Key (v3)** — query parameter:
   ```
   GET https://api.themoviedb.org/3/movie/11?api_key=YOUR_KEY
   ```
2. **Bearer Token (v3+v4)** — header (preferred by TMDB docs):
   ```
   GET https://api.themoviedb.org/3/movie/11
   Authorization: Bearer <<access_token>>
   ```

The project currently uses method 1 (`api_key` query param). Both are fully supported.

### Endpoints Used in This Project

#### Search Movie
- **GET** `https://api.themoviedb.org/3/search/movie`
- Query params: `api_key`, `query` (required), `year`, `include_adult` (default false), `language` (default en-US), `page` (default 1), `primary_release_year`, `region`
- Returns: `{ page, results: [{ id, title, original_title, overview, release_date, vote_average, vote_count, genre_ids, poster_path, backdrop_path, original_language, ... }], total_pages, total_results }`
- Ref: https://developer.themoviedb.org/reference/search-movie

#### Movie Details
- **GET** `https://api.themoviedb.org/3/movie/{movie_id}`
- Query params: `api_key`, `append_to_response` (comma-separated, max 20 items), `language` (default en-US)
- `append_to_response=credits` appends cast and crew data in the same request
- Returns: `{ id, title, original_title, overview, tagline, release_date, runtime, vote_average, vote_count, genres, poster_path, backdrop_path, imdb_id, original_language, status, production_companies, credits: { cast, crew } }`
- Ref: https://developer.themoviedb.org/reference/movie-details

#### TV Series Search
- **GET** `https://api.themoviedb.org/3/search/tv`
- Query params: `api_key`, `query` (required), `first_air_date_year`, `language`, `page`
- Returns: `{ results: [{ id, name, original_name, overview, first_air_date, poster_path, backdrop_path, ... }] }`
- Ref: https://developer.themoviedb.org/reference/search-tv

#### TV Series Details
- **GET** `https://api.themoviedb.org/3/tv/{series_id}`
- Query params: `api_key`, `append_to_response` (e.g. `credits`), `language`
- Returns: `{ id, name, original_name, overview, tagline, first_air_date, genres, poster_path, backdrop_path, credits, ... }`
- Ref: https://developer.themoviedb.org/reference/tv-series-details

#### TV Episode Details
- **GET** `https://api.themoviedb.org/3/tv/{series_id}/season/{season_number}/episode/{episode_number}`
- Query params: `api_key`, `language`
- Returns: `{ name, overview, still_path, air_date, episode_number, season_number, vote_average, ... }`
- Used to get the episode title, pitch/overview, and still image for individual episodes
- Ref: https://developer.themoviedb.org/reference/tv-episode-details

#### Append To Response
- Available on movie, TV show, TV season, TV episode and person detail methods
- Comma-separated list of sub-endpoints (e.g. `credits,videos,images`), max 20 items
- Each sub-request responds to its own query parameters (e.g. `include_image_language` for images)
- Ref: https://developer.themoviedb.org/docs/append-to-response

### Image URLs

To build a full image URL, combine 3 pieces:
```
https://image.tmdb.org/t/p/{size}/{file_path}
```
- **Base URL**: `https://image.tmdb.org/t/p`
- **Poster sizes**: `w92`, `w154`, `w185`, `w342`, `w500`, `w780`, `original`
- **Backdrop sizes**: `w300`, `w780`, `w1280`, `original`
- **Profile sizes**: `w45`, `w185`, `h632`, `original`
- Example: `https://image.tmdb.org/t/p/w500/1E5baAaEse26fej7uHcjOgEE2t2.jpg`
- Ref: https://developer.themoviedb.org/docs/image-basics

### Configuration & Limits

- **Rate Limiting**: documented at https://developer.themoviedb.org/docs/rate-limiting
- **Errors**: documented at https://developer.themoviedb.org/docs/errors
- **API Key registration**: https://www.themoviedb.org/settings/api
- The API key is injected at compile time via `TMDB_API_KEY` env var (see `.env` / GitHub Secrets)

## Development Guidelines

When working on this project, keep in mind:

### Tauri & Desktop Development
1. Backend logic should be implemented in Rust (in `src-tauri/src/`)
2. Use the `invoke` function in JavaScript/Svelte to call Rust functions
3. Configure Tauri in `src-tauri/tauri.conf.json`
4. Ensure all desktop capabilities are properly defined in `src-tauri/capabilities/`
5. Follow Tauri best practices for security and performance

### Logging
All actions performed by the application must be systematically logged using `tauri-plugin-log`:

1. **Frontend (Svelte/TypeScript)**: Use `info`, `warn`, `error`, `debug`, `trace` from `@tauri-apps/plugin-log` to log every significant action (navigation, user interactions, API calls, state changes, errors)
2. **Backend (Rust)**: Use the `log` crate macros (`log::info!`, `log::warn!`, `log::error!`, `log::debug!`, `log::trace!`) to log every command execution, database operation, and error
3. **Log format**: Prefix log messages with the component/module name in brackets, e.g. `[DevToolbar] Toolbar pinned`, `[LanguageSwitcher] Changing locale from en to fr`
4. **Log targets**: Logs are persisted to the application log directory (LogDir), printed to stdout, and forwarded to the webview console via `attachConsole()`
5. **Error handling**: Always log errors with `warn` or `error` level before handling them
6. **New features**: When adding any new feature, action, or command, always include appropriate log statements

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
