<script lang="ts">
  import { cubicInOut } from "svelte/easing";
  import { fly } from "svelte/transition";

  import { close } from "$lib/alert";
  import type { Alert } from "$lib/types";
  import { t } from "$lib/i18n";

  let {
    alert,
  }: {
    alert: Alert;
  } = $props();

  const duration = 4000;
  let remainingTime = $state<number>(duration);
  let startTime: number;
  let timerId: number | null = null;
  let intervalId: number | null = null;
  let isHovered = $state<boolean>(false);

  let alertEl: HTMLDivElement | null = null;

  const startTimer = () => {
    startTime = Date.now();
    const initialRemaining = remainingTime;

    timerId = window.setTimeout(() => {
      close(alert.id);
    }, remainingTime);

    intervalId = window.setInterval(() => {
      const newRemaining = Math.max(0, initialRemaining - (Date.now() - startTime));

      if (newRemaining <= 0 && intervalId !== null) {
        remainingTime = 0;
        clearInterval(intervalId);
        intervalId = null;
      } else {
        remainingTime = newRemaining;
      }
    }, 5);
  };

  const pauseTimer = () => {
    if (timerId !== null) {
      window.clearTimeout(timerId);
      timerId = null;
    }
    if (intervalId !== null) {
      window.clearInterval(intervalId);
      intervalId = null;
    }
  };

  $effect(() => {
    if (alert.isTimer && !isHovered) startTimer();
    return () => {
      if (timerId !== null) {
        window.clearTimeout(timerId);
        timerId = null;
      }
      if (intervalId !== null) {
        window.clearInterval(intervalId);
        intervalId = null;
      }
    };
  });

  $effect(() => {
    if (alertEl) {
      const progress = `${((duration - remainingTime) / duration) * 100}%`;
      alertEl.style.setProperty('--progress-width', progress);
    }
  });

</script>

<div role="alert" class="alert" bind:this={alertEl} transition:fly={{ y: 50, duration: 200, easing: cubicInOut }} onmouseenter={() => { pauseTimer(); isHovered = true; }} onmouseleave={() => { isHovered = false; }}>
  <p id="alert-message">
    {#if Array.isArray($t[alert.message])}
      {#each $t[alert.message] as msg, i (i)}
        <span>{msg}</span>
      {/each}
    {:else}
      {$t[alert.message]}
    {/if}
  </p>
  {#if alert.buttons}
    <div id="alert-buttons">
      <button class="primary-button" onclick={() => { alert.onConfirm(); close(alert.id); }}>{$t["confirm.button"]}</button>
      <button class="primary-button" onclick={() => { alert.onCancel(); close(alert.id); }}>{$t["cancel.button"]}</button>
    </div>
  {/if}
</div>

<style>
  .alert {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 420px;
    background-color: #181818;
    padding: 16px 12px;
    outline: 1px solid #333;
    border: none;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
    transition: width 100ms linear;
    overflow: hidden;
  }

  .alert::after {
    position: absolute;
    content: '';
    bottom: 0;
    left: 0;
    width: var(--progress-width);
    height: 3px;
    background-color: #f6f6f6;
  }

  .alert:hover {
    outline-color: rgba(255, 70, 70, 1);
  }

  #alert-message {
    text-align: left;
    margin: 0;
    word-wrap: break-word;
    hyphens: auto;
  }

  #alert-message span {
    position: relative;
    display: list-item;
    list-style-type: none;
  }

  #alert-message span:not(:first-child, :nth-child(2)) {
    padding-left: 2rem;
  }

  #alert-message span:not(:first-child, :nth-child(2))::before {
    content: '•';
    position: absolute;
    left: 1rem;
  }

  #alert-buttons {
    display: flex;
    flex-direction: row;
    gap: 10px;
    padding-top: 12px;
    border-top: 1px solid #333;
  }
</style>