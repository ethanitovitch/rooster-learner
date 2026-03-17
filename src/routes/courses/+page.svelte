<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";

  interface CourseSummary {
    id: string;
    title: string;
    description: string;
    total_lessons: number;
    completed_lessons: number;
  }

  type View = "courses" | "settings" | "chat";

  let courses = $state<CourseSummary[]>([]);
  let selectedCourse = $state<CourseSummary | null>(null);
  let view = $state<View>("courses");
  let loading = $state(true);

  let chatMessages = $state<{ role: "user" | "assistant"; content: string }[]>([]);
  let chatInput = $state("");
  let chatLoading = $state(false);

  let config = $state<{ api_key: string; theme: string; courses_dir: string }>({
    api_key: "",
    theme: "dark",
    courses_dir: "",
  });

  onMount(async () => {
    config = await invoke<typeof config>("load_config");
    document.documentElement.setAttribute("data-theme", config.theme || "dark");

    courses = await invoke<CourseSummary[]>("list_courses");
    loading = false;
  });

  function selectCourse(course: CourseSummary) {
    selectedCourse = course;
    view = "courses";
  }

  function openSettings() {
    view = "settings";
    selectedCourse = null;
  }

  function openChat() {
    view = "chat";
    selectedCourse = null;
    if (chatMessages.length === 0) {
      chatMessages = [
        {
          role: "assistant",
          content:
            "What would you like to learn? Describe a topic and I'll build a course for you.",
        },
      ];
    }
  }

  async function sendMessage() {
    const msg = chatInput.trim();
    if (!msg || chatLoading) return;

    chatMessages = [...chatMessages, { role: "user", content: msg }];
    chatInput = "";
    chatLoading = true;

    // TODO: integrate Claude API to generate course
    chatMessages = [
      ...chatMessages,
      {
        role: "assistant",
        content: "Course generation coming soon! I'll use Claude to build your course here.",
      },
    ];
    chatLoading = false;
  }

  function handleChatKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  async function onSettingsSaved() {
    config = await invoke<typeof config>("load_config");
    courses = await invoke<CourseSummary[]>("list_courses");
    view = "courses";
  }
</script>

