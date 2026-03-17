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

  interface Lesson {
    name: string;
    file: string;
    completed: boolean;
  }

  interface CourseDetail {
    id: string;
    title: string;
    description: string;
    lessons: Lesson[];
  }

  type View = "courses" | "settings" | "chat" | "course-detail";

  let courses = $state<CourseSummary[]>([]);
  let selectedCourse = $state<CourseSummary | null>(null);
  let courseDetail = $state<CourseDetail | null>(null);
  let activeLesson = $state<Lesson | null>(null);
  let lessonHtml = $state("");
  let view = $state<View>("courses");
  let loading = $state(true);

  interface ChatSummary {
    id: string;
    title: string;
    created_at: number;
    message_count: number;
  }

  interface ChatMsg {
    role: string;
    content: string;
  }

  interface Chat {
    id: string;
    title: string;
    created_at: number;
    messages: ChatMsg[];
  }

  let chatMessages = $state<{ role: "user" | "assistant"; content: string }[]>([]);
  let chatInput = $state("");
  let chatLoading = $state(false);

  let sidebarOpen = $state(true);
  let chatPanelOpen = $state(false);
  let lessonChats = $state<ChatSummary[]>([]);
  let activeChatId = $state<string | null>(null);
  let lessonChatMessages = $state<ChatMsg[]>([]);
  let lessonChatInput = $state("");
  let lessonChatLoading = $state(false);
  let showChatList = $state(false);

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

  async function selectCourse(course: CourseSummary) {
    selectedCourse = course;
    view = "course-detail";
    courseDetail = await invoke<CourseDetail>("get_course_detail", { courseId: course.id });
    activeLesson = null;
    lessonHtml = "";
  }

  async function openLesson(lesson: Lesson) {
    if (!courseDetail) return;
    activeLesson = lesson;
    lessonHtml = await invoke<string>("get_lesson_content", {
      courseId: courseDetail.id,
      file: lesson.file,
    });
    chatPanelOpen = false;
    activeChatId = null;
    lessonChatMessages = [];
    lessonChats = [];
    showChatList = false;
  }

  function backToCourse() {
    activeLesson = null;
    lessonHtml = "";
  }

  function goHome() {
    view = "courses";
    selectedCourse = null;
    courseDetail = null;
    activeLesson = null;
    lessonHtml = "";
  }

  async function completeAndNext() {
    if (!courseDetail || !activeLesson) return;

    await invoke("complete_lesson", {
      courseId: courseDetail.id,
      lessonFile: activeLesson.file,
    });

    const currentIdx = courseDetail.lessons.findIndex((l) => l.file === activeLesson!.file);
    courseDetail.lessons[currentIdx].completed = true;

    courses = await invoke<CourseSummary[]>("list_courses");

    const nextIdx = currentIdx + 1;
    if (nextIdx < courseDetail.lessons.length) {
      await openLesson(courseDetail.lessons[nextIdx]);
    } else {
      backToCourse();
    }
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

  async function toggleChatPanel() {
    chatPanelOpen = !chatPanelOpen;
    if (chatPanelOpen && courseDetail && activeLesson) {
      lessonChats = await invoke<ChatSummary[]>("list_lesson_chats", {
        courseId: courseDetail.id,
        lessonFile: activeLesson.file,
      });
      if (lessonChats.length > 0 && !activeChatId) {
        await loadLessonChat(lessonChats[0].id);
      } else if (lessonChats.length === 0) {
        await newLessonChat();
      }
    }
  }

  async function newLessonChat() {
    if (!courseDetail || !activeLesson) return;
    const id = `chat-${Date.now()}`;
    const chat: Chat = {
      id,
      title: "New chat",
      created_at: Math.floor(Date.now() / 1000),
      messages: [],
    };
    await invoke("save_chat", {
      courseId: courseDetail.id,
      lessonFile: activeLesson.file,
      chat,
    });
    activeChatId = id;
    lessonChatMessages = [];
    showChatList = false;
    lessonChats = await invoke<ChatSummary[]>("list_lesson_chats", {
      courseId: courseDetail.id,
      lessonFile: activeLesson.file,
    });
  }

  async function loadLessonChat(chatId: string) {
    if (!courseDetail || !activeLesson) return;
    const chat = await invoke<Chat>("load_chat", {
      courseId: courseDetail.id,
      lessonFile: activeLesson.file,
      chatId,
    });
    activeChatId = chat.id;
    lessonChatMessages = chat.messages;
    showChatList = false;
  }

  async function sendLessonChat() {
    const msg = lessonChatInput.trim();
    if (!msg || lessonChatLoading || !courseDetail || !activeLesson || !activeChatId) return;

    lessonChatMessages = [...lessonChatMessages, { role: "user", content: msg }];
    lessonChatInput = "";
    lessonChatLoading = true;

    // TODO: integrate Claude API
    lessonChatMessages = [
      ...lessonChatMessages,
      { role: "assistant", content: "Claude integration coming soon!" },
    ];
    lessonChatLoading = false;

    const title = lessonChatMessages[0]?.content.slice(0, 40) || "Chat";
    await invoke("save_chat", {
      courseId: courseDetail.id,
      lessonFile: activeLesson.file,
      chat: {
        id: activeChatId,
        title,
        created_at: Math.floor(Date.now() / 1000),
        messages: lessonChatMessages,
      },
    });
    lessonChats = await invoke<ChatSummary[]>("list_lesson_chats", {
      courseId: courseDetail.id,
      lessonFile: activeLesson.file,
    });
  }

  function handleLessonChatKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendLessonChat();
    }
  }

  async function onSettingsSaved() {
    config = await invoke<typeof config>("load_config");
    courses = await invoke<CourseSummary[]>("list_courses");
    view = "courses";
  }
