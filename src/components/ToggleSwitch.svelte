<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "$lib/i18n/i18n";

  import ModalWrapper from "./ModalWrapper.svelte";

  let {
    onClickCommand,
    activeDerivedFrom,
    translationKey,
    height,
  }: {
    onClickCommand: () => void;
    activeDerivedFrom: boolean;
    translationKey: string;
    height: number;
  } = $props();

  let toggleSwitch: HTMLButtonElement | null = null;
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let isHovering = $state(false);

  onMount(() => {
    toggleSwitch?.style.setProperty('--toggle-thumb-dimensions', `${height - 4}px`);
    toggleSwitch?.style.setProperty('--toggle-thumb-slide-length', `${height}px`);
  });

  $effect(() => {
    return () => { if (timeout) clearTimeout(timeout); };
  });

  const handleMouseEnter = () => {
    if (timeout) clearTimeout(timeout);

    timeout = setTimeout(() => {
      isHovering = true;
    }, 300);
  };

  const handleMouseLeave = () => {
    if (timeout) clearTimeout(timeout);
    timeout = null;

    isHovering = false;
  };
</script>

<button
  bind:this={toggleSwitch}
  aria-label={$t[translationKey] as string}
  id="toggle-track"
  class="button-primary transparent highlight"
  class:active={activeDerivedFrom}
  style="min-height: {height}px; height: {height}px; width: {height * 2}px;"
  onclick={() => onClickCommand()}
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  {#if isHovering}
    <ModalWrapper options={{
      position: { centerElement: true, moveTop: -40 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color1)'}
      }}
    >
      <p id="toggle-switch-hover-title-content">
        {$t[translationKey] as string}
      </p>
    </ModalWrapper>
  {/if}

  <span class="toggle-thumb"></span>
</button>

<style>
  #toggle-switch-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary1);
  }

  #toggle-track {
    position: relative;
    border-radius: 9999px;
    outline: 2px solid var(--outline-color3);
    transition: background-color 0.2s;

    &:focus {
      outline: 2px solid var(--color-highlight1);
    }

    &.active {
      background-color: var(--color-highlight1);

      .toggle-thumb {
        transform: translateX(var(--toggle-thumb-slide-length));
      }

      &:hover {
        background-color: rgba(255, 70, 70, 0.7);
      }
    }
  }
  .toggle-thumb {
    position: absolute;
    left: 2px;
    top: 2px;
    width: var(--toggle-thumb-dimensions);
    height: var(--toggle-thumb-dimensions);
    background-color: var(--color-white-primary1);
    border-radius: 50%;
    transform: translateX(0);
    transition: transform 0.2s;
  }
</style>