<div class="layout">
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-logo">🐓</span>
      <h2>Courses</h2>
    </div>

    <div class="course-list">
      {#if loading}
        <div class="sidebar-empty">Loading...</div>
      {:else if courses.length === 0}
        <div class="sidebar-empty">No courses yet</div>
      {:else}
        {#each courses as course}
          <button
            class="course-item"
            class:active={selectedCourse?.id === course.id && view === "courses"}
            onclick={() => selectCourse(course)}
          >
            <span class="course-title">{course.title}</span>
            <span class="course-progress">
              {course.completed_lessons}/{course.total_lessons}
            </span>
          </button>
        {/each}
      {/if}
    </div>

    <div class="sidebar-footer">
      <button class="build-btn" class:active={view === "chat"} onclick={openChat}>
        + Build Course
      </button>
      <button class="settings-btn" class:active={view === "settings"} onclick={openSettings}>
        Settings
      </button>
    </div>
  </aside>

  <main class="content">
    {#if view === "settings"}
      <div class="settings-view">
        <h1>Settings</h1>
        <SettingsPanel
          apiKey={config.api_key}
          theme={config.theme === "light" ? "light" : "dark"}
          coursesDir={config.courses_dir}
          saveLabel="Save"
          showCancel={true}
          onsave={onSettingsSaved}
          oncancel={() => (view = "courses")}
        />
      </div>
    {:else if view === "chat"}
      <div class="chat-view">
        <div class="chat-header">
          <h1>Build a Course</h1>
        </div>
        <div class="chat-messages">
          {#each chatMessages as msg}
            <div class="chat-msg" class:user={msg.role === "user"}>
              <div class="msg-bubble">
                {msg.content}
              </div>
            </div>
          {/each}
          {#if chatLoading}
            <div class="chat-msg">
              <div class="msg-bubble typing">
                <span class="dot"></span>
                <span class="dot"></span>
                <span class="dot"></span>
              </div>
            </div>
          {/if}
        </div>
        <div class="chat-input-row">
          <textarea
            bind:value={chatInput}
            placeholder="Describe what you want to learn..."
            onkeydown={handleChatKey}
            rows="1"
          ></textarea>
          <button class="send-btn" onclick={sendMessage} disabled={!chatInput.trim() || chatLoading}>
            Send
          </button>
        </div>
      </div>
    {:else if selectedCourse}
      <div class="course-view">
        <h1>{selectedCourse.title}</h1>
        {#if selectedCourse.description}
          <p class="course-desc">{selectedCourse.description}</p>
        {/if}
        <div class="progress-bar">
          <div
            class="progress-fill"
            style="width: {selectedCourse.total_lessons > 0
              ? (selectedCourse.completed_lessons / selectedCourse.total_lessons) * 100
              : 0}%"
          ></div>
        </div>
        <span class="progress-label">
          {selectedCourse.completed_lessons} of {selectedCourse.total_lessons} lessons completed
        </span>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">📚</div>
        <h2>
          {#if courses.length === 0}
            Build your first course
          {:else}
            Select a course
          {/if}
        </h2>
        <p>
          {#if courses.length === 0}
            Click "Build Course" to generate a new course with Claude
          {:else}
            Pick a course from the sidebar to get started
          {/if}
        </p>
      </div>
    {/if}
  </main>
</div>

<style>
  .layout {
    display: flex;
    height: 100vh;
    background: var(--bg);
  }

  .sidebar {
    width: 260px;
    min-width: 260px;
    background: var(--bg-card);
    border-right: 1px solid var(--input-border);
    display: flex;
    flex-direction: column;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0 1rem;
    height: 52px;
    border-bottom: 1px solid var(--input-border);
  }

  .sidebar-logo {
    font-size: 1.25rem;
  }

  .sidebar-header h2 {
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .course-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .sidebar-empty {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.8rem;
  }

  .course-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    background: none;
    border: none;
    border-radius: 6px;
    color: var(--text);
    font-size: 0.85rem;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }

  .course-item:hover {
    background: var(--input-bg);
  }

  .course-item.active {
    background: var(--accent);
    color: #fff;
  }

  .course-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .course-progress {
    font-size: 0.7rem;
    color: var(--text-muted);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .course-item.active .course-progress {
    color: rgba(255, 255, 255, 0.7);
  }

  .sidebar-footer {
    padding: 0.75rem;
    border-top: 1px solid var(--input-border);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .build-btn {
    width: 100%;
    padding: 0.55rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .build-btn:hover {
    background: var(--accent-hover);
  }

  .build-btn.active {
    background: var(--accent-hover);
    box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.2);
  }

  .settings-btn {
    width: 100%;
    padding: 0.45rem;
    background: none;
    border: 1px solid var(--input-border);
    border-radius: 6px;
    color: var(--text-muted);
    font-size: 0.75rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .settings-btn:hover {
    border-color: var(--accent);
    color: var(--text);
  }

  .settings-btn.active {
    border-color: var(--accent);
    color: var(--accent);
  }

  .content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  /* Settings */
  .settings-view {
    max-width: 440px;
    padding: 2rem;
  }

  .settings-view h1 {
    font-size: 1.3rem;
    font-weight: 700;
    margin-bottom: 1.5rem;
  }

  /* Chat */
  .chat-view {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .chat-header {
    display: flex;
    align-items: center;
    padding: 0 1rem;
    height: 52px;
    border-bottom: 1px solid var(--input-border);
  }

  .chat-header h1 {
    font-size: 1.1rem;
    font-weight: 700;
  }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .chat-msg {
    display: flex;
  }

  .chat-msg.user {
    justify-content: flex-end;
  }

  .msg-bubble {
    max-width: 75%;
    padding: 0.65rem 0.9rem;
    border-radius: 12px;
    font-size: 0.875rem;
    line-height: 1.5;
    background: var(--input-bg);
    color: var(--text);
  }

  .chat-msg.user .msg-bubble {
    background: var(--accent);
    color: #fff;
  }

  .msg-bubble.typing {
    display: flex;
    gap: 4px;
    align-items: center;
    padding: 0.75rem 1rem;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-muted);
    animation: bounce 1.2s infinite;
  }

  .dot:nth-child(2) {
    animation-delay: 0.15s;
  }

  .dot:nth-child(3) {
    animation-delay: 0.3s;
  }

  @keyframes bounce {
    0%,
    60%,
    100% {
      transform: translateY(0);
    }
    30% {
      transform: translateY(-4px);
    }
  }

  .chat-input-row {
    display: flex;
    gap: 0.5rem;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--input-border);
  }

  .chat-input-row textarea {
    flex: 1;
    resize: none;
    padding: 0.6rem 0.75rem;
    background: var(--input-bg);
    border: 1px solid var(--input-border);
    border-radius: 8px;
    color: var(--text);
    font-size: 0.875rem;
    font-family: var(--font);
    outline: none;
    transition: border-color 0.15s;
    min-height: 38px;
    max-height: 120px;
  }

  .chat-input-row textarea:focus {
    border-color: var(--input-focus);
  }

  .chat-input-row textarea::placeholder {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .send-btn {
    padding: 0 1rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
    white-space: nowrap;
    align-self: flex-end;
    height: 38px;
  }

  .send-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .send-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* Course detail */
  .course-view {
    max-width: 640px;
    padding: 2rem;
  }

  .course-view h1 {
    font-size: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-bottom: 0.5rem;
  }

  .course-desc {
    color: var(--text-muted);
    font-size: 0.9rem;
    margin-bottom: 1.5rem;
    line-height: 1.6;
  }

  .progress-bar {
    width: 100%;
    height: 6px;
    background: var(--input-bg);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 3px;
    transition: width 0.3s;
  }

  .progress-label {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  /* Empty */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    gap: 0.5rem;
    padding: 2rem;
  }

  .empty-icon {
    font-size: 3rem;
    margin-bottom: 0.5rem;
  }

  .empty-state h2 {
    font-size: 1.2rem;
    font-weight: 700;
  }

  .empty-state p {
    color: var(--text-muted);
    font-size: 0.85rem;
    max-width: 300px;
  }
</style>
