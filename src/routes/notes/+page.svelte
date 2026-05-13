<script lang="ts">
  import { onMount, tick } from "svelte";
  import { t } from "$lib/i18n";
  import { user } from "$lib/user";
  import { createNote, createTab, getNotes, getTabs, notes, tabs, updateTab, deleteTab } from "$lib/notes";
  import { sendAlert } from "$lib/alert";

  import Note from "../../components/notes/Note.svelte";

  let displayNotes = $derived($notes.filter(n => n.tab_id === currentTabId));
  let displayTabs = $derived($tabs);

  let currentTabId = $state<number>(0);
  let editingTabId = $state<number | null>(null);
  let editingTabTitle = $derived.by(() => { const tab = $tabs.find(t => t.id === editingTabId); return tab ? tab.title : 'Unknown title' });

  const toolBarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: () => addNote() },
    { titleKey: "notes.delete-tab.button", icon: "/trash-can.svg", command: () => sendAlert("alert.delete-tab.confirmation", false, true, () => handleDeleteTab()) },
  ];

  onMount(async () => {
    if (!$user) return;
    await getTabs($user.id, $user.name);
  });

  $effect(() => {
    if (currentTabId !== null && $user) {
      const timer = setTimeout(() => {
        (async () => await getNotes($user.id, $user.name, currentTabId))();
      }, 200);

      return () => clearTimeout(timer);
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleTabEditStart = async (node: EventTarget | null, tabId: number) => {
    editingTabId = tabId;
    await tick();
    const inputEl = ((node as HTMLButtonElement).firstChild as HTMLInputElement);
    inputEl ? inputEl.focus() : {};
  };

  /***********************************************************************************************************************************/

  const addNote = async () => {
    if (!$user || currentTabId <= 0) return;

    const result = await createNote($user.id, $user.name, currentTabId, "Untitled", "No content");
    if (result.success) sendAlert("alert.add-note.success", true, false);
    else sendAlert("alert.add-note.fail", true, false);
  };

  const addTab = async () => {
    if (!$user) return;

    const result = await createTab($user.id, $user.name, "New tab");
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

  const handleDeleteTab = async () => {
    if (!$user || currentTabId <= 0) return;

    const result = await deleteTab($user.id, $user.name, currentTabId);
    if (result.success) sendAlert("alert.delete-tab.success", true, false);
    else sendAlert("alert.delete-tab.fail", true, false);
    currentTabId = 0;
  };
</script>

<div id="notes-main-container" class="vertical-flex-container">
  <div id="notes-toolbar" class="horizontal-flex-container">
    {#each toolBarButtons as button (button.titleKey)}
      <button class="primary-button horizontal-flex-container"
        class:disabled={currentTabId <= 0}
        disabled={currentTabId <= 0}
        style="gap: 8px;"
        onclick={() => currentTabId > 0 ? button.command() : {}}
      >
        <img src={button.icon} alt="Add" class="img-small" />
        {$t[button.titleKey]}
      </button>
    {/each}
  </div>
  {#if currentTabId <= 0}
    <p style="justify-self: center; margin-top: 120px;">{$t["notes.no-current-tabid"]}</p>
  {/if}
  <div id="notes-container">
    {#each displayNotes as note (note.id)}
      <Note title={note.title} content={note.content} />
    {/each}
  </div>
  <div id="notes-tabbar" class="horizontal-flex-container">
    <button id="notes-tab-add-button" class="primary-button" onclick={() => addTab()}>{$t["notes.add-tab.button"]}</button>
    <div id="notes-tabs-list" class="horizontal-flex-container">
      {#each displayTabs as tab (tab.id)}
        <button class="transparent-button-highlight" class:selected={tab.id === currentTabId}
          onclick={() => currentTabId = tab.id}
          ondblclick={(e) => handleTabEditStart(e.target, tab.id)}
          onkeydown={(e) => { if (e.key === "Enter") saveTabEdit(); if (e.key === "Escape") exitTabEdit(); }}
        >
          {#if editingTabId === tab.id}
            <input class="transparent-input" type="text" bind:value={editingTabTitle} onblur={() => saveTabEdit()} />
          {:else}
            {tab.title}
          {/if}
        </button>
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
    height: 32px;
    padding: 2px 4px 2px 0;
    gap: 20px;
    border-top: 1px solid #333;
  }

  #notes-tabbar button {
    justify-content: flex-start;
    padding: 6px 8px;
    transform: none;
    box-shadow: none;
    outline: none;
  }

  #notes-tabbar #notes-tab-add-button {
    border-radius: 0 6px 6px 0;
  }

  #notes-tabbar button input {
    padding: 0;
  }

  #notes-tabs-list {
    justify-content: flex-start;
  }

  #notes-tabs-list button {
    width: 6rem;
    text-align: left;
    border-radius: 0;
    border-right: 1px solid #333;
    color: #f6f6f6;
  }
  #notes-tabs-list button:first-child {
    border-left: 1px solid #333;
  }
  #notes-tabs-list button:hover {
    background-color: #222;
  }
  #notes-tabs-list button.selected {
    height: 27px;
    padding: 0 8px;
    background-color: #222;
  }
</style>