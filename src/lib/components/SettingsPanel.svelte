<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  interface Props {
    apiKey?: string;
    theme?: "light" | "dark";
    coursesDir?: string;
    saveLabel?: string;
    showCancel?: boolean;
    onsave?: () => void;
    oncancel?: () => void;
  }

  let {
    apiKey: initialKey = "",
    theme: initialTheme = "dark",
    coursesDir: initialDir = "",
    saveLabel = "Save & Continue",
    showCancel = false,
    onsave,
    oncancel,
  }: Props = $props();

  // svelte-ignore state_referenced_locally
  let apiKey = $state(initialKey);
  // svelte-ignore state_referenced_locally
  let theme = $state<"light" | "dark">(initialTheme);
  // svelte-ignore state_referenced_locally
  let coursesDir = $state(initialDir);
  let defaultDir = $state("");
  let saved = $state(false);
  let ready = $state(false);

  $effect(() => {
    invoke<string>("get_default_courses_dir").then((dir) => {
      defaultDir = dir;
      if (!coursesDir) coursesDir = dir;
      ready = true;
    });
  });

  function applyTheme(t: "light" | "dark") {
    document.documentElement.setAttribute("data-theme", t);
  }

  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
    applyTheme(theme);
  }

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      coursesDir = selected as string;
    }
  }

  async function save() {
    await invoke("save_config", {
      config: { api_key: apiKey, theme, courses_dir: coursesDir },
    });
    saved = true;
    setTimeout(() => {
      saved = false;
      onsave?.();
    }, 400);
  }

  function useDefault() {
    coursesDir = defaultDir;
  }
</script>

{#if ready}
  <div class="fields">
    <label class="field">
      <span class="label">Claude API Key</span>
      <div class="input-wrap">
        <input
          type="password"
          bind:value={apiKey}
          placeholder="sk-ant-..."
          spellcheck="false"
          autocomplete="off"
        />
      </div>
      <span class="hint">Stored locally in app config</span>
    </label>

    <div class="field">
      <span class="label">Theme</span>
      <button class="theme-toggle" onclick={toggleTheme}>
        <span class="toggle-track" class:light={theme === "light"}>
          <span class="toggle-icon moon">🌙</span>
          <span class="toggle-thumb"></span>
          <span class="toggle-icon sun">☀️</span>
        </span>
      </button>
      <span class="hint">{theme === "dark" ? "Dark mode" : "Light mode"}</span>
    </div>

    <label class="field">
      <span class="label">Course Storage</span>
      <div class="path-row">
        <input
          type="text"
          bind:value={coursesDir}
          placeholder="Select a folder..."
          spellcheck="false"
        />
        <button class="browse-btn" onclick={pickFolder}>Browse</button>
      </div>
      {#if coursesDir !== defaultDir}
        <button class="use-default" onclick={useDefault}>
          Use default: {defaultDir}
        </button>
      {/if}
      <span class="hint">Where your course folders will live</span>
    </label>
  </div>

  <div class="actions">
    <button
      class="save-btn"
      onclick={save}
      disabled={!apiKey}
      class:saved
    >
      {#if saved}
        ✓ Saved
      {:else}
        {saveLabel}
      {/if}
    </button>
    {#if showCancel}
      <button class="cancel-btn" onclick={() => oncancel?.()}>Cancel</button>
    {/if}
  </div>
{/if}

<style>
  .fields {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    margin-bottom: 2rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .label {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }

  .hint {
    font-size: 0.7rem;
    color: var(--text-muted);
    opacity: 0.7;
  }

  .input-wrap {
    position: relative;
  }

  .path-row {
    display: flex;
    gap: 0.5rem;
  }

  .path-row input {
    flex: 1;
    min-width: 0;
  }

  .browse-btn {
    padding: 0.6rem 0.9rem;
    background: var(--input-bg);
    border: 1px solid var(--input-border);
    border-radius: 8px;
    color: var(--text);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: border-color 0.15s;
  }

  .browse-btn:hover {
    border-color: var(--accent);
  }

  .use-default {
    background: none;
    border: none;
    color: var(--accent);
    font-size: 0.7rem;
    cursor: pointer;
    text-align: left;
    padding: 0;
  }

  .use-default:hover {
    text-decoration: underline;
  }

  .theme-toggle {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    width: fit-content;
  }

  .toggle-track {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 68px;
    height: 34px;
    background: var(--input-bg);
    border: 1px solid var(--input-border);
    border-radius: 17px;
    position: relative;
    padding: 0 8px;
    transition: background 0.2s;
  }

  .toggle-thumb {
    position: absolute;
    width: 24px;
    height: 24px;
    background: var(--accent);
    border-radius: 50%;
    left: 4px;
    transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .toggle-track.light .toggle-thumb {
    transform: translateX(34px);
  }

  .toggle-icon {
    font-size: 0.85rem;
    z-index: 1;
    user-select: none;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .save-btn {
    width: 100%;
    padding: 0.7rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s, transform 0.1s;
  }

  .save-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .save-btn:active:not(:disabled) {
    transform: scale(0.98);
  }

  .save-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .save-btn.saved {
    background: #16a34a;
  }

  .cancel-btn {
    width: 100%;
    padding: 0.55rem;
    background: none;
    border: 1px solid var(--input-border);
    border-radius: 8px;
    color: var(--text-muted);
    font-size: 0.8rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .cancel-btn:hover {
    border-color: var(--accent);
    color: var(--text);
  }
</style>
