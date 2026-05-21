<script lang="ts">
  import { onMount, onDestroy, getContext } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import TextAlign from '@tiptap/extension-text-align';
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { deleteNote } from "$lib/notes";
  import { user } from "$lib/user";
  import type { Note } from "$lib/types";
  import { handleClickOutside } from "$lib/functions";

  let {
    note,
    cursorX,
    cursorY,
    toggleHeadingOptions,
    onUpdate,
    onFocusChange,
  }: {
    note: Note;
    cursorX: number;
    cursorY: number;
    toggleHeadingOptions: HTMLButtonElement | null;
    onUpdate: (note: Note) => void;
    onFocusChange?: (controls: {
      applyProperty: (command: string) => void;
      isTitleActive: boolean;
    } | null) => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let title = $state(note.title);
  // svelte-ignore state_referenced_locally
  let content = $state(note.content);
  let debounceTimer: number;
  let isHeadings = $state<boolean>(false);
  let cursorPosX = $state<number>(0);
  let cursorPosY = $state<number>(0);

  let toggleSettingsButton = $state<HTMLButtonElement | null>(null);
  let isSettingsBanner = $state<boolean>(false);
  const noteSettingsButtons = [
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => async () => { await handleDeleteNote(note.id); isSettingsBanner = false; }}
  ];

  let contentEditorState = $state<{ editor: Editor | null }>({ editor: null });
  let contentEditorElement = $state<HTMLElement | null>(null);
  let titleEditorState = $state<{ editor: Editor | null }>({ editor: null });
  let titleEditorElement = $state<HTMLElement | null>(null);
  let activeEditor = $state<Editor | null>(null);

  onMount(() => {
    contentEditorState.editor = new Editor({
      element: contentEditorElement,
      extensions: [
        StarterKit,
        TextAlign.configure({
          types: ['heading', 'paragraph'],
        }),
      ],
      content: content,
      onTransaction: ({ editor }) => {
        contentEditorState = { editor };
      },
      onUpdate: ({ editor }) => {
        content = editor.getHTML();
        scheduleUpdate();
      },
      onFocus: ({ editor }) => {
        activeEditor = editor;
        notifyParent(editor);
      },
    }),
    titleEditorState.editor = new Editor({
      element: titleEditorElement,
      extensions: [
        StarterKit,
        TextAlign.configure({
          types: ['heading', 'paragraph'],
        }),
      ],
      content: title,
      onTransaction: ({ editor }) => {
        titleEditorState = { editor };
      },
      onUpdate: ({ editor }) => {
        title = editor.getHTML();
        scheduleUpdate();
      },
      onFocus: ({ editor }) => {
        activeEditor = editor;
        notifyParent(editor);
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

  const getCursorPosOnClick = () => { cursorPosX = cursorX - 150; cursorPosY = cursorY - 48; };

  const notifyParent = (focusedEditor: Editor) => {
    onFocusChange?.({
      applyProperty,
      isTitleActive: focusedEditor === titleEditorState.editor,
    });
  };

  /***********************************************************************************************************************************/

  const handleDeleteNote = async (noteId: number) => {
    sendAlert("alert.delete-note.confirmation", false, true, async () => await deleteNoteConfirmation(noteId), undefined, stripHtml(note.title));
  };

  const applyProperty = (command: string) => {
    if (!activeEditor) return;

    switch (command) {
      case 'underline': activeEditor.chain().focus().toggleUnderline().run(); break;
      case 'bold': activeEditor.chain().focus().toggleBold().run(); break;
      case 'italic': activeEditor.chain().focus().toggleItalic().run(); break;
      case 'bullet-list': activeEditor.chain().focus().toggleBulletList().run(); break;
      case 'heading': !isHeadings ? getCursorPosOnClick() : {}; isHeadings = !isHeadings; break;
      case 'align-left': activeEditor.chain().focus().setTextAlign('left').run(); break;
      case 'align-center': activeEditor.chain().focus().setTextAlign('center').run(); break;
      case 'align-right': activeEditor.chain().focus().setTextAlign('right').run(); break;
    }
  };
</script>

{#if isHeadings}
  <div class="headings-modal modal-default vertical-flex-container" style="top: {cursorPosY}px; left: {cursorPosX}px;" 
    use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isHeadings = false, additionalElements: [toggleHeadingOptions] }} 
    transition:fade={{ duration: 200, easing: cubicInOut }}
  >
    <button class="primary-button" onclick={() => { activeEditor?.chain().focus().setParagraph().run(); isHeadings = false; }}>{$t["notes.heading-unset"]}</button>
    <button class="primary-button" onclick={() => { activeEditor?.chain().focus().setHeading({ level: 2 }).run(); isHeadings = false; }}>{$t["notes.heading-option"] + " " + "1"}</button>
    <button class="primary-button" onclick={() => { activeEditor?.chain().focus().setHeading({ level: 3 }).run(); isHeadings = false; }}>{$t["notes.heading-option"] + " " + "2"}</button>
  </div>
{/if}

<div class="note-container vertical-flex-container">
  {#if isSettingsBanner}
    <div class="note-settings-banner modal-default vertical-flex-container" transition:fade={{ duration: 200, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isSettingsBanner = false, additionalElements: [toggleSettingsButton] }}>
      <div class="note-settings-banner-topbar horizontal-flex-container">
        <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => isSettingsBanner = false}><img src="close-x.svg" alt="Close" class="img-small" /></button>
      </div>
      {#each noteSettingsButtons as button, i (button.titleKey)}
        <button class="primary-button horizontal-flex-container" onclick={button.command()}><img src={button.icon} alt="button-icon-{i}" class="img-small" />{$t[button.titleKey]}</button>
      {/each}
    </div>
  {/if}

  <div class="note-toolbar horizontal-flex-container">
    <button class="transparent-button-highlight" style="margin-right: 8px;" bind:this={toggleSettingsButton} onclick={() => isSettingsBanner = !isSettingsBanner}><img src="/burger.svg" alt="Burger" class="img-small" /></button>
  </div>
  <div class="note-content vertical-flex-container">
    <div class="note-title-container horizontal-flex-container" bind:this={titleEditorElement}></div>
    <div class="note-content-container vertical-flex-container" bind:this={contentEditorElement}></div>
  </div>
</div>

<style>
  .note-settings-banner {
    z-index: 1;
    top: 44px;
    left: 8px;
    justify-content: flex-start;
    max-width: calc(100% - 56px);
    width: 240px;
  }

  .note-settings-banner-topbar {
    justify-content: space-between;
    width: 100%;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid #333;
  }

  .note-settings-banner .primary-button {
    justify-content: flex-start;
    width: 100%;
    gap: 8px;
    background-color: transparent;
    box-shadow: none;
  }
  .note-settings-banner .primary-button:hover {
    background-color: #333;
  }

  .note-container {
    position: relative;
    justify-content: flex-start;
    gap: 6px;
    padding: 4px 8px 24px;
    border-radius: 8px;
    background-color: #222;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    overflow: hidden;
  }

  .note-toolbar {
    justify-content: flex-start;
    width: 100%;
    padding: 4px 0 8px;
    border-bottom: 2px solid #333;
  }

  .note-toolbar button {
    min-width: 32px;
    width: 32px;
    min-height: 32px;
    height: 32px;
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

  .headings-modal {
    z-index: 1;
  }

  .headings-modal .primary-button {
    width: 100%;
    background-color: transparent;
    text-align: left;
    box-shadow: none;
  }
  .headings-modal .primary-button:hover {
    background-color: #333;
  }
</style>