<script lang="ts">
  import { onMount, tick, getContext } from "svelte";

  import { lang, t } from "$lib/i18n";
  import { user } from "$lib/user";
  import { createNote, createTab, getNotes, getTabs, notes, tabs, updateTab, deleteTab, updateTabColor } from "$lib/notes";
  import { sendAlert } from "$lib/alert";
  import { setViewState, viewStore } from "$lib/viewStore";
  import { handleClickOutside } from "$lib/functions";

  import Note from "../../components/notes/Note.svelte";
  import ContextMenu from "../../components/notes/ContextMenu.svelte";

  let displayNotes = $derived($notes.filter(n => n.tab_id === currentTabId));
  let displayTabs = $derived($tabs);
  let isDeleteModalVisible = $state<boolean>(false);
  let isColorOptions = $state<boolean>(false);

  let currentTabId = $state<number | null>(null);
  let editingTabId = $state<number | null>(null);
  let editingTabTitle = $derived.by(() => { const tab = $tabs.find(t => t.id === editingTabId); return tab ? tab.title : 'Unknown title' });

  let contextMenuTabId = $state<number | null>(null);
  let isContextMenu = $derived($viewStore.isContextMenu);
  let cursorTimer: number;
  let cursorX = $state(0);
  let cursorY = $state(0);

  const toolBarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: async () => await addNote() },
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => {
      isDeleteModalVisible = true;
      sendAlert("alert.delete-tab.confirmation", false, true, async () => { if (currentTabId !== null) { await handleDeleteTab(currentTabId); currentTabId = null; } else {} }, () => isDeleteModalVisible = false);
    }},
    { titleKey: "notes.change-tab-color", icon: "/palette.svg", command: () => isColorOptions = true },
  ];

  const availableColors = [
    { value: "rgba(113, 45, 255, 0.16)" },
    { value: "rgba(255, 70, 70, 0.16)" },
    { value: "rgba(94, 255, 94, 0.16)" },
  ];

  onMount(() => {
    if (!$user) return;
    (async () => await getTabs($user.id, $user.name))();
    window.addEventListener('mousemove', updateCursorPos);
    return () => {
      if (cursorTimer) clearTimeout(cursorTimer);
      removeEventListener('mousemove', updateCursorPos);
    };
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
    if (cursorX !== null || cursorY !== null) console.log({cursorX, cursorY});
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

  const handleTabEditStart = async (node: EventTarget | null, tabId: number) => {
    editingTabId = tabId;
    await tick();
    const inputEl = ((node as HTMLButtonElement).firstChild as HTMLInputElement);
    inputEl ? inputEl.focus() : {};
  };

  const handleContextMenu = (e: MouseEvent, tabId: number) => {
    contextMenuTabId = tabId;
    setViewState("isContextMenu", true);
    cursorX = e.clientX;
    cursorY = e.clientY;
  };

  const handleContextMenuDelete = async () => {
    isDeleteModalVisible = true;
    sendAlert("alert.delete-tab.confirmation", false, true, async () => { if (contextMenuTabId !== null) { await handleDeleteTab(contextMenuTabId); contextMenuTabId = null; } else {} }, () => isDeleteModalVisible = false);
  };

  const handleContextMenuTabColor = async (color: string) => {
    if (!$user || !$tabs.some(t => t.id === contextMenuTabId) || contextMenuTabId === null) return;
    const result = await updateTabColor($user.id, $user.name, contextMenuTabId, color);
    if (!result.success) sendAlert("alert.tab-color-update.fail", true, false);
  };

  /***********************************************************************************************************************************/

  const addNote = async () => {
    if (!$user || currentTabId === null) return;

    const result = await createNote($user.id, $user.name, currentTabId, "Untitled", "No content");
    if (result.success) sendAlert("alert.add-note.success", true, false);
    else sendAlert("alert.add-note.fail", true, false);
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

  const handleDeleteTab = async (tabId: number | null) => {
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
  <ContextMenu {handleContextMenuDelete} {cursorX} {cursorY} {availableColors} {handleContextMenuTabColor} />
{/if}

{#if isColorOptions}
  <div id="notes-tab-color-options" class="horizontal-flex-container" use:handleClickOutside={{ getIgnoredElements, onOutsideClick: handleOutsideClick }}>
    {#each availableColors as color (color.value)}
      <button class="transparent-button" title={color.value} style="background-color: {color.value}; border-radius: 50%;"
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
      <div id="notes-container">
        {#each displayNotes as note (note.id)}
          <Note title={note.title} content={note.content} />
        {/each}
      </div>
    {/if}
  {/if}
  <div id="notes-tabbar" class="horizontal-flex-container">
    <button id="notes-tab-add-button" class="primary-button horizontal-flex-container" onclick={() => addTab()}><img src="/plus.svg" alt="Plus" class="img-small" />{$t["notes.add-tab.button"]}</button>
    <div id="notes-tabs-list" class="horizontal-flex-container">
      {#each displayTabs as tab (tab.id)}
        <div class="notes-tab-outer-container">
          <button class="transparent-button-highlight" class:selected={tab.id === currentTabId} style="background-color: {tab.color};"
            onclick={() => currentTabId = tab.id}
            oncontextmenu={(e) => { e.preventDefault(); handleContextMenu(e, tab.id); }}
            ondblclick={(e) => handleTabEditStart(e.target, tab.id)}
            onkeydown={(e) => { if (e.key === "Enter") saveTabEdit(); if (e.key === "Escape") exitTabEdit(); }}
            class:disabled={isDeleteModalVisible}
            disabled={isDeleteModalVisible}
          >
            {#if editingTabId === tab.id}
              <input class="transparent-input" type="text" bind:value={editingTabTitle} onblur={() => saveTabEdit()} />
            {:else}
              {tab.title}
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

  #notes-container {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 20px;
    padding: 20px;
    width: 100%;
    height: 100%;
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
    justify-content: flex-start;
    gap: 2px;
    overflow-x: auto;
  }

  .notes-tab-outer-container {
    border-right: 1px solid #333;
    padding-right: 2px;
  }
  .notes-tab-outer-container:first-of-type {
    border-left: 1px solid #333;
    padding-left: 2px;
  }

  #notes-tabs-list button {
    position: relative;
    width: 6rem;
    text-align: left;
    border-radius: 4px;
    color: #f6f6f6;
  }
  #notes-tabs-list button:not(.disabled):hover::before {
    position: absolute;
    content: "";
    inset: 0;
    z-index: -1;
    border-radius: 4px;
    background-color: #222 !important;
  }
  #notes-tabs-list button.selected {
    height: 25px;
    padding: 0 8px;
    outline: 1px solid rgba(255, 70, 70, 1);
  }
  #notes-tabs-list button.selected::before {
    position: absolute;
    content: "";
    inset: 0;
    z-index: -1;
    border-radius: 4px;
    background-color: #222 !important;
  }

  #notes-tab-color-options {
    position: absolute;
    top: 50px;
    left: 158px;
    z-index: 1;
    flex-wrap: wrap;
    gap: 4px;
    padding: 8px 12px;
    border-radius: 6px;
    background-color: #181818;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }

  #notes-tab-color-options button {
    width: 24px;
    height: 24px;
  }
</style>