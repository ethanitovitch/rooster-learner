Tauri 2 + SvelteKit 2 + Svelte 5 app. An LLM harness for building courses and learning with Claude.

## Architecture

- **Frontend**: SvelteKit (static adapter, SPA mode), TypeScript
- **Backend**: Tauri 2 (Rust), plugins: dialog (folder picker), opener
- **Build**: Vite 6, pnpm
- **Config**: JSON at `~/Library/Application Support/com.roosterlearner.app/config.json`
  - `api_key`: Claude API key (stored in plaintext)
  - `theme`: "light" | "dark"
  - `courses_dir`: path to courses folder (default `~/Documents/RoosterCourses`)

## File Structure

- `src/styles/global.css` — CSS variables (dark/light theme), reset, base input styles
- `src/routes/+layout.svelte` — imports global CSS
- `src/routes/+page.svelte` — setup/onboarding page, auto-redirects to /courses if API key exists
- `src/routes/courses/+page.svelte` — main app: sidebar with course list + settings/build buttons, main content area with course view, settings panel, or chat
- `src/lib/components/SettingsPanel.svelte` — reusable settings form (API key, theme toggle, course dir picker)
- `src-tauri/src/lib.rs` — Rust commands: load_config, save_config, get_default_courses_dir, list_courses

## Course Format

Courses live in the configured `courses_dir`. Each course is a folder containing:
- `course.yaml` — manifest with title, description, and lessons array (each with name, file, completed)
- HTML files for each lesson

## Rust Commands

- `load_config` / `save_config` — read/write config JSON
- `get_default_courses_dir` — returns `~/Documents/RoosterCourses`
- `list_courses` — scans courses_dir for folders with course.yaml, returns summaries

## Design

- Colors: black/white/orange (#f97316 dark, #ea580c light)
- Dark mode default, toggle in settings
