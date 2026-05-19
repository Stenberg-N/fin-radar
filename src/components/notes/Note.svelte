<script lang="ts">
  import { onMount, onDestroy, getContext } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { deleteNote } from "$lib/notes";
  import { user } from "$lib/user";
  import type { Note } from "$lib/types";
  import { handleClickOutside } from "$lib/functions";

  let {
    note,
    onUpdate,
  }: {
    note: Note;
    onUpdate: (note: Note) => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let title = $state(note.title);
  // svelte-ignore state_referenced_locally
  let content = $state(note.content);
  let debounceTimer: number;
  let toggleSettingsButton = $state<HTMLButtonElement | null>(null);
  let isSettingsBanner = $state<boolean>(false);
  const noteSettingsButtons = [
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => async () => { await handleDeleteNote(note.id); isSettingsBanner = false; }}
  ];

  let contentEditorState = $state<{ editor: Editor | null }>({ editor: null });
  let contentEditorElement = $state<HTMLElement | null>(null);
  let titleEditorState = $state<{ editor: Editor | null }>({ editor: null });
  let titleEditorElement = $state<HTMLElement | null>(null);

  onMount(() => {
    contentEditorState.editor = new Editor({
      element: contentEditorElement,
      extensions: [
        StarterKit,
      ],
      content: content,
      onTransaction: ({ editor }) => {
        contentEditorState = { editor };
      },
      onUpdate: ({ editor }) => {
        content = editor.getHTML();
        scheduleUpdate();
      },
    }),
    titleEditorState.editor = new Editor({
      element: titleEditorElement,
      extensions: [
        StarterKit,
      ],
      content: title,
      onTransaction: ({ editor }) => {
        titleEditorState = { editor };
      },
      onUpdate: ({ editor }) => {
        title = editor.getHTML();
        scheduleUpdate();
      },
    })
  });

  onDestroy(() => {
    contentEditorState.editor?.destroy();
    titleEditorState.editor?.destroy();
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');

  const deleteNoteConfirmation = async (noteId: number) => {
    if (!$user) return;
    const result = await deleteNote($user.id, $user.name, noteId);
    if (result.success) sendAlert("alert.delete-note.success", true, false);
    else sendAlert("alert.delete-note.fail", true, false);
  };

  const scheduleUpdate = () => {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      onUpdate({ ...note, title, content });
    }, 400);
  };

  const stripHtml = (text: string) => {
    const doc = new DOMParser().parseFromString(text, "text/html");
    return doc.body.textContent || "";
  };

  /***********************************************************************************************************************************/

  const handleDeleteNote = async (noteId: number) => {
    sendAlert("alert.delete-note.confirmation", false, true, async () => await deleteNoteConfirmation(noteId), undefined, stripHtml(note.title));
  };
</script>

<div class="note-container vertical-flex-container">
  {#if isSettingsBanner}
    <div class="notes-settings-banner vertical-flex-container" transition:fly={{ y: -40, duration: 200, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isSettingsBanner = false, additionalElements: [toggleSettingsButton] }}>
      <div class="notes-settings-banner-topbar horizontal-flex-container">
        <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => isSettingsBanner = false}><img src="close-x.svg" alt="Close" class="img-small" /></button>
      </div>
      {#each noteSettingsButtons as button, i (button.titleKey)}
        <button class="primary-button horizontal-flex-container" onclick={button.command()}><img src={button.icon} alt="button-icon-{i}" class="img-small" />{$t[button.titleKey]}</button>
      {/each}
    </div>
  {/if}

  <div class="note-toolbar horizontal-flex-container">
    <button class="transparent-button-highlight" bind:this={toggleSettingsButton} onclick={() => isSettingsBanner = !isSettingsBanner}><img src="/burger.svg" alt="Burger" /></button>
  </div>
  <div class="note-content vertical-flex-container">
    <div class="note-title-container horizontal-flex-container" bind:this={titleEditorElement}></div>
    <div class="note-content-container vertical-flex-container" bind:this={contentEditorElement}></div>
  </div>
</div>

<style>
  .notes-settings-banner {
    position: absolute;
    z-index: 1;
    top: 8px;
    left: 48px;
    justify-content: flex-start;
    align-items: flex-start;
    max-width: calc(100% - 56px);
    width: 240px;
    gap: 12px;
    padding: 12px;
    background-color: #181818;
    border-radius: 8px;
    border: 1px solid #333;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }

  .notes-settings-banner-topbar {
    justify-content: space-between;
    width: 100%;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid #333;
  }

  .notes-settings-banner .primary-button {
    justify-content: flex-start;
    width: 100%;
    gap: 8px;
    background-color: transparent;
    box-shadow: none;
  }
  .notes-settings-banner .primary-button:hover {
    background-color: #333;
  }

  .note-container {
    position: relative;
    justify-content: flex-start;
    gap: 6px;
    padding: 4px 8px 24px;
    border-radius: 8px;
    background-color: #222;
  }

  .note-toolbar {
    justify-content: flex-start;
    width: 100%;
    padding: 4px 0;
  }

  .note-toolbar button {
    width: 32px;
    height: 32px;
  }

  .note-toolbar button img {
    width: 20px;
    height: 20px;
  }

  .note-content {
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .note-title-container, .note-content-container {
    width: 100%;
    padding: 2px 10px 2px 16px;
    word-break: break-all;
    overflow-y: auto;
    scrollbar-gutter: stable;
  }
  .note-title-container {
    min-height: fit-content;
  }
  .note-content-container {
    height: 100%;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 3%, rgb(0, 0, 0) 97%, rgba(0, 0, 0, 0));
  }
</style>