</script>

<div class="layout">
  <aside class="sidebar" class:collapsed={!sidebarOpen}>
    <div class="sidebar-header">
      <button class="sidebar-home" onclick={goHome}>
        <span class="sidebar-logo">🐓</span>
        {#if sidebarOpen}<span>Courses</span>{/if}
      </button>
      <button class="sidebar-toggle" onclick={() => (sidebarOpen = !sidebarOpen)}>
        {sidebarOpen ? "◀" : "▶"}
      </button>
    </div>

    {#if sidebarOpen}
      <div class="course-list">
        {#if loading}
          <div class="sidebar-empty">Loading...</div>
        {:else if courses.length === 0}
          <div class="sidebar-empty">No courses yet</div>
        {:else}
          {#each courses as course}
            <button
              class="course-item"
              class:active={selectedCourse?.id === course.id && (view === "courses" || view === "course-detail")}
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
    {/if}
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
    {:else if view === "course-detail" && courseDetail}
      {#if activeLesson}
        <div class="lesson-layout">
          <div class="detail-view">
            <div class="lesson-header">
              <button class="back-btn" onclick={backToCourse}>← Back</button>
              <span class="lesson-title"></span>
              <button class="chat-toggle-btn" class:active={chatPanelOpen} onclick={toggleChatPanel}>
                💬
              </button>
            </div>
            <div class="lesson-content">
              {@html lessonHtml}
            </div>
            <div class="lesson-footer">
              <button class="next-btn" onclick={completeAndNext}>
                {#if courseDetail.lessons.findIndex((l) => l.file === activeLesson?.file) === courseDetail.lessons.length - 1}
                  Complete Course ✓
                {:else}
                  Complete & Next →
                {/if}
              </button>
            </div>
          </div>

          {#if chatPanelOpen}
            <div class="chat-panel">
              <div class="chat-panel-header">
                <div class="chat-panel-title">
                  {#if showChatList}
                    <button class="chat-back-btn" onclick={() => (showChatList = false)}>←</button>
                    <span>Chats</span>
                  {:else}
                    <span>Chat</span>
                  {/if}
                </div>
                <div class="chat-panel-actions">
                  <button class="chat-action-btn" onclick={() => (showChatList = !showChatList)} title="Chat history">
                    📋
                  </button>
                  <button class="chat-action-btn" onclick={newLessonChat} title="New chat">
                    +
                  </button>
                  <button class="chat-action-btn" onclick={toggleChatPanel} title="Close chat">
                    ✕
                  </button>
                </div>
              </div>

              {#if showChatList}
                <div class="chat-history">
                  {#if lessonChats.length === 0}
                    <div class="chat-history-empty">No chats yet</div>
                  {:else}
                    {#each lessonChats as chat}
                      <button
                        class="chat-history-item"
                        class:active={activeChatId === chat.id}
                        onclick={() => loadLessonChat(chat.id)}
                      >
                        <span class="chat-history-title">{chat.title}</span>
                        <span class="chat-history-count">{chat.message_count} msgs</span>
                      </button>
                    {/each}
                  {/if}
                </div>
              {:else}
                <div class="chat-panel-messages">
                  {#if lessonChatMessages.length === 0}
                    <div class="chat-panel-empty">Ask a question about this lesson</div>
                  {:else}
                    {#each lessonChatMessages as msg}
                      <div class="chat-msg" class:user={msg.role === "user"}>
                        <div class="msg-bubble">{msg.content}</div>
                      </div>
                    {/each}
                  {/if}
                  {#if lessonChatLoading}
                    <div class="chat-msg">
                      <div class="msg-bubble typing">
                        <span class="dot"></span>
                        <span class="dot"></span>
                        <span class="dot"></span>
                      </div>
                    </div>
                  {/if}
                </div>
                <div class="chat-panel-input">
                  <textarea
                    bind:value={lessonChatInput}
                    placeholder="Ask about this lesson..."
                    onkeydown={handleLessonChatKey}
                    rows="1"
                  ></textarea>
                  <button
                    class="send-btn"
                    onclick={sendLessonChat}
                    disabled={!lessonChatInput.trim() || lessonChatLoading}
                  >
                    Send
                  </button>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {:else}
        <div class="detail-view">
          <div class="detail-header">
            <h1>{courseDetail.title}</h1>
            {#if courseDetail.description}
              <p class="detail-desc">{courseDetail.description}</p>
            {/if}
          </div>
          <div class="lesson-list">
            {#each courseDetail.lessons as lesson, i}
              <button class="lesson-item" onclick={() => openLesson(lesson)}>
                <span class="lesson-num">{i + 1}</span>
                <span class="lesson-name">{lesson.name}</span>
                {#if lesson.completed}
                  <span class="lesson-check">✓</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    {:else if courses.length > 0}
      <div class="dashboard">
        <div class="dashboard-header">
          <h1>Your Courses</h1>
          <p class="dashboard-subtitle">{courses.length} course{courses.length === 1 ? "" : "s"}</p>
        </div>
        <div class="course-grid">
          {#each courses as course}
            <button class="course-card" onclick={() => selectCourse(course)}>
              <h3>{course.title}</h3>
              {#if course.description}
                <p class="card-desc">{course.description}</p>
              {/if}
              <div class="card-footer">
                <div class="card-progress-bar">
                  <div
                    class="card-progress-fill"
                    style="width: {course.total_lessons > 0
                      ? (course.completed_lessons / course.total_lessons) * 100
                      : 0}%"
                  ></div>
                </div>
                <span class="card-progress-label">
                  {course.completed_lessons}/{course.total_lessons} lessons
                </span>
              </div>
            </button>
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">📚</div>
        <h2>Build your first course</h2>
        <p>Click "Build Course" to generate a new course with Claude</p>
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
    transition: width 0.2s, min-width 0.2s;
  }

  .sidebar.collapsed {
    width: 52px;
    min-width: 52px;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0 1rem;
    height: 52px;
    border-bottom: 1px solid var(--input-border);
  }

  .sidebar-toggle {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 0.7rem;
    cursor: pointer;
    margin-left: auto;
    padding: 0.25rem;
    transition: color 0.15s;
    flex-shrink: 0;
  }

  .sidebar-toggle:hover {
    color: var(--text);
  }

  .sidebar.collapsed .sidebar-header {
    justify-content: center;
    padding: 0;
  }

  .sidebar.collapsed .sidebar-toggle {
    margin-left: 0;
  }

  .sidebar.collapsed .sidebar-logo {
    display: none;
  }

  .sidebar-logo {
    font-size: 1.25rem;
  }

  .sidebar-home {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: none;
    border: none;
    color: #fff;
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: -0.01em;
    cursor: pointer;
    padding: 0;
  }

  .sidebar-home:hover {
    color: var(--accent);
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

  /* Dashboard */
  .dashboard {
    padding: 2rem;
  }

  .dashboard-header {
    margin-bottom: 1.5rem;
  }

  .dashboard-header h1 {
    font-size: 1.4rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text);
  }

  .dashboard-subtitle {
    color: var(--text-muted);
    font-size: 0.8rem;
    margin-top: 0.2rem;
  }

  .course-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }

  .course-card {
    background: var(--bg-card);
    border: 1px solid var(--input-border);
    border-radius: 10px;
    padding: 1.25rem;
    text-align: left;
    cursor: pointer;
    transition: border-color 0.15s;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .course-card:hover {
    border-color: var(--accent);
  }

  .course-card h3 {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
  }

  .card-desc {
    color: var(--text-muted);
    font-size: 0.8rem;
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-footer {
    margin-top: auto;
    padding-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .card-progress-bar {
    width: 100%;
    height: 4px;
    background: var(--input-bg);
    border-radius: 2px;
    overflow: hidden;
  }

  .card-progress-fill {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s;
  }

  .card-progress-label {
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  /* Course detail */
  .detail-view {
    padding: 2rem;
    max-width: 720px;
    overflow-y: auto;
  }

  .detail-header {
    margin-bottom: 1.5rem;
  }

  .detail-header h1 {
    font-size: 1.4rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-bottom: 0.4rem;
  }

  .detail-desc {
    color: var(--text-muted);
    font-size: 0.85rem;
    line-height: 1.6;
  }

  .lesson-list {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .lesson-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.7rem 0.85rem;
    background: var(--bg-card);
    border: 1px solid var(--input-border);
    border-radius: 8px;
    color: var(--text);
    font-size: 0.875rem;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s;
    width: 100%;
  }

  .lesson-item:hover {
    border-color: var(--accent);
  }

  .lesson-num {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--input-bg);
    border-radius: 6px;
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .lesson-name {
    flex: 1;
  }

  .lesson-check {
    color: var(--accent);
    font-size: 0.85rem;
    flex-shrink: 0;
  }


  .lesson-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
  }

  .lesson-title {
    font-size: 1.1rem;
    font-weight: 700;
    flex: 1;
  }

  .chat-toggle-btn {
    background: none;
    border: 1px solid var(--input-border);
    border-radius: 6px;
    padding: 0.35rem 0.6rem;
    font-size: 0.9rem;
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    flex-shrink: 0;
  }

  .chat-toggle-btn:hover,
  .chat-toggle-btn.active {
    border-color: var(--accent);
    background: var(--input-bg);
  }

  .lesson-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .lesson-layout > .detail-view {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    max-width: none;
  }

  .lesson-footer {
    margin-top: 2rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--input-border);
  }

  .next-btn {
    padding: 0.7rem 1.5rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .next-btn:hover {
    background: var(--accent-hover);
  }

  .back-btn {
    background: none;
    border: 1px solid var(--input-border);
    border-radius: 6px;
    color: var(--text-muted);
    font-size: 0.8rem;
    padding: 0.35rem 0.7rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
    white-space: nowrap;
  }

  .back-btn:hover {
    border-color: var(--accent);
    color: var(--text);
  }

  .lesson-content {
    line-height: 1.7;
    font-size: 0.9rem;
  }

  .lesson-content :global(h1) {
    font-size: 1.3rem;
    font-weight: 700;
    margin-bottom: 0.75rem;
  }

  .lesson-content :global(h2) {
    font-size: 1.1rem;
    font-weight: 600;
    margin: 1.25rem 0 0.5rem;
  }

  .lesson-content :global(p) {
    margin-bottom: 0.75rem;
  }

  .lesson-content :global(pre) {
    background: var(--input-bg);
    border: 1px solid var(--input-border);
    border-radius: 8px;
    padding: 1rem;
    overflow-x: auto;
    margin-bottom: 0.75rem;
  }

  .lesson-content :global(code) {
    font-family: "SF Mono", "Fira Code", monospace;
    font-size: 0.8rem;
  }

  .lesson-content :global(p code) {
    background: var(--input-bg);
    padding: 0.15rem 0.35rem;
    border-radius: 4px;
  }

  /* Chat panel */
  .chat-panel {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-card);
    border-left: 1px solid var(--input-border);
  }

  .chat-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 0.8rem;
    border-bottom: 1px solid var(--input-border);
    height: 48px;
    box-sizing: border-box;
  }

  .chat-panel-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text);
  }

  .chat-back-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0;
  }

  .chat-back-btn:hover {
    color: var(--accent);
  }

  .chat-panel-actions {
    display: flex;
    gap: 0.3rem;
  }

  .chat-action-btn {
    background: none;
    border: 1px solid var(--input-border);
    border-radius: 4px;
    color: var(--text-muted);
    font-size: 0.8rem;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .chat-action-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
  }

  .chat-panel-messages {
    flex: 1;
    overflow-y: auto;
    padding: 0.8rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .chat-panel-empty {
    color: var(--text-muted);
    font-size: 0.8rem;
    text-align: center;
    padding: 2rem 1rem;
  }

  .chat-panel-messages .chat-msg {
    display: flex;
  }

  .chat-panel-messages .chat-msg.user {
    justify-content: flex-end;
  }

  .chat-panel-messages .msg-bubble {
    max-width: 85%;
    padding: 0.5rem 0.7rem;
    border-radius: 12px;
    font-size: 0.8rem;
    line-height: 1.4;
    background: var(--input-bg);
    color: var(--text);
  }

  .chat-panel-messages .chat-msg.user .msg-bubble {
    background: var(--accent);
    color: #fff;
  }

  .chat-panel-input {
    display: flex;
    gap: 0.4rem;
    padding: 0.6rem 0.8rem;
    border-top: 1px solid var(--input-border);
  }

  .chat-panel-input textarea {
    flex: 1;
    background: var(--input-bg);
    border: 1px solid var(--input-border);
    border-radius: 8px;
    padding: 0.5rem 0.6rem;
    font-size: 0.8rem;
    color: var(--text);
    resize: none;
    font-family: inherit;
    min-height: 36px;
  }

  .chat-panel-input textarea:focus {
    outline: none;
    border-color: var(--accent);
  }

  .chat-panel-input .send-btn {
    padding: 0 0.8rem;
    background: var(--accent);
    color: #fff;
    border: none;
    border-radius: 8px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }

  .chat-panel-input .send-btn:hover {
    background: var(--accent-hover);
  }

  .chat-panel-input .send-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .chat-history {
    flex: 1;
    overflow-y: auto;
    padding: 0.4rem;
  }

  .chat-history-empty {
    color: var(--text-muted);
    font-size: 0.8rem;
    text-align: center;
    padding: 2rem 1rem;
  }

  .chat-history-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.6rem 0.7rem;
    background: none;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    text-align: left;
    color: var(--text);
    font-size: 0.8rem;
    transition: background 0.15s;
  }

  .chat-history-item:hover {
    background: var(--input-bg);
  }

  .chat-history-item.active {
    background: var(--input-bg);
    border-left: 2px solid var(--accent);
  }

  .chat-history-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .chat-history-count {
    color: var(--text-muted);
    font-size: 0.7rem;
    flex-shrink: 0;
    margin-left: 0.5rem;
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
