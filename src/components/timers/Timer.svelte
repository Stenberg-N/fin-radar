<script lang="ts">
  import { deleteTimer, timerRuntimes, queueTimerUpdate, startTimerCountdown, stopTimerCountdown } from "$lib/timers";
  import { user } from "$lib/user";
  import type { Timer } from "$lib/types";
  import { beforeNavigate, goto } from "$app/navigation";

  let {
    timer,
  }: {
    timer: Timer;
  } = $props();

  // svelte-ignore state_referenced_locally
  if (!$timerRuntimes.has(timer.id)) {
    timerRuntimes.update((map) => map.set(timer.id, { isRunning: false, currentDuration: timer.duration }));
  }

  // svelte-ignore state_referenced_locally
  let timerTitle = $state(timer.title);
  // svelte-ignore state_referenced_locally
  let timerMessage = $state(timer.message);
  let timerDuration = $derived($timerRuntimes.get(timer.id)?.currentDuration ?? timer.duration);
  let isTimerRunning = $derived($timerRuntimes.get(timer.id)?.isRunning ?? false);
  let updateDebounce: number;
  let isScheduledUpdate = $state<boolean>(false);
  let pendingNavigation = $state<string | null>(null);

  let displayMinutes = $derived.by(() => Math.floor(timerDuration / 60));
  let displaySeconds = $derived.by(() => timerDuration % 60);

  beforeNavigate(({ to, cancel }) => {
    if (!to || !isScheduledUpdate) return;

    cancel();
    pendingNavigation = to.url.pathname;
  });

  $effect(() => {
    if (pendingNavigation !== null && !isScheduledUpdate) {
      goto(pendingNavigation);
      pendingNavigation = null;
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const scheduleUpdate = () => {
    isScheduledUpdate = true;
    clearTimeout(updateDebounce);
    updateDebounce = setTimeout(() => {
      queueTimerUpdate({ ...timer, duration: timerDuration, title: timerTitle, message: timerMessage });
      isScheduledUpdate = false;
    }, 400);
  };

  const updateTimerDuration = (newMinutes: number, newSeconds: number) => {
    const current = $timerRuntimes.get(timer.id)!;
    timerRuntimes.update((map) => map.set(timer.id, { ...current, currentDuration: newMinutes * 60 + newSeconds }));
    scheduleUpdate();
  };

  const handleTimerInput = (event: KeyboardEvent) => {
    const allowedKeys = ["Backspace", "ArrowLeft", "ArrowRight"];
    const regex = /^[0-9]+$/g;
    if (allowedKeys.includes(event.key)) return;
    if (!regex.test(event.key)) event.preventDefault();
  };

  /***********************************************************************************************************************************/

  const toggleTimer = () => {
    const current = $timerRuntimes.get(timer.id)!;
    const newIsRunning = !current.isRunning;
    timerRuntimes.update((map) => map.set(timer.id, { ...current, isRunning: newIsRunning }));

    if (newIsRunning) {
      startTimerCountdown(timer.id);
    } else {
      stopTimerCountdown(timer.id);
      scheduleUpdate();
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
    <input type="number" min="0" class:no-interaction={isTimerRunning}
      onkeydown={(e) => handleTimerInput(e)}
      oninput={(e) => updateTimerDuration(+e.currentTarget.value, displaySeconds)}
      value={String(displayMinutes).padStart(2, '0')}
    />
    <input type="number" min="0" class:no-interaction={isTimerRunning}
      onkeydown={(e) => handleTimerInput(e)}
      oninput={(e) => updateTimerDuration(displayMinutes, +e.currentTarget.value)}
      value={String(displaySeconds).padStart(2, '0')}
    />
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
    flex-shrink: 0;
    height: 170px;
    min-width: 280px;
    max-width: calc((100% - 80px) / 5);
    padding: 8px 12px;
    border-radius: 8px;
    background-color: #222;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
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