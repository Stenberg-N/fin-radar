<script lang="ts">
  import { onMount, onDestroy, getContext } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import TextAlign from '@tiptap/extension-text-align';
  import { fly, slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { deleteNote } from "$lib/notes";
  import { user } from "$lib/user";
  import type { Note } from "$lib/types";
  import { handleClickOutside, handleHorizontalScroll } from "$lib/functions";

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
  let isHeadings = $state<boolean>(false);
  let isToolBarButtons = $state<boolean>(false);

  let noteToolBarButtonRefs = $state<HTMLButtonElement[]>([]);
  let toggleHeadingOptions = $state<HTMLButtonElement | null>(null);

  let toggleSettingsButton = $state<HTMLButtonElement | null>(null);
  let isSettingsBanner = $state<boolean>(false);
  const noteSettingsButtons = [
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => async () => { await handleDeleteNote(note.id); isSettingsBanner = false; }}
  ];

  const noteToolBarButtons = [
    { name: "underline", icon: "/underline.svg" },
    { name: "bold", icon: "/bold.svg" },
    { name: "italic", icon: "/italic.svg" },
    { name: "bullet-list", icon: "/bulleted-list.svg" },
    { name: "heading", icon: "/heading.svg" },
    { name: "align-left", icon: "/align-left.svg" },
    { name: "align-center", icon: "/align-center.svg" },
    { name: "align-right", icon: "/align-right.svg" },
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
      }
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
      }
    })
  });

  onDestroy(() => {
    contentEditorState.editor?.destroy();
    titleEditorState.editor?.destroy();
  });

  // Used to collect toolbar's button references and bind the button for showing heading options to toggleHeadingOptions,
  // and pass that to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (noteToolBarButtonRefs[4]) toggleHeadingOptions = noteToolBarButtonRefs[4];
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

  const applyProperty = (command: string) => {
    if (!activeEditor) return;

    switch (command) {
      case 'underline': activeEditor.chain().focus().toggleUnderline().run(); break;
      case 'bold': activeEditor.chain().focus().toggleBold().run(); break;
      case 'italic': activeEditor.chain().focus().toggleItalic().run(); break;
      case 'bullet-list': activeEditor.chain().focus().toggleBulletList().run(); break;
      case 'heading': isHeadings = !isHeadings; break;
      case 'align-left': activeEditor.chain().focus().setTextAlign('left').run(); break;
      case 'align-center': activeEditor.chain().focus().setTextAlign('center').run(); break;
      case 'align-right': activeEditor.chain().focus().setTextAlign('right').run(); break;
    }
  };
</script>

<div class="note-container vertical-flex-container">
  {#if isSettingsBanner}
    <div class="note-settings-banner modal-default vertical-flex-container" transition:fly={{ y: -40, duration: 200, easing: cubicInOut }} use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isSettingsBanner = false, additionalElements: [toggleSettingsButton] }}>
      <div class="note-settings-banner-topbar horizontal-flex-container">
        <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => isSettingsBanner = false}><img src="close-x.svg" alt="Close" class="img-small" /></button>
      </div>
      {#each noteSettingsButtons as button, i (button.titleKey)}
        <button class="primary-button horizontal-flex-container" onclick={button.command()}><img src={button.icon} alt="button-icon-{i}" class="img-small" />{$t[button.titleKey]}</button>
      {/each}
    </div>
  {/if}

  {#if isHeadings}
    <div class="headings-modal modal-default vertical-flex-container" use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isHeadings = false, additionalElements: [toggleHeadingOptions] }} transition:fly={{ y: -40, duration: 200, easing: cubicInOut }}>
      <button class="primary-button" onclick={() => { activeEditor?.chain().focus().setParagraph().run(); isHeadings = false; }}>{$t["notes.heading-unset"]}</button>
      <button class="primary-button" onclick={() => { activeEditor?.chain().focus().setHeading({ level: 2 }).run(); isHeadings = false; }}>{$t["notes.heading-option"] + " " + "1"}</button>
      <button class="primary-button" onclick={() => { activeEditor?.chain().focus().setHeading({ level: 3 }).run(); isHeadings = false; }}>{$t["notes.heading-option"] + " " + "2"}</button>
    </div>
  {/if}

  <div class="note-toolbar horizontal-flex-container">
    <button class="transparent-button-highlight" style="margin-right: 8px;" bind:this={toggleSettingsButton} onclick={() => isSettingsBanner = !isSettingsBanner}><img src="/burger.svg" alt="Burger" class="img-small" /></button>
    <button class="note-toggle-toolbar-buttons transparent-button-highlight" class:toggled={isToolBarButtons} onclick={() => isToolBarButtons = !isToolBarButtons}>{$t["notes.tools.button"]}</button>
    {#if isToolBarButtons}
      <div class="note-toolbar-buttons-container horizontal-flex-container" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }} transition:slide={{ axis: "x", duration: 200, easing: cubicInOut }}>
        {#each noteToolBarButtons as button, i (button.name)}
          <button class="primary-button" title={$t["note-toolbar.button.titles"][i] as string} class:disabled={(i === 3 || i === 4) && activeEditor === titleEditorState.editor} disabled={(i === 3 || i === 4) && activeEditor === titleEditorState.editor}
            bind:this={noteToolBarButtonRefs[i]} onclick={() => i === 4 && !activeEditor ? sendAlert("alert.notes.no-editor-selected", true, false) : applyProperty(button.name)}
          >
            <img src={button.icon} alt={button.icon} class="img-small" style="object-fit: contain;" />
          </button>
        {/each}
      </div>
    {/if}
  </div>
  <div class="note-content vertical-flex-container">
    <div class="note-title-container horizontal-flex-container" bind:this={titleEditorElement}></div>
    <div class="note-content-container vertical-flex-container" bind:this={contentEditorElement}></div>
  </div>
</div>

<style>
  .note-settings-banner {
    z-index: 1;
    top: 8px;
    left: 48px;
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

  .note-toolbar .note-toggle-toolbar-buttons {
    height: 48px;
    min-width: fit-content;
    width: unset;
    padding: 6px 10px;
    border-radius: 4px;
    color: #f6f6f6;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }
  .note-toggle-toolbar-buttons.toggled {
    border-radius: 4px 0 0 4px;
    background-color: #181818;
  }

  .note-toolbar-buttons-container {
    height: 48px;
    justify-content: flex-start;
    align-items: flex-start;
    gap: 6px;
    padding: 8px 8px 4px;
    background-color: #181818;
    border-radius: 0 4px 4px 0;
    overflow-x: auto;
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
    top: 48px;
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