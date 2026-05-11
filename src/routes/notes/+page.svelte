<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "$lib/i18n";
  import { user } from "$lib/user";
  import { createNote, createTab, getNotes, getTabs, notes, tabs } from "$lib/notes";
  import { sendAlert } from "$lib/alert";

  import Note from "../../components/notes/Note.svelte";

  let displayNotes = $derived($notes.filter(n => n.tab_id === currentTabId));
  let displayTabs = $derived($tabs);
  let currentTabId = $state<number>(0);

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

  const addNote = async () => {
    if (!$user) return;

    const result = await createNote($user.id, $user.name, currentTabId, "Untitled", "No content");
    if (result.success) sendAlert("alert.add-note.success", true, false);
    else sendAlert("alert.add-note.fail", true, false);
  };

  const addTab = async () => {
    if (!$user) return;

    const result = await createTab($user.id, $user.name, "New tab");
    if (result.success) sendAlert("alert.add-tab.success", true, false);
    else sendAlert("alert.add-tab.fail", true, false);
  };
</script>

<div id="notes-main-container" class="vertical-flex-container">
  <div id="notes-toolbar" class="horizontal-flex-container">
    <button class="primary-button horizontal-flex-container" style="gap: 8px;" onclick={() => addNote()}><img src="/plus.svg" alt="Add" class="img-small" />{$t["add.button"]}</button>
  </div>
  <div id="notes-container">
    {#each displayNotes as note (note.id)}
      <Note title={note.title} content={note.content} />
    {/each}
  </div>
  <div id="notes-tabbar" class="horizontal-flex-container">
    <button id="notes-tab-add-button" class="primary-button" onclick={() => addTab()}>{$t["notes.add-tab.button"]}</button>
    <div id="notes-tabs-list" class="horizontal-flex-container">
      {#each displayTabs as tab (tab.id)}
        <button class="primary-button" onclick={() => currentTabId = tab.id}>{tab.title}</button>
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
    padding: 6px 8px;
    transform: none;
    box-shadow: none;
  }

  #notes-tabbar #notes-tab-add-button {
    border-radius: 0 6px 6px 0;
  }

  #notes-tabs-list {
    justify-content: flex-start;
    gap: 8px;
  }
</style>