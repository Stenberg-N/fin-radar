<script lang="ts">
  import { onMount } from "svelte";

  import { t } from "$lib/i18n/i18n";

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

  onMount(() => {
    toggleSwitch?.style.setProperty('--toggle-thumb-dimensions', `${height - 4}px`);
    toggleSwitch?.style.setProperty('--toggle-thumb-slide-length', `${height}px`);
  });
</script>

<button
  bind:this={toggleSwitch}
  style="min-height: {height}px; height: {height}px; width: {height * 2}px;"
  class="toggle-track transparent-button-highlight"
  title={$t[translationKey] as string}
  class:active={activeDerivedFrom}
  onclick={() => onClickCommand()}
>
  <span class="toggle-thumb"></span>
</button>

<style>
  .toggle-track {
    position: relative;
    border-radius: 9999px;
    outline: 2px solid rgb(180, 180, 180);
    transition: background-color 0.2s;
  }
  .toggle-track:focus {
    outline: 2px solid rgba(255, 70, 70, 1);
  }
  .toggle-track.active {
    background-color: rgba(255, 70, 70, 1);
  }
  .toggle-track.active:hover {
    background-color: rgba(255, 70, 70, 0.7);
  }

  .toggle-thumb {
    position: absolute;
    left: 2px;
    top: 2px;
    width: var(--toggle-thumb-dimensions);
    height: var(--toggle-thumb-dimensions);
    background-color: #f6f6f6;
    border-radius: 50%;
    transform: translateX(0);
    transition: transform 0.2s;
  }

  .toggle-track.active .toggle-thumb {
    transform: translateX(var(--toggle-thumb-slide-length));
  }
</style>