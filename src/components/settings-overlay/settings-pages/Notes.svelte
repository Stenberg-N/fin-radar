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
      get options() { return $t["notes.bg-color-options"] },
      get currentValue() { return $userPrefs.notePrefs.mainBgColor; },
      set currentValue(value: "dark" | "light") { updateUserPrefs("notePrefs", "mainBgColor", value); },
    },
    {
      get title() { return $t["settings.pages.notes.note-bg-color"]; },
      get options() { return $t["notes.bg-color-options"] },
      get currentValue() { return $userPrefs.notePrefs.noteBgColor; },
      set currentValue(value: "dark" | "light") { updateUserPrefs("notePrefs", "noteBgColor", value); },
    },
  ];
</script>

<div id="main-settings-notes-page-container" class="main-settings-page-container vertical-flex-container">
  {#each notesOptions as option, i (i)}
    <div class="main-settings-note-option-container horizontal-flex-container">
      <p>{option.title}:</p>
      <select bind:value={option.currentValue} class="primary-input">
        {#each option.options as value (value)}
          <option value={[2, 3].includes(i) ? String(value).toLowerCase() : value}>{value}</option>
        {/each}
      </select>
    </div>
  {/each}
</div>

<style>
  #main-settings-notes-page-container {

    .main-settings-note-option-container {
      justify-content: flex-start;
      width: 100%;
      gap: 12px;

      select.primary-input {
        width: unset;
        height: unset;

        option {
          background-color: #0f0f0f;
        }
      }

      p {
        white-space: nowrap;
      }
    }
  }
</style>