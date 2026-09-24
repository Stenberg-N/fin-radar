<script lang="ts">
  import { t } from "$lib/i18n/i18n";
  import { updateUserPrefs, userPrefs } from "$lib/prefsStore";

  const notesOptions = [
    {
      get title() { return $t["settings.pages.notes.columns"]; },
      options: [1, 2, 3, 4, 5],
      get currentValue() { return $userPrefs.notePrefs.noteColumns; },
      set currentValue(value: number) { updateUserPrefs("notePrefs", "noteColumns", value); },
    },
    {
      get title() { return $t["settings.pages.notes.height"]; },
      options: ["100%", "50%"],
      get currentValue() { return $userPrefs.notePrefs.noteHeight; },
      set currentValue(value: "100%" | "50%") { updateUserPrefs("notePrefs", "noteHeight", value); },
    },
    {
      get title() { return $t["settings.pages.notes.main-bg-color"]; },
      options: ["dark", "light"],
      get currentValue() { return $userPrefs.notePrefs.mainBgColor; },
      set currentValue(value: "dark" | "light") { updateUserPrefs("notePrefs", "mainBgColor", value); },
    },
    {
      get title() { return $t["settings.pages.notes.note-bg-color"]; },
      options: ["dark", "light"],
      get currentValue() { return $userPrefs.notePrefs.noteBgColor; },
      set currentValue(value: "dark" | "light") { updateUserPrefs("notePrefs", "noteBgColor", value); },
    },
  ];
</script>

<div id="main-settings-notes-page-container" class="main-settings-page-container flex column">
  {#each notesOptions as option, i (i)}
    <div class="main-settings-note-option-container flex row">
      <p>{option.title}:</p>
      <select bind:value={option.currentValue} class="primary-input">
        {#each option.options as value, idx (value)}
          <option value={value}>
            {[2, 3].includes(i) ? $t["notes.bg-color-options"][idx] : value}
          </option>
        {/each}
      </select>
    </div>
  {/each}
</div>

<style>
  #main-settings-notes-page-container {
    gap: 0.5rem;

    .main-settings-note-option-container {
      justify-content: flex-start;
      width: 100%;
      gap: 1rem;
      padding: 1rem;
      border-radius: 0.5rem;
      outline: 2px solid var(--outline-color1);

      select.primary-input {
        width: unset;
        height: unset;

        &:hover {
          cursor: pointer;
        }

        option {
          background-color: var(--color-primary2);
        }
      }

      p {
        margin: 0;
        white-space: nowrap;
      }
    }
  }
</style>