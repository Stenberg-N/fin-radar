<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { calendarDate } from "$lib/calendar";
  import { userPrefs } from "$lib/utils/prefsStore";

  import ModalWrapper from "./ModalWrapper.svelte";

  let {
    options,
  }: {
    options: {
      onChangeDate: (date: Date) => void;
    }
  } = $props();

  const MAIN_BAR = 34 + 2; // Actual height + small offset

  let dateNavToggle = $state<HTMLButtonElement | null>(null);
  let isDateNavVisible = $state<boolean>(false);
  let navDateValue = $state<number>($calendarDate.getFullYear());
  let position = $state({ isPositionAbsolute: true, centerElement: true, left: 0, top: 0 });

  const updatePosition = () => {
    if (!dateNavToggle) return;
    const rect = dateNavToggle.getBoundingClientRect();

    position = {
      isPositionAbsolute: true,
      centerElement: true,
      left: rect.left - ($userPrefs.mainPrefs.navBarWidth + 16) + rect.width / 2,
      top: rect.bottom - MAIN_BAR,
    };
  };

  const toggle = () => {
    if (!isDateNavVisible) {
      updatePosition();
      navDateValue = $calendarDate.getFullYear();
    }
    isDateNavVisible = !isDateNavVisible;
  };
</script>

<svelte:window onresize={() => isDateNavVisible && updatePosition()} />

<div id="date-nav-container">
  {#if isDateNavVisible}
    <ModalWrapper
      options={{
        position: position,
        transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
        outline: { width: 1, color: 'var(--outline-color1)'},
        borderRadius: 8,
        onOutsideClick: () => { isDateNavVisible = false; },
        ignorableEls: [dateNavToggle]
      }}
    >
      <div id="date-nav-modal-container">
        <div id="topbar" class="flex">
            {#each Array.from({ length: 2 }, (_, i) => i) as btn (btn)}
              <button aria-label={`${btn === 0 ? "Decrease" : "Increase"} year by 1`} class="button-primary transparent highlight default-corners lower-padding"
                onclick={() => btn === 0 ? navDateValue-- : navDateValue++}
              >
                <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate({btn === 0 ? '90' : '-90'}deg);"></span>
              </button>
              {#if btn === 0}
                <input bind:value={navDateValue} type="number" readonly />
              {/if}
            {/each}
          <button aria-label="Reset to today" class="button-primary transparent highlight default-corners lower-padding"
            onclick={() => { options.onChangeDate(new Date()); navDateValue = $calendarDate.getFullYear(); }}
          >
            <span class="span-icon img-extra-small" style="mask-image: url('/refresh.svg');"></span>
          </button>
        </div>
        <div id="month-grid">
          {#each (i18n.t["calendar.monthnames"] as string[]) as month, i (i)}
            <button class="button-primary transparent highlight default-corners"
              onclick={() => options.onChangeDate(new Date(navDateValue, i, 1))}
            >
              {i18n.lang === 'en' ? month.slice(0, 3) : month.slice(0, -3)}
            </button>
          {/each}
        </div>
      </div>
    </ModalWrapper>
  {/if}

  <button bind:this={dateNavToggle} class="button-primary transparent highlight outline default-corners" onclick={toggle}>
    {$calendarDate.getFullYear()}
    <span class="span-icon img-extra-small" style="mask-image: url('/arrow.svg'); transform: rotate({isDateNavVisible ? '' : '-90'}deg); transition: 0.1s ease-in-out;"></span>
  </button>
</div>

<style>
  #date-nav-container {

    #date-nav-modal-container {
      background-color: var(--color-secondary1);

      input {
        font: inherit;
        field-sizing: content;
        padding: 0;
        background-color: transparent;
        border: none;
        outline: none;
        cursor: default;
      }

      > div {
        padding: 0.5rem;
      }

      #topbar {
        position: relative;
        gap: 0.25rem;
        background-color: var(--color-secondary1-a);

        > button:last-of-type {
          position: absolute;
          right: 0.5rem;
        }

        button {
          height: fit-content;
        }
      }

      #month-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 0.5rem;
      }
    }
  }
</style>