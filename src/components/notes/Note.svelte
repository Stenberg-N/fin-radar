<script lang="ts">
  import { onMount, onDestroy, untrack } from "svelte";
  import { Editor, Extension } from "@tiptap/core";
  import { Plugin } from "@tiptap/pm/state";
  import { Decoration, DecorationSet } from "@tiptap/pm/view";
  import StarterKit from "@tiptap/starter-kit";
  import TextAlign from '@tiptap/extension-text-align';
  import { TaskItem, TaskList, BulletList } from '@tiptap/extension-list'
  import { TextStyleKit } from '@tiptap/extension-text-style'
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t } from "$lib/i18n/i18n";
  import { sendAlert } from "$lib/alert";
  import { deleteNote, isNoteUpdateBatchOngoing, queueNoteUpdate } from "$lib/notes";
  import type { Note } from "$lib/types";
  import { handleClickOutside } from "$lib/actions";
  import { viewport } from "$lib/viewport";
  import ModalWrapper from "../ModalWrapper.svelte";

  let {
    note,
    fontSize,
    noteColor,
    toggleHeadingOptions,
    zoomedNote,
    isNoteUpdating,
    noteBgColor,
    setDeleteModalVisibility,
    onFocusChange,
    setZoomedNote,
  }: {
    note: Note;
    fontSize: string;
    noteColor: string | null;
    toggleHeadingOptions: HTMLButtonElement | null;
    zoomedNote: Note | undefined;
    isNoteUpdating: boolean;
    noteBgColor: "dark" | "light";
    setDeleteModalVisibility: (state: boolean) => void;
    onFocusChange?: (controls: {
      applyProperty: (command: string) => void;
      isTitleActive: boolean;
      focusedEditor: Editor;
    } | null) => void;
    setZoomedNote: (noteId: number | null) => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let title = $state(note.title);
  // svelte-ignore state_referenced_locally
  let content = $state(note.content);
  let debounceTimer: ReturnType<typeof setTimeout>;
  let isHeadings = $state<boolean>(false);
  let cursorPosX = $state<number>(0);
  let cursorPosY = $state<number>(0);

  let toggleSettingsButton = $state<HTMLButtonElement | null>(null);
  let isSettingsBanner = $state<boolean>(false);
  const noteSettingsButtons = [
    { titleKey: () => "delete.button",
      icon: () => "/trash-can.svg",
      command: async () => { await deleteNoteConfirmation(note.id); isSettingsBanner = false; }
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
        StarterKit.configure({
          bulletList: false,
        }),
        TaskList.configure({
          itemTypeName: 'taskItem'
        }),
        TaskItem.configure({
          nested: true,
        }),
        BulletList.configure({
          itemTypeName: 'listItem'
        }),
        TextStyleKit,
        TextAlign.configure({
          types: ['heading', 'paragraph'],
          alignments: ['left', 'center', 'right'],
          defaultAlignment: 'left',
        }),
        listMarkerSize,
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
      onBlur: () => {
        contentFocused = false;
        handleBlur;
      },
    }),
    titleEditorState.editor = new Editor({
      element: titleEditorElement,
      extensions: [
        StarterKit,
        TaskList.configure({
          itemTypeName: 'taskItem'
        }),
        TaskItem.configure({
          nested: true,
        }),
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
      onBlur: () => {
        titleFocused = false;
        handleBlur;
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
  const handleDeleteNote = async (noteId: number) => {
    const result = await deleteNote(noteId);
    if (result.success) sendAlert({ message: "alert.delete-note.success", isTimer: true, buttons: false });
    else sendAlert({ message: "alert.delete-note.fail", isTimer: true, buttons: false });
    setDeleteModalVisibility(false);
  };

  const scheduleUpdate = () => {
    isNoteUpdateBatchOngoing.update(() => true);
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      queueNoteUpdate({ ...note, title, content });
    }, 400);
  };

  const stripHtml = (text: string) => {
    const doc = new DOMParser().parseFromString(text, "text/html");
    return doc.body.textContent || "";
  };

  const getCursorPosOnClick = () => {
    if (zoomedNote) {
      cursorPosX = 160;
      cursorPosY = 60;
    } else {
      cursorPosX = $viewport.cursorX;
      cursorPosY = $viewport.cursorY;
    }
  };

  const notifyParent = (focusedEditor: Editor) => {
    onFocusChange?.({
      applyProperty,
      isTitleActive: focusedEditor === titleEditorState.editor,
      focusedEditor,
    });
  };

  const handleBlur = () => {
    queueMicrotask(() => {
      if (!titleFocused && !contentFocused) onFocusChange?.(null);
    });
  };

  const listMarkerSize = Extension.create({
    name: "listMarkerSize",

    addProseMirrorPlugins() {
      return [
        new Plugin({
          props: {
            decorations(state) {
              const decorations: Decoration[] = []
              state.doc.descendants((node, pos) => {
                if (node.type.name !== 'listItem') return

                const firstText = node.firstChild?.firstChild
                const mark = firstText?.marks.find((m) => m.type.name === 'textStyle' && m.attrs.fontSize)

                decorations.push(Decoration.node(pos, pos + node.nodeSize, { style: `font-size: ${mark?.attrs.fontSize ?? '1rem'};`}))
              })

              return DecorationSet.create(state.doc, decorations)
            }
          }
        })
      ]
    }
  });

  /***********************************************************************************************************************************/

  const deleteNoteConfirmation = async (noteId: number) => {
    sendAlert({
      message: "alert.delete-note.confirmation",
      isTimer: false,
      buttons: true,
      onConfirm: async () => await handleDeleteNote(noteId),
      onCancel: () => setDeleteModalVisibility(false),
      additionalText: [stripHtml(note.title)],
    });
    setDeleteModalVisibility(true);
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
      case 'toggle-tasklist': activeEditor.chain().focus().toggleTaskList().run(); break;
      case 'split-listitem': activeEditor.chain().focus().splitListItem(activeEditor.isActive('taskItem') ? 'taskItem' : activeEditor.isActive('listItem') ? 'listItem' : '').run(); break;
      case 'sink-listitem': activeEditor.chain().focus().sinkListItem(activeEditor.isActive('taskItem') ? 'taskItem' : activeEditor.isActive('listItem') ? 'listItem' : '').run(); break;
      case 'lift-listitem': activeEditor.chain().focus().liftListItem(activeEditor.isActive('taskItem') ? 'taskItem' : activeEditor.isActive('listItem') ? 'listItem' : '').run(); break;
    }
  };
</script>

{#if isHeadings}
  <div class="headings-modal modal-default flex column" style="top: {cursorPosY}px; left: {cursorPosX}px;" 
    use:handleClickOutside={{ onOutsideClick: () => isHeadings = false, additionalElements: [toggleHeadingOptions] }} 
    transition:fade={{ duration: 200, easing: cubicInOut }}
  >
    <button class="button-primary" onclick={() => { activeEditor?.chain().focus().setParagraph().run(); isHeadings = false; }}>{$t["notes.heading-unset"]}</button>
    <button class="button-primary" onclick={() => { activeEditor?.chain().focus().setHeading({ level: 2 }).run(); isHeadings = false; }}>{$t["notes.heading-option"] + " " + "1"}</button>
    <button class="button-primary" onclick={() => { activeEditor?.chain().focus().setHeading({ level: 3 }).run(); isHeadings = false; }}>{$t["notes.heading-option"] + " " + "2"}</button>
  </div>
{/if}

{#if isSettingsBanner}
  <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
    <div class="note-settings-banner modal-default flex column"
      use:handleClickOutside={{ onOutsideClick: () => isSettingsBanner = false, additionalElements: [toggleSettingsButton] }}
    >
      <div class="note-settings-banner-topbar flex row">
        <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
        <button aria-label="Close settings" class="button-primary transparent highlight static" onclick={() => isSettingsBanner = false}>
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
        </button>
      </div>
      {#each noteSettingsButtons as button, i (button.titleKey)}
        <button class="button-primary" disabled={i === 1 && isNoteUpdating} onclick={() => button.command()}>
          <span class="span-icon img-small" style="mask-image: url('{button.icon()}');"></span>
          {$t[button.titleKey()]}
        </button>
      {/each}
    </div>
  </ModalWrapper>
{/if}

<div class="note-topbar flex row">
  <button aria-label="Open note settings" class="button-primary transparent highlight static" class:light-theme={noteBgColor === "light"} style="margin-right: 0.5rem;" bind:this={toggleSettingsButton}
    onclick={() => { isSettingsBanner = !isSettingsBanner; getCursorPosOnClick(); }}
  >
    <span class="span-icon img-small" style="mask-image: url('/burger.svg'); background-color: {noteBgColor === 'dark' ? 'var(--color-white-primary1)' : 'black'};"></span>
  </button>
  <div class="note-title-container flex row" bind:this={titleEditorElement}></div>
</div>
<div class="note-content flex column">
  <div class="note-content-container flex column" bind:this={contentEditorElement}></div>
</div>

<style>
  .note-settings-banner {
    justify-content: flex-start;
    max-width: calc(100% - 56px);
    min-width: 200px;
    background-color: var(--color-secondary2);
  }

  .note-settings-banner-topbar {
    justify-content: space-between;
    width: 100%;
    gap: 0.75rem;
    padding-bottom: 0.75rem;
    border-bottom: 2px solid var(--outline-color2);
  }

  .note-topbar {
    justify-content: flex-start;
    width: 100%;
    padding-bottom: 0.75rem;
    border-bottom: 2px solid var(--outline-color1);
  }

  .note-topbar button.light-theme:hover {
    background-color: rgba(0, 0, 0, 0.2);
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
    padding: 2px 30px 2px 6px;
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
    position: fixed;
    z-index: 1000;
  }

  .headings-modal .button-primary {
    width: 100%;
    background-color: transparent;
    text-align: left;
    box-shadow: none;
  }
  .headings-modal .button-primary:hover {
    background-color: var(--color-secondary3);
  }
</style>