<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";

  import { sendAlert } from "$lib/alert";
  import { deleteNote } from "$lib/notes";
  import { user } from "$lib/user";
  import type { Note } from "$lib/types";

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
  <button class="primary-button" onclick={async () => await handleDeleteNote(note.id)}><img src="/trash-can.svg" alt="Trash" class="img-small" /></button>
  <div class="note-title-container horizontal-flex-container" bind:this={titleEditorElement}></div>
  <div class="note-content-container vertical-flex-container" bind:this={contentEditorElement}></div>
</div>

<style>
  .note-container {
    justify-content: flex-start;
    gap: 10px;
    padding: 24px;
    background-color: #222;
  }

  .note-title-container, .note-content-container {
    width: 100%;
    outline: 1px solid #555;
  }
  .note-content-container {
    height: 100%;
  }
</style>