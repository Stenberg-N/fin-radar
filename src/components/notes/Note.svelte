<script lang="ts">
  import { onMount, onDestroy, getContext, untrack } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import TextAlign from '@tiptap/extension-text-align';
  import { TextStyleKit } from '@tiptap/extension-text-style'
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
    fontSize,
    noteColor,
    toggleHeadingOptions,
    zoomedNote,
    isNoteUpdating,
    noteBgColor,
    onUpdate,
    onFocusChange,
    updateFontSize,
    setZoomedNote,
    updateOngoing,
  }: {
    note: Note;
    cursorX: number;
    cursorY: number;
    fontSize: string;
    noteColor: string | null;
    toggleHeadingOptions: HTMLButtonElement | null;
    zoomedNote: Note | undefined;
    isNoteUpdating: boolean;
    noteBgColor: number | null;
    onUpdate: (note: Note) => void;
    onFocusChange?: (controls: {
      applyProperty: (command: string) => void;
      isTitleActive: boolean;
    } | null) => void;
    updateFontSize: (fontsize: string) => void;
    setZoomedNote: (noteId: number | null) => void;
    updateOngoing: (state: boolean) => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let title = $state(note.title);
  // svelte-ignore state_referenced_locally
  let content = $state(note.content);
  let debounceTimer: number;
  let debounceZoomOut: number;
  let isHeadings = $state<boolean>(false);
  let cursorPosX = $state<number>(0);
  let cursorPosY = $state<number>(0);

  let toggleSettingsButton = $state<HTMLButtonElement | null>(null);
  let isSettingsBanner = $state<boolean>(false);
  const noteSettingsButtons = [
    { titleKey: () => "delete.button",
      icon: () => "/trash-can.svg",
      command: async () => { await handleDeleteNote(note.id); isSettingsBanner = false; }
    },
    { titleKey: () => !zoomedNote ? "zoom.button" : "exit-zoom.button",
      icon: () => !zoomedNote ? "/zoom-in.svg" : "/zoom-out.svg",
      command: () => { !zoomedNote ? setZoomedNote(note.id) : setZoomedNote(null); isSettingsBanner = false; }
    },
  ];

  let contentEditorState = $state<{ editor: Editor | null }>({ editor: null });
  let contentEditorElement = $state<HTMLElement | null>(null);
  let titleEditorState = $state<{ editor: Editor | null }>({ editor: null });
  let titleEditorElement = $state<HTMLElement | null>(null);
  let activeEditor = $state<Editor | null>(null);
  let titleFocused = $state(false);
  let contentFocused = $state(false);

  onMount(() => {
    contentEditorState.editor = new Editor({
      element: contentEditorElement,
      extensions: [
        StarterKit,
        TextStyleKit,
        TextAlign.configure({
          types: ['heading', 'paragraph'],
          alignments: ['left', 'center', 'right'],
          defaultAlignment: 'left',
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
        contentFocused = true;
        activeEditor = editor;
        notifyParent(editor);
      },
      onBlur: () => { contentFocused = false; },
      onSelectionUpdate: ({ editor }) => {
        updateFontSize(editor?.getAttributes('textStyle').fontSize);
      },
    }),
    titleEditorState.editor = new Editor({
      element: titleEditorElement,
      extensions: [
        StarterKit,
        TextStyleKit,
        TextAlign.configure({
          types: ['heading', 'paragraph'],
          alignments: ['left', 'center', 'right'],
          defaultAlignment: 'left',
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
        titleFocused = true;
        activeEditor = editor;
        notifyParent(editor);
      },
      onBlur: () => { titleFocused = false; },
      onSelectionUpdate: ({ editor }) => {
        updateFontSize(editor?.getAttributes('textStyle').fontSize);
      },
    })
  });

  onDestroy(() => {
    contentEditorState.editor?.destroy();
    titleEditorState.editor?.destroy();
  });

  $effect(() => {
    const incomingTitle = note.title;
    const incomingContent = note.content;
    untrack(() => {
      if (!titleFocused && incomingTitle !== title) {
        titleEditorState.editor?.commands.setContent(incomingTitle, { emitUpdate: false });
        title = incomingTitle;
      }
      if (!contentFocused && incomingContent !== content) {
        contentEditorState.editor?.commands.setContent(incomingContent, { emitUpdate: false });
        content = incomingContent;
      }
    });
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
    updateOngoing(true);
    clearTimeout(debounceTimer);
    clearTimeout(debounceZoomOut);
    debounceTimer = setTimeout(() => {
      onUpdate({ ...note, title, content });
    }, 400);
    debounceZoomOut = setTimeout(() => {
      updateOngoing(false);
    }, 2000);
  };

  const stripHtml = (text: string) => {
    const doc = new DOMParser().parseFromString(text, "text/html");
    return doc.body.textContent || "";
  };

  const getCursorPosOnClick = () => {
    if (zoomedNote) {
      cursorPosX = 120;
      cursorPosY = 52;
    } else {
      cursorPosX = cursorX - 150;
      cursorPosY = cursorY - 48;
    }
  };

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
      case 'set-fontsize': activeEditor.chain().focus().setFontSize(fontSize).run(); break;
      case 'bg-color': activeEditor.chain().focus().setBackgroundColor(noteColor ? noteColor : 'transparent').run(); break;
      case 'fore-color': activeEditor.chain().focus().setColor(noteColor ? noteColor : 'white').run(); break;
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

<div class="note-container vertical-flex-container" style="background-color: {noteBgColor === 1 ? '#181818' : 'rgb(200, 200, 200)'}; color: {noteBgColor === 1 ? '#f6f6f6' : 'black'};">
  {#if isSettingsBanner}
    <div class="note-settings-banner modal-default vertical-flex-container" transition:fade={{ duration: 200, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isSettingsBanner = false, additionalElements: [toggleSettingsButton] }}>
      <div class="note-settings-banner-topbar horizontal-flex-container">
        <h2 style="margin: 0; color: #f6f6f6;">{$t["settings-banner.title"]}</h2>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => isSettingsBanner = false}><img src="close-x.svg" alt="Close" class="img-small" /></button>
      </div>
      {#each noteSettingsButtons as button, i (button.titleKey)}
        <button class="primary-button horizontal-flex-container" class:disabled={i === 1 && isNoteUpdating} disabled={i === 1 && isNoteUpdating} onclick={() => button.command()}><img src={button.icon()} alt="button-icon-{i}" class="img-small" />{$t[button.titleKey()]}</button>
      {/each}
    </div>
  {/if}

  <div class="note-topbar horizontal-flex-container">
    <button class="transparent-button-highlight" style="margin-right: 8px;" bind:this={toggleSettingsButton} onclick={() => isSettingsBanner = !isSettingsBanner}>
      <img src="/burger.svg" alt="Burger" class="img-small" style="filter: {noteBgColor === 1 ? 'brightness(0) invert(0.9)' : 'brightness(0) invert(0)'};" />
    </button>
    <div class="note-title-container horizontal-flex-container" bind:this={titleEditorElement}></div>
  </div>
  <div class="note-content vertical-flex-container">
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
  .note-settings-banner .primary-button:not(.disabled):hover {
    background-color: #333;
  }

  .note-container {
    position: relative;
    justify-content: flex-start;
    height: 100%;
    width: 100%;
    gap: 6px;
    padding: 8px 8px 24px;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    overflow: hidden;
  }

  .note-topbar {
    justify-content: flex-start;
    width: 100%;
    padding-bottom: 8px;
    border-bottom: 2px solid #333;
  }

  .note-topbar button {
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
    padding: 2px 6px;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
  }
  .note-title-container {
    min-height: fit-content;
    overflow-y: hidden;
    overflow-x: auto;
    scrollbar-gutter: unset;
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