<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { updateUserPrefs, userPrefs } from "$lib/utils/prefsStore";
  import ItemSelect from "../../elements/ItemSelect.svelte";

  const notesOptions = [
    {
      get title() { return i18n.t["settings.pages.notes.columns"]; },
      options: ["1", "2", "3", "4", "5"],
      get currentValue() { return String($userPrefs.notePrefs.noteColumns); },
      set currentValue(value: string) { updateUserPrefs("notePrefs", "noteColumns", Number(value)); },
    },
    {
      get title() { return i18n.t["settings.pages.notes.height"]; },
      options: ["50%", "100%"],
      get currentValue() { return $userPrefs.notePrefs.noteHeight; },
      set currentValue(value: "50%" | "100%") { updateUserPrefs("notePrefs", "noteHeight", value); },
    },
    {
      get title() { return i18n.t["settings.pages.notes.main-bg-color"]; },
      options: ["dark", "light"],
      get currentValue() { return $userPrefs.notePrefs.mainBgColor; },
      set currentValue(value: "dark" | "light") { updateUserPrefs("notePrefs", "mainBgColor", value); },
    },
  ];
</script>

<div id="main-settings-notes-page-container" class="main-settings-page-container flex column">
  <div class="flex column">
    {#each notesOptions as option, i (i)}
      <div class="main-settings-note-option-container setting-container flex column">
        <p>{option.title}:</p>
        <ItemSelect
          onItemSelect={(item) => option.currentValue = item}
          data={option.options}
          overrideDefaultStartValue={option.currentValue}
          overrideDataValues={[2, 3].includes(i) ? (i18n.t["notes.bg-color-options"] as string[]) : undefined}
          position={{ moveTop: 36 }}
        />
      </div>
    {/each}
  </div>
</div>

<style>
  #main-settings-notes-page-container {
    padding: 20px;
    border-radius: 1rem;
    background-color: var(--color-secondary2);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    > div {
      width: 100%;
      gap: 1rem;
      padding: 2.5rem;
      border-radius: 0.5rem;
      background-color: var(--color-primary2);
    }

    .main-settings-note-option-container {
      align-items: flex-start;

      p {
        margin: 0;
        white-space: nowrap;
      }
    }
  }
</style>