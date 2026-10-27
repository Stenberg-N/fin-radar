<script lang="ts">
  import { cubicInOut } from "svelte/easing";
  import { slide } from "svelte/transition";

  import { sendAlert } from "$lib/alert.svelte";
  import { calendarTags, deleteCalendarTag, addCalendarTag } from "$lib/calendar";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import type { CalendarTag, CalendarEventForm } from "$lib/types";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";

  interface Props {
    /**
     * Returns a boolean false when the close button is clicked.
     * 
     * Assign this value to a $state to conditionally render the component.
     */
    setListVisibility: (state: boolean) => void;
    /** Returns a tag that was clicked and was not in the passed form. */
    onAddButtonClick?: (tag: CalendarTag) => void;
    /**
     * Used to optionally pass a calendar event form.
     * 
     * If given, checks which tags are in the given from and marks them differently.
     */
    form?: CalendarEventForm;
    theme?: "default" | "lighter"; 
  }

  type HoverContent = { content: string };

  let {
    setListVisibility,
    onAddButtonClick = undefined,
    form = undefined,
    theme = "default",
  }: Props = $props();

  const themes = {
    default: {
      bg: "var(--color-secondary2)",
      border: "var(--outline-color2)",
      tagBg: "var(--color-secondary4)",
    },
    lighter: {
      bg: "var(--color-secondary3)",
      border: "var(--outline-color3)",
      tagBg: "var(--color-secondary5)",
    }
  }
  const t = $derived(themes[theme] ?? themes.default);

  const TAG_ROW_HEIGHT = 48;
  const TAG_ROW_GAPS = 24;
  let isNewTagNameInput = $state<boolean>(false);
  let newTagName = $state<string | null>(null);
  const hover = new HoverTitle<HoverContent>();

  $effect(() => {
    return () => { hover.destroy(); };
  });

  const handleAddCalendarTag = async (tagName: string | null) => {
    const result = await addCalendarTag(tagName);
    if (result.success) newTagName = null;
  };
</script>

<div
  id="calendar-tags-list-container"
  style:--bg={t.bg}
  style:--calendar-tag-row-height={`${TAG_ROW_HEIGHT}px`}
  style:--border={t.border}
  style:--tag-bg={t.tagBg}
>
  {#if hover.isHovering}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ centerElement: true, moveTop: -40 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      borderRadius={8}
      outline={{ width: 1, color: 'var(--outline-color3)'}}
    >
      <p id="tags-list-hover-title-content">
        {hover.target?.content}
      </p>
    </ModalWrapper>
  {/if}

  <div id="calendar-tags-top-bar" class="flex row">
    <h2>{i18n.t["calendar.tags-list-header"]}</h2>
    <button aria-label="Close list" class="button-primary transparent highlight static" onclick={() => setListVisibility(false)}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
  </div>
  {#if !onAddButtonClick && !form}
    <div id="calendar-tags-toolbar" class="flex row">
      <button aria-label="Toggle tag name input" class="button-primary light static" onclick={() => isNewTagNameInput = !isNewTagNameInput}>
        <span class="span-icon img-small" style="mask-image: url('plus.svg'); transform: rotate({isNewTagNameInput ? '-45deg' : ''});"></span>
      </button>
      {#if isNewTagNameInput}
        <div id="calendar-tags-create-container" class="flex row" transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
          <input class="primary-input" bind:value={newTagName} placeholder={i18n.t["calendar.tags-list.add-tag.input"] as string}
            onkeydown={(e) => {
              e.stopPropagation();
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
            {i18n.t["add.button"]}
          </button>
        </div>
      {/if}
    </div>
  {/if}
  <div id="calendar-tags-container-outer" style="height: {TAG_ROW_HEIGHT * 5 + TAG_ROW_GAPS}px;">
    <div id="calendar-tags-container" class="flex column">
      {#each $calendarTags as tag (tag.id)}
        <div class="calendar-tag-row flex row">
          <p
            onmouseenter={() => hover.enter({ content: tag.name })}
            onmouseleave={(e) => hover.leave(e)}
          >
            {tag.name}
          </p>
          <div class="flex row">
            {#if onAddButtonClick && form}
              <button aria-label="Add tag" class="button-primary transparent highlight static" onclick={() => onAddButtonClick ? onAddButtonClick(tag) : {}} disabled={form?.tags.some(t => t.id === tag.id)}
                style="opacity: 1;"
              >
                <span class="span-icon img-small"
                  style="{form?.tags.some(t => t.id === tag.id)
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
    background-color: var(--bg);

    #tags-list-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary3);
    }
  }

  #calendar-tags-top-bar {
    justify-content: space-between;
    padding-bottom: 1rem;
    border-bottom: 2px solid var(--border);

    h2 {
      margin: 0;
    }
  }

  #calendar-tags-toolbar {
    justify-content: flex-start;
    gap: 0.75rem;
    padding: 1rem 10px;

    > button:first-of-type {
      box-shadow: none;

      span {
        transition: transform 0.1s;
      }
    }

    #calendar-tags-create-container {
      position: relative;
      gap: 6px;
      background-color: var(--color-secondary5);
      border-radius: 0.25rem;

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
      border-radius: 0.25rem;
      background-color: var(--tag-bg);

      button {
        border-radius: 0.25rem;
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