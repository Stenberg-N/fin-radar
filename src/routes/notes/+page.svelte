<script lang="ts">
  import { onMount, getContext } from "svelte";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { goto, beforeNavigate } from "$app/navigation";
  import { load, Store } from '@tauri-apps/plugin-store';

  import { lang, t } from "$lib/i18n";
  import { user } from "$lib/user";
  import { createNote, createTab, getNotes, getTabs, notes, tabs, updateTab, deleteTab, updateTabColor, updateNote } from "$lib/notes";
  import { sendAlert } from "$lib/alert";
  import { setViewState, viewStore } from "$lib/viewStore";
  import { handleClickOutside, handleHorizontalScroll } from "$lib/functions";
  import type { Note } from "$lib/types";

  import NoteComponent from "../../components/notes/Note.svelte";
  import ContextMenu from "../../components/notes/ContextMenu.svelte";

  let displayNotes = $derived($notes.filter(n => n.tab_id === currentTabId));
  let displayTabs = $derived($tabs);
  let noteUpdateBatch = $state<Note[]>([]);

  let store: Store;
  let noteColumns = $state<number | null>(null);

  let isDeleteModalVisible = $state<boolean>(false);
  let isColorOptions = $state<boolean>(false);
  let cursorTimer: number;
  let cursorX = $state(0);
  let cursorY = $state(0);
  let pendingNavigation = $state<string | null>(null);

  let currentTabId = $state<number | null>(null);
  let editingTabId = $state<number | null>(null);
  let editingTabTitle = $derived.by(() => { const tab = $tabs.find(t => t.id === editingTabId); return tab ? tab.title : 'Unknown title' });
  let editingTabInput = $state<HTMLInputElement | null>(null);

  let contextMenuTabId = $state<number | null>(null);
  const isContextMenu = $derived($viewStore.isContextMenu);

  const toolBarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: async () => await addNote() },
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => {
      isDeleteModalVisible = true;
      sendAlert("alert.delete-tab.confirmation", false, true, async () => { if (currentTabId !== null) { await handleTabDelete(currentTabId); currentTabId = null; } else {} }, () => isDeleteModalVisible = false);
    }},
    { titleKey: "notes.change-tab-color", icon: "/palette.svg", command: () => isColorOptions = true },
  ];

  const availableColors = [
    { value: "transparent", title: ["No color", "Ei väriä"]},
    { value: "rgba(200, 200, 200, 1)", title: ["White", "Valkoinen"]},
    { value: "rgba(113, 45, 255, 0.2)", title: ["Purple", "Purppura"] },
    { value: "rgba(255, 70, 70, 0.2)", title: ["Red", "Punainen"] },
    { value: "rgba(255, 0, 255, 0.2)", title: ["Pink", "Pinkki"] },
    { value: "rgba(255, 150, 72, 0.2)", title: ["Orange", "Oranssi"] },
    { value: "rgba(255, 220, 0, 0.2)", title: ["Yellow", "Keltainen"] },
    { value: "rgba(94, 255, 94, 0.2)", title: ["Green", "Vihreä"] },
    { value: "rgba(215, 255, 0, 0.2)", title: ["Lime", "Lime"] },
    { value: "rgba(0, 255, 240, 0.2)", title: ["Turquoise", "Turkoosi"] },
    { value: "rgba(0, 140, 255, 0.2)", title: ["Blue", "Sininen"] },
  ];

  onMount(() => {
    (async () => {
      if (!$user) return;
      await getTabs($user.id, $user.name);
      store = await load('note-preferences.json', { defaults: { autoSave: false } });
      noteColumns = await store.get<number | null>('note-columns') ?? 4;
      window.addEventListener('mousemove', updateCursorPos);
      return () => {
        if (cursorTimer) clearTimeout(cursorTimer);
        removeEventListener('mousemove', updateCursorPos);
      };
    })();
  });

  beforeNavigate(({ to, cancel }) => {
    if (!to || noteUpdateBatch.length === 0) return;

    cancel();
    pendingNavigation = to.url.pathname;
    sendAlert("alert.notes.unsaved-changes", true, false);
  });

  $effect(() => {
    if (pendingNavigation !== null && noteUpdateBatch.length === 0) {
      goto(pendingNavigation);
      pendingNavigation = null;
    }
  });

  $effect(() => {
    const _currentTabId = currentTabId;
    if (_currentTabId !== null && $user) {
      const timer = setTimeout(() => {
        (async () => await getNotes($user.id, $user.name, _currentTabId))();
      }, 200);

      return () => clearTimeout(timer);
    }
  });

  $effect(() => {
    const interval = setInterval(async () => {
      if (noteUpdateBatch.length === 0 || !$user) return;

      const batch = noteUpdateBatch.splice(0);
      const result = await updateNote($user.id, $user.name, batch);
      if (!result.success) sendAlert("alert.note-update.fail", true, false);
    }, 2000);

    return () => clearInterval(interval);
  });

  $effect(() => {
    if (noteColumns !== null && store) {
      (async () => await store.set('note-columns', noteColumns))();
      (async () => await store.save())();
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const handleOutsideClick = () => { isColorOptions = false };

  const updateCursorPos = (e: MouseEvent) => {
    if (isContextMenu) return;
    if (cursorTimer) clearTimeout(cursorTimer);

    cursorTimer = setTimeout(() => {
      cursorX = e.clientX;
      cursorY = e.clientY;
    }, 50);
  };

  const handleTabEditStart = async (contextmenu?: boolean) => {
    if (contextmenu) {
      editingTabId = contextMenuTabId;
      setViewState("isContextMenu", false);
    }
    else editingTabId = currentTabId;
  };

  const handleContextMenu = (e: MouseEvent, tabId: number) => {
    contextMenuTabId = tabId;
    setViewState("isContextMenu", true);
    cursorX = e.clientX;
    cursorY = e.clientY;
  };

  const handleContextMenuDelete = async () => {
    isDeleteModalVisible = true;
    setViewState("isContextMenu", false);
    sendAlert("alert.delete-context-tab.confirmation", false, true,
      async () => { if (contextMenuTabId !== null) { await handleTabDelete(contextMenuTabId); } else {} },
      () => { isDeleteModalVisible = false; contextMenuTabId = null; },
      $tabs.find(t => t.id === contextMenuTabId)?.title
    );
  };

  const handleContextMenuTabColor = async (color: string) => {
    if (!$user || !$tabs.some(t => t.id === contextMenuTabId) || contextMenuTabId === null) return;
    const result = await updateTabColor($user.id, $user.name, contextMenuTabId, color);
    if (!result.success) sendAlert("alert.tab-color-update.fail", true, false);
  };

  /***********************************************************************************************************************************/

  const addNote = async () => {
    if (!$user || currentTabId === null) return;

    const result = await createNote($user.id, $user.name, currentTabId, ($lang === 'en' ? "Title" : "Otsikko"), ($lang === 'en' ? "No content" : "Ei sisältöä"));
    if (!result.success) sendAlert("alert.add-note.fail", true, false);
  };

  const handleNoteUpdate = (updatedNote: Note) => {
    const idx = noteUpdateBatch.findIndex(n => n.id === updatedNote.id);
    if (idx !== -1) noteUpdateBatch[idx] = updatedNote;
    else noteUpdateBatch.push(updatedNote);
  };

  const addTab = async () => {
    if (!$user) return;

    const result = await createTab($user.id, $user.name, ($lang === 'en' ? "New tab" : "Uusi välilehti"));
    if (!result.success) sendAlert("alert.add-tab.fail", true, false);
  };

  const saveTabEdit = async () => {
    if (!$user || !editingTabId) return;
    if (editingTabTitle.trim() === '') {
      sendAlert("alert.tab.no-title", true, false);
      return;
    }

    const result = await updateTab($user.id, $user.name, editingTabId, editingTabTitle);
    if (!result.success) sendAlert("alert.update-tab.fail", true, false);
    editingTabId = null;
  };

  const exitTabEdit = () => {
    editingTabId = null;
  };

  const handleTabDelete = async (tabId: number | null) => {
    if (!$user || !$tabs.some(t => t.id === tabId) || tabId === null) return;

    const result = await deleteTab($user.id, $user.name, tabId);
    if (result.success) sendAlert("alert.delete-tab.success", true, false);
    else sendAlert("alert.delete-tab.fail", true, false);
    if (contextMenuTabId === currentTabId) currentTabId = null;
    isDeleteModalVisible = false;
    contextMenuTabId = null;
    setViewState("isContextMenu", false);
  };

  const handleUpdateTabColor = async (color: string) => {
    if (!$user || !$tabs.some(t => t.id === currentTabId) || currentTabId === null) return;
    const result = await updateTabColor($user.id, $user.name, currentTabId, color);
    if (!result.success) sendAlert("alert.tab-color-update.fail", true, false);
    isColorOptions = false;
  };
</script>

{#if isContextMenu}
  <ContextMenu {handleContextMenuDelete} {cursorX} {cursorY} {availableColors} {handleContextMenuTabColor} {handleTabEditStart} />
{/if}

{#if isColorOptions}
  <div class="horizontal-flex-container notes-tab-color-menu" use:handleClickOutside={{ getIgnoredElements, onOutsideClick: handleOutsideClick }} transition:slide={{ axis:"y", duration: 200, easing: cubicInOut }}>
    {#each availableColors as color (color.value)}
      <button class="transparent-button" title={$lang === 'en' ? color.title[0] : color.title[1]} style="background-color: {color.value}; border-radius: 50%;"
        onclick={() => handleUpdateTabColor(color.value)}
      ></button>
    {/each}
  </div>
{/if}

<div id="notes-main-container" class="vertical-flex-container">
  <div id="notes-toolbar" class="horizontal-flex-container">
    {#each toolBarButtons as button (button.titleKey)}
      <button class="primary-button horizontal-flex-container"
        class:disabled={currentTabId === null}
        disabled={currentTabId === null}
        style="gap: 8px;"
        onclick={() => currentTabId !== null ? button.command() : {}}
      >
        <img src={button.icon} alt="Add" class="img-small" />
        {$t[button.titleKey]}
      </button>
    {/each}
    <select id="notes-columns-select" class="primary-input" bind:value={noteColumns}>
      {#each Array.from({ length: 5}, (_, i) => i+1) as index}
        <option style="background-color: #0f0f0f;" value={index}>{index}</option>
      {/each}
    </select>
  </div>
  {#if currentTabId === null}
    <p style="justify-self: center;">{$t["notes.no-current-tabid"]}</p>
  {:else}
    {#if displayNotes.length <= 0}
      <div class="vertical-flex-container">
        <p>{$t["notes.no-notes-yet"]}</p>
        <img src="/notes.svg" alt="Notes" style="width: 6rem; height: 8rem;" />
      </div>
    {:else}
      <div id="notes-container" style="grid-template-columns: repeat({noteColumns}, 1fr);">
        {#each displayNotes as note (note.id)}
          <NoteComponent {note} onUpdate={handleNoteUpdate} />
        {/each}
      </div>
    {/if}
  {/if}
  <div id="notes-tabbar" class="horizontal-flex-container">
    <button id="notes-tab-add-button" class="primary-button horizontal-flex-container" onclick={() => addTab()}><img src="/plus.svg" alt="Plus" class="img-small" />{$t["notes.add-tab.button"]}</button>
    <div id="notes-tabs-list" class="horizontal-flex-container" use:handleHorizontalScroll>
      {#each displayTabs as tab (tab.id)}
        <div class="notes-tab-outer-container">
          <button class="transparent-button-highlight" style="background-color: {tab.color}; color: {tab.color === availableColors[1].value ? 'black' : '#f6f6f6'}"
            onclick={() => currentTabId = tab.id}
            oncontextmenu={(e) => { e.preventDefault(); handleContextMenu(e, tab.id); }}
            ondblclick={() => handleTabEditStart()}
            onkeydown={(e) => { if (e.key === "Enter") saveTabEdit(); if (e.key === "Escape") exitTabEdit(); }}
            class:in-editmode={tab.id === editingTabId}
            class:disabled={isDeleteModalVisible}
            disabled={isDeleteModalVisible}
            title={tab.title}
          >
            {#if editingTabId === tab.id}
              <input class="transparent-input" type="text" bind:value={editingTabTitle} bind:this={editingTabInput} onblur={() => saveTabEdit()} onclick={(e) => e.stopPropagation()} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: saveTabEdit }} />
              {#each [editingTabInput], i (i)}
                {onMount(() => editingTabInput?.focus())}
              {/each}
            {:else}
              <span class:slideText={tab.title.length >= 18}>{tab.title}</span>
            {/if}
          </button>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  #notes-main-container {
    justify-content: space-between;
    height: 100%;
    width: 100%;
  }

  #notes-toolbar {
    justify-content: flex-start;
    width: 100%;
    height: 48px;
    gap: 12px;
    padding: 8px;
    border-bottom: 1px solid #333;
  }

  #notes-columns-select {
    max-width: 50px;
    max-height: 28px;
    color: #f6f6f6;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }
  #notes-columns-select:hover {
    cursor: pointer;
  }

  #notes-container {
    display: grid;
    grid-auto-rows: 400px;
    gap: 20px;
    padding: 20px 14px 20px 20px;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  #notes-tabbar {
    justify-content: flex-start;
    width: 100%;
    min-height: 32px;
    height: 32px;
    padding: 2px 4px 2px 0;
    gap: 20px;
    border-top: 1px solid #333;
    overflow: hidden;
  }

  #notes-tabbar button {
    justify-content: flex-start;
    gap: 4px;
    padding: 6px 8px;
    transform: none;
    box-shadow: none;
    outline: none;
  }

  #notes-tabbar #notes-tab-add-button {
    height: 100%;
    min-width: 130px;
    width: 130px;
    gap: 8px;
    border-radius: 0 6px 6px 0;
  }

  #notes-tabbar button input {
    padding: 0;
  }

  #notes-tabs-list {
    height: 100%;
    justify-content: flex-start;
    gap: 4px;
    overflow-x: auto;
    overflow-y: hidden;
  }

  .notes-tab-outer-container {
    height: 100%;
    border-right: 1px solid #333;
    padding-right: 4px;
  }
  .notes-tab-outer-container:first-of-type {
    border-left: 1px solid #333;
    padding-left: 4px;
  }

  #notes-tabs-list button {
    position: relative;
    width: 6rem;
    height: 100%;
    padding: 0 8px;
    border-radius: 4px;
    overflow: hidden;
  }
  #notes-tabs-list button:not(.disabled):hover::before {
    position: absolute;
    content: "";
    inset: 0;
    z-index: -1;
    border-radius: 4px;
  }
  #notes-tabs-list button:not(.disabled):hover::before {
    background-color: #222 !important;
  }

  #notes-tabs-list button span {
    width: calc(6rem - 18px);
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  #notes-tabs-list button span.slideText:hover {
    text-overflow: unset;
    overflow: visible;
    animation: slideLeft 3s linear infinite;
  }

  .notes-tab-color-menu {
    top: 50px;
    left: 144px;
    background-color: #181818;
  }
</style>