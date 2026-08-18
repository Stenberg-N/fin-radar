<script lang="ts">
  import { cubicInOut } from "svelte/easing";
  import { slide } from "svelte/transition";

  import { sendAlert } from "$lib/alert";
  import { calendarTags, deleteCalendarTag, addCalendarTag } from "$lib/calendar";
  import { t } from "$lib/i18n";

  let {
    setListVisibility,
  }: {
    setListVisibility: (state: boolean) => void;
  } = $props();

  let isNewTagNameInput = $state<boolean>(false);
  let newTagName = $state<string | null>(null);
</script>

<div id="calendar-tags-list-container">
  <div id="calendar-tags-top-bar" class="horizontal-flex-container">
    <h2>{$t["calendar.tags-list-header"]}</h2>
    <button class="transparent-button-highlight" onclick={() => setListVisibility(false)}>
      <img src="close-x.svg" alt="Close X" class="img-small" />
    </button>
  </div>
  <div id="calendar-tags-toolbar" class="horizontal-flex-container">
    <button class="primary-button" onclick={() => isNewTagNameInput = !isNewTagNameInput}>
      <img src="plus.svg" alt="Plus" class="img-small" style="transform: rotate({isNewTagNameInput ? '-45deg' : ''});" />
    </button>
    {#if isNewTagNameInput}
      <div id="calendar-tags-search-container" class="horizontal-flex-container" transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
        <input class="primary-input" bind:value={newTagName} placeholder={$t["calendar.tags-list.add-tag.input"] as string} />
        <button class="transparent-button-highlight" onclick={() => newTagName = null}>
          <img src="close-x.svg" alt="Close X" />
        </button>
        <button class="transparent-button-highlight" onclick={() => addCalendarTag(newTagName)}>{$t["add.button"]}</button>
      </div>
    {/if}
  </div>
  <div id="calendar-tags-container" class="vertical-flex-container">
    {#each $calendarTags as tag (tag.id)}
      <div class="calendar-tag-row horizontal-flex-container">
        <p title={tag.name}>{tag.name}</p>
        <button class="transparent-button-highlight"
          onclick={() => sendAlert({
            message: "alert.delete-calendar-tag.confirmation",
            isTimer: false,
            buttons: true,
            additionalText: [tag.name],
            onConfirm: () => deleteCalendarTag(tag.id)
          })}
        >
          <img src="trash-can.svg" alt="Trash can" class="img-small" />
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  #calendar-tags-list-container {
    flex-shrink: 0;
    width: 360px;
    padding: 16px 24px;
    background-color: #222;
    border-radius: 8px;

    button.primary-button {
      height: 32px;
      width: 32px;
      background-color: #444;
    }

    button.primary-button:hover {
      background-color: #555;
    }
  }

  #calendar-tags-top-bar {
    position: relative;
    border-bottom: 2px solid #333;
    padding-bottom: 16px;

    h2 {
      margin: 0;
    }

    button {
      position: absolute;
      right: 0;
      width: 32px;
      height: 32px;
    }
  }

  #calendar-tags-toolbar {
    justify-content: flex-start;
    gap: 12px;
    padding: 16px 0;

    > button:first-of-type img {
      transition: transform 0.1s;
    }

    #calendar-tags-search-container {
      position: relative;
      gap: 6px;
      background-color: #444;
      border-radius: 4px;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button:first-of-type {
        flex-shrink: 0;
        width: 20px;
        height: 20px;

        img {
          height: 10px;
          width: 10px;
        }
      }

      button:last-of-type {
        height: 32px;
        padding: 8px;
        border-radius: 0 4px 4px 0;
        color: #f6f6f6;
      }

      input.primary-input {
        outline: none;
        color: #f6f6f6;
      }
    }
  }

  #calendar-tags-container {
    padding: 8px 0;
    gap: 4px;

    .calendar-tag-row {
      justify-content: space-between;
      width: 100%;
      gap: 12px;
      padding: 8px;
      background-color: #333;
      border-radius: 4px;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button {
        flex-shrink: 0;
        width: 32px;
        height: 32px;
        border-radius: 4px;
      }
    }

    p {
      margin: 0;
      overflow: hidden;
      text-wrap: nowrap;
      text-overflow: ellipsis;
    }
  }
</style>