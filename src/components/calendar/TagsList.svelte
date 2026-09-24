<script lang="ts">
  import { cubicInOut } from "svelte/easing";
  import { slide } from "svelte/transition";
  import { onMount } from "svelte";

  import { sendAlert } from "$lib/alert";
  import { calendarTags, deleteCalendarTag, addCalendarTag } from "$lib/calendar";
  import { t } from "$lib/i18n/i18n";
  import type { CalendarTag, CalendarEventForm } from "$lib/types";
  import { handleClickOutside } from "$lib/actions";

  let {
    options,
  }: {
    options: {
      setListVisibility: (state: boolean) => void;
      tagsListToggleButton: HTMLButtonElement | null;
      isTagsListVisible: boolean;
      onAddButtonClick?: (tag: CalendarTag) => void;
      form?: CalendarEventForm;
    }
  } = $props();

  const TAG_ROW_HEIGHT = 48;
  const TAG_ROW_GAPS = 24;
  let isNewTagNameInput = $state<boolean>(false);
  let newTagName = $state<string | null>(null);

  onMount(() => {
    document.documentElement.style.setProperty('--calendar-tag-row-height', `${TAG_ROW_HEIGHT}px`);
  });

  const handleAddCalendarTag = async (tagName: string | null) => {
    const result = await addCalendarTag(tagName);
    if (result.success) newTagName = null;
  };
</script>

<div id="calendar-tags-list-container"
  use:handleClickOutside={{ onOutsideClick: () => options.setListVisibility(false), additionalElements: [options.tagsListToggleButton]}}
>
  <div id="calendar-tags-top-bar" class="flex row">
    <h2>{$t["calendar.tags-list-header"]}</h2>
    <button aria-label="Close list" class="button-primary transparent highlight static" onclick={() => options.setListVisibility(false)}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
  </div>
  {#if !options.onAddButtonClick && !options.form}
    <div id="calendar-tags-toolbar" class="flex row">
      <button aria-label="Toggle tag name input" class="button-primary light static" onclick={() => isNewTagNameInput = !isNewTagNameInput}>
        <span class="span-icon img-small" style="mask-image: url('plus.svg'); transform: rotate({isNewTagNameInput ? '-45deg' : ''});"></span>
      </button>
      {#if isNewTagNameInput}
        <div id="calendar-tags-create-container" class="flex row" transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
          <input class="primary-input" bind:value={newTagName} placeholder={$t["calendar.tags-list.add-tag.input"] as string}
            onkeydown={(e) => {
              switch (e.key) {
                case 'Enter': handleAddCalendarTag(newTagName); break;
                case 'Escape': newTagName = null; break;
              }
            }}
          />
          <button aria-label="Clear tag name" class="button-primary transparent highlight" onclick={() => newTagName = null}>
            <span class="span-icon" style="mask-image: url('close-x.svg');"></span>
          </button>
          <button class="button-primary transparent highlight" onclick={() => handleAddCalendarTag(newTagName)}>
            <span class="span-icon img-small" style="mask-image: url('/plus.svg');"></span>
            {$t["add.button"]}
          </button>
        </div>
      {/if}
    </div>
  {/if}
  <div id="calendar-tags-container-outer" style="height: {TAG_ROW_HEIGHT * 5 + TAG_ROW_GAPS}px;">
    <div id="calendar-tags-container" class="flex column">
      {#each $calendarTags as tag (tag.id)}
        <div class="calendar-tag-row flex row">
          <p title={tag.name}>{tag.name}</p>
          <div class="flex row">
            {#if options.onAddButtonClick && options.form}
              <button aria-label="Add tag" class="button-primary transparent highlight static" onclick={() => options.onAddButtonClick ? options.onAddButtonClick(tag) : {}} disabled={options.form?.tags.some(t => t.id === tag.id)}
                style="opacity: 1;"
              >
                <span class="span-icon img-small"
                  style="{options.form?.tags.some(t => t.id === tag.id)
                    ? 'width: 20px; height: 20px; background-color: var(--color-positive); mask-image: url("check-circle.svg");'
                    : 'mask-image: url("plus.svg");'
                  }"
                ></span>
              </button>
            {:else}
              <button aria-label="Delete tag" class="button-primary transparent highlight static"
                onclick={() => sendAlert({
                  message: "alert.delete-calendar-tag.confirmation",
                  isTimer: false,
                  buttons: true,
                  additionalText: [tag.name],
                  onConfirm: () => deleteCalendarTag(tag.id)
                })}
              >
                <span class="span-icon img-small" style="mask-image: url('trash-can.svg');"></span>
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  #calendar-tags-list-container {
    flex-shrink: 0;
    width: 360px;
    padding: 1rem 1.5rem;
    background-color: var(--color-secondary1);
    border-radius: 0.5rem;
  }

  #calendar-tags-top-bar {
    justify-content: space-between;
    border-bottom: 2px solid var(--outline-color1);
    padding-bottom: 1rem;

    h2 {
      margin: 0;
      color: var(--color-white-primary1);
    }
  }

  #calendar-tags-toolbar {
    justify-content: flex-start;
    gap: 0.75rem;
    padding: 1rem 10px;

    > button:first-of-type span {
      transition: transform 0.1s;
    }

    #calendar-tags-create-container {
      position: relative;
      gap: 6px;
      background-color: var(--color-secondary3);
      border-radius: 0.25rem;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button:first-of-type {
        flex-shrink: 0;
        width: 20px;
        height: 20px;

        span {
          height: 10px;
          width: 10px;
        }
      }

      button:last-of-type {
        height: 2rem;
        padding: 0.5rem;
        border-radius: 0 0.25rem 0.25rem 0;
      }

      input.primary-input {
        outline: none;
      }
    }
  }

  #calendar-tags-container-outer {
    padding: 0.25rem;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
  }

  #calendar-tags-container {
    gap: 0.25rem;

    .calendar-tag-row {
      justify-content: space-between;
      width: 100%;
      height: var(--calendar-tag-row-height);
      gap: 0.75rem;
      padding: 0.5rem;
      background-color: var(--color-secondary2);
      border-radius: 0.25rem;

      button {
        border-radius: 0.25rem;
      }
    }

    p {
      margin: 0;
      overflow: hidden;
      text-wrap: nowrap;
      text-overflow: ellipsis;
      color: var(--color-white-primary1);
    }
  }
</style>