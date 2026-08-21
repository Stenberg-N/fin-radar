<script lang="ts">
  import { cubicInOut } from "svelte/easing";
  import { slide } from "svelte/transition";
  import { onMount, getContext } from "svelte";

  import { sendAlert } from "$lib/alert";
  import { calendarTags, deleteCalendarTag, addCalendarTag } from "$lib/calendar";
  import { t } from "$lib/i18n";
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

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  
  /***********************************************************************************************************************************/

  const handleAddCalendarTag = async (tagName: string | null) => {
    const result = await addCalendarTag(tagName);
    if (result.success) newTagName = null;
  };
</script>

<div id="calendar-tags-list-container"
  use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => options.setListVisibility(false), additionalElements: [options.tagsListToggleButton]}}
>
  <div id="calendar-tags-top-bar" class="horizontal-flex-container">
    <h2>{$t["calendar.tags-list-header"]}</h2>
    <button aria-label="Close list" class="transparent-button-highlight" onclick={() => options.setListVisibility(false)}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
  </div>
  {#if !options.onAddButtonClick && !options.form}
    <div id="calendar-tags-toolbar" class="horizontal-flex-container">
      <button aria-label="Toggle tag name input" class="primary-button-light" onclick={() => isNewTagNameInput = !isNewTagNameInput}>
        <span class="span-icon img-small" style="mask-image: url('plus.svg'); transform: rotate({isNewTagNameInput ? '-45deg' : ''});"></span>
      </button>
      {#if isNewTagNameInput}
        <div id="calendar-tags-create-container" class="horizontal-flex-container" transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
          <input class="primary-input" bind:value={newTagName} placeholder={$t["calendar.tags-list.add-tag.input"] as string}
            onkeydown={(e) => {
              switch (e.key) {
                case 'Enter': handleAddCalendarTag(newTagName); break;
                case 'Escape': newTagName = null; break;
              }
            }}
          />
          <button aria-label="Clear tag name" class="transparent-button-highlight" onclick={() => newTagName = null}>
            <span class="span-icon" style="mask-image: url('close-x.svg');"></span>
          </button>
          <button class="transparent-button-highlight" onclick={() => handleAddCalendarTag(newTagName)}>{$t["add.button"]}</button>
        </div>
      {/if}
    </div>
  {/if}
  <div id="calendar-tags-container-outer" style="height: {TAG_ROW_HEIGHT * 5 + TAG_ROW_GAPS}px;">
    <div id="calendar-tags-container" class="vertical-flex-container">
      {#each $calendarTags as tag (tag.id)}
        <div class="calendar-tag-row horizontal-flex-container">
          <p title={tag.name}>{tag.name}</p>
          <div class="horizontal-flex-container">
            {#if options.onAddButtonClick && options.form}
              <button aria-label="Add tag" class="transparent-button-highlight" onclick={() => options.onAddButtonClick ? options.onAddButtonClick(tag) : {}} disabled={options.form?.tags.some(t => t.id === tag.id)}
                style="opacity: 1;"
              >
                <span class="span-icon img-small"
                  style="{options.form?.tags.some(t => t.id === tag.id)
                    ? 'width: 20px; height: 20px; background-color: rgb(170, 255, 170); mask-image: url("check-circle.svg");'
                    : 'mask-image: url("plus.svg");'
                  }"
                ></span>
              </button>
            {:else}
              <button aria-label="Delete tag" class="transparent-button-highlight"
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
    padding: 16px 24px;
    background-color: #222;
    border-radius: 8px;

    button.primary-button-light {
      height: 32px;
      width: 32px;
    }
  }

  #calendar-tags-top-bar {
    position: relative;
    border-bottom: 2px solid #333;
    padding-bottom: 16px;

    h2 {
      margin: 0;
      color: #f6f6f6;
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
    padding: 16px 10px;

    > button:first-of-type span {
      transition: transform 0.1s;
    }

    #calendar-tags-create-container {
      position: relative;
      gap: 6px;
      background-color: #444;
      border-radius: 4px;
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

  #calendar-tags-container-outer {
    padding: 4px;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 2%, rgb(0, 0, 0) 98%, rgba(0, 0, 0, 0));
  }

  #calendar-tags-container {
    gap: 4px;

    .calendar-tag-row {
      justify-content: space-between;
      width: 100%;
      height: var(--calendar-tag-row-height);
      gap: 12px;
      padding: 8px;
      background-color: #333;
      border-radius: 4px;

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
      color: #f6f6f6;
    }
  }
</style>