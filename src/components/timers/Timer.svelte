<script lang="ts">
  import { deleteTimer } from "$lib/timers";
  import { user } from "$lib/user";
  import type { Timer } from "$lib/types";

  let {
    timer,
    onUpdate,
  }: {
    timer: Timer;
    onUpdate: (timer: Timer) => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let timerDuration = $state(timer.duration);
  // svelte-ignore state_referenced_locally
  let timerTitle = $state(timer.title);
  // svelte-ignore state_referenced_locally
  let timerMessage = $state(timer.message);
  let timerInterval: number;
  let updateDebounce: number;

  let isTimerRunning = $state<boolean>(false);

  let displayMinutes = $derived.by(() => Math.floor(timerDuration / 60));
  let displaySeconds = $derived.by(() => timerDuration % 60);

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const scheduleUpdate = () => {
    clearTimeout(updateDebounce);
    updateDebounce = setTimeout(() => {
      onUpdate({ ...timer, duration: timerDuration, title: timerTitle, message: timerMessage });
    }, 400);
  };

  const handleStopTimer = () => {
    isTimerRunning = false;
    clearInterval(timerInterval);
    scheduleUpdate();
  };

  const updateTimerDuration = (newMinutes: number, newSeconds: number) => {
    timerDuration = newMinutes * 60 + newSeconds;
    scheduleUpdate();
  };

  /***********************************************************************************************************************************/

  const toggleTimer = () => {
    isTimerRunning = !isTimerRunning;
    clearInterval(timerInterval);
    if (!isTimerRunning) scheduleUpdate();

    if (isTimerRunning) {
      timerInterval = setInterval(() => {
        if (timerDuration <= 0) { handleStopTimer(); return; }
        timerDuration--;
      }, 1000);
    }
  };
</script>

<div class="timer-container vertical-flex-container">
  <div class="timer-controls horizontal-flex-container">
    <button onclick={() => toggleTimer()}>T</button>
    <button class="transparent-button-highlight horizontal-flex-container" onclick={async () => $user ? await deleteTimer($user.id, $user.name, timer.id) : {} }><img src="/trash-can.svg" alt="Trash can" class="img-small" /></button>
  </div>
  <input oninput={() => scheduleUpdate()} bind:value={timerTitle} />
  <div class="duration-container">
    <input type="number" min="0" class:no-interaction={isTimerRunning} oninput={(e) => updateTimerDuration(+e.currentTarget.value, displaySeconds)} value={String(displayMinutes).padStart(2, '0')} />
    <input type="number" min="0" class:no-interaction={isTimerRunning} oninput={(e) => updateTimerDuration(displayMinutes, +e.currentTarget.value)} value={String(displaySeconds).padStart(2, '0')} />
  </div>
  <div class="timer-message-container">
    <textarea oninput={() => scheduleUpdate()} bind:value={timerMessage}></textarea>
  </div>
</div>

<style>
  .no-interaction {
    pointer-events: none;
  }

  .timer-container {
    justify-content: flex-start;
    padding: 8px 12px;
    background-color: #181818;
  }

  .timer-controls {
    justify-content: flex-start;
    width: 100%;
    height: 32px;
    gap: 12px;
    padding: 4px;
  }

  .timer-controls button {
    gap: 8px;
    height: 24px;
    width: 24px;
  }
</style>