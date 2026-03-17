<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";

  let loading = $state(true);
  let config = $state<{ api_key: string; theme: string; courses_dir: string } | null>(null);

  onMount(async () => {
    const exists = await invoke<boolean>("config_exists");
    const c = await invoke<{ api_key: string; theme: string; courses_dir: string }>("load_config");
    document.documentElement.setAttribute("data-theme", c.theme || "dark");

    if (exists) {
      goto("/courses");
      return;
    }

    config = c;
    loading = false;
  });
</script>

{#if loading}
  <div class="loader">
    <div class="spinner"></div>
  </div>
{:else if config}
  <main class="setup">
    <div class="card">
      <div class="header">
        <div class="logo-mark">🐓</div>
        <h1>Rooster Learner</h1>
        <p class="subtitle">Configure your learning environment</p>
      </div>

      <SettingsPanel
        apiKey={config.api_key}
        theme={config.theme === "light" ? "light" : "dark"}
        coursesDir={config.courses_dir}
        onsave={() => goto("/courses")}
      />
    </div>
  </main>
{/if}

<style>
  .loader {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg);
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 2.5px solid var(--input-border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  .setup {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    background: var(--bg);
  }

  .card {
    width: 100%;
    max-width: 440px;
    background: var(--bg-card);
    border: 1px solid var(--input-border);
    border-radius: var(--radius);
    padding: 2.5rem 2rem;
  }

  .header {
    text-align: center;
    margin-bottom: 2rem;
  }

  .logo-mark {
    font-size: 2.5rem;
    margin-bottom: 0.25rem;
  }

  h1 {
    font-size: 1.4rem;
    font-weight: 700;
    letter-spacing: -0.025em;
  }

  .subtitle {
    color: var(--text-muted);
    font-size: 0.85rem;
    margin-top: 0.25rem;
  }
</style>
