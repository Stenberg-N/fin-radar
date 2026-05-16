<script lang="ts">
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

  /***********************************************************************************************************************************/

  const handleDeleteNote = async (noteId: number) => {
    sendAlert("alert.delete-note.confirmation", false, true, async () => await deleteNoteConfirmation(noteId), undefined, note.title);
  };
</script>

<div class="note-container vertical-flex-container">
  <div class="note-title-container vertical-flex-container">
    <textarea bind:value={title} oninput={scheduleUpdate}></textarea>
    <button class="primary-button" onclick={async () => await handleDeleteNote(note.id)}><img src="/trash-can.svg" alt="Trash" class="img-small" /></button>
  </div>
  <div class="note-content-container">
    <textarea bind:value={content} oninput={scheduleUpdate}></textarea>
  </div>
</div>

<style>
  .note-container {
    justify-content: flex-start;
    padding: 24px;
    background-color: #222;
  }
</style>