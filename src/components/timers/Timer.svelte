<script lang="ts">
  import { deleteTimer, timerRuntimes, queueTimerUpdate, startTimerCountdown, stopTimerCountdown } from "$lib/timers";
  import { user } from "$lib/user";
  import type { Timer } from "$lib/types";
  import { beforeNavigate, goto } from "$app/navigation";
  import { t, lang } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";

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
  let selectedDurationEl = $state<{idx: number, inputEl: HTMLInputElement}>()

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

  const handleTimerDelete = async () => {
    if (!$user) return;
    const result = await deleteTimer($user.id, $user.name, timer.id);
    if (!result.success) sendAlert("alert.delete-timer.fail", true, false);
  };

  const handleTimerDurationStep = (delta: number) => {
    if (!selectedDurationEl) return;

    const { idx, inputEl } = selectedDurationEl;
    let newValue = Number(inputEl.value) + delta;
    if (newValue < 0) newValue = 0;
    if (Number(inputEl.value) === Number(newValue)) return;
    inputEl.value = String(newValue).padStart(2, '0');
    idx === 0 ? updateTimerDuration(Number(inputEl.value), displaySeconds) : updateTimerDuration(displayMinutes, Number(inputEl.value));
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
    <button class="transparent-button-highlight" onclick={() => toggleTimer()}>
      <img src={isTimerRunning ? "/pause-circle.svg" : "/play-circle.svg"} alt={isTimerRunning ? "Pause" : "Play"} />
    </button>
    <button class="transparent-button-highlight horizontal-flex-container" onclick={() => sendAlert("alert.delete-timer.confirmation", false, true, async () => handleTimerDelete(), undefined, timer.title)}>
      <img src="/trash-can.svg" alt="Trash can" />
    </button>
    {#each [{ command: () => handleTimerDurationStep(1) }, { command: () => handleTimerDurationStep(-1) }] as stepper, i (i)}
      <button class="transparent-button-highlight" onclick={() => stepper.command()}><img src="arrow.svg" alt="Arrow" class="img-small" style="transform: {i === 0 ? 'rotate(180deg)' : ''};" /></button>
    {/each}
    <p class="timer-state" style="color: {!isTimerRunning && timerDuration > 0 ? "#f6f6f6" : isTimerRunning ? "rgb(255, 70, 70)" : "rgb(115, 240, 115)"}; user-select: none;">
      {(!isTimerRunning && timerDuration > 0)
        ? $t["timers.state.paused"]
        : isTimerRunning
          ? $t["timers.state.running"]
          : $t["timers.state.finished"]}
    </p>
  </div>
  <div class="timer-content vertical-flex-container">
    <input class="timer-title primary-input" class:no-interaction={isTimerRunning} disabled={isTimerRunning} oninput={() => scheduleUpdate()} bind:value={timerTitle} />

    <div class="duration-container horizontal-flex-container">
      {#each [{ value: displayMinutes }, { value: displaySeconds }] as input, i (i)}
        <input type="number" min="0" class="primary-input" class:no-interaction={isTimerRunning}
          disabled={isTimerRunning}
          onkeydown={(e) => handleTimerInput(e)}
          oninput={(e) => i === 0 ? updateTimerDuration(+e.currentTarget.value, displaySeconds) : updateTimerDuration(displayMinutes, +e.currentTarget.value)}
          onclick={(e) => selectedDurationEl = { idx: i, inputEl: e.target as HTMLInputElement }}
          value={String(input.value).padStart(2, '0')}
        />
        {#if i === 0}
          <span style="user-select: none;">:</span>
        {/if}
      {/each}
    </div>

    <textarea
      class:no-interaction={isTimerRunning}
      disabled={isTimerRunning}
      placeholder={$lang === 'en' ? "Add an optional timer message..." : "Lisää vaihtoehtoinen viesti ajastimeen..."}
      oninput={() => scheduleUpdate()} bind:value={timerMessage}
    ></textarea>
  </div>
</div>

<style>
  .no-interaction {
    pointer-events: none;
  }

  .primary-input {
    color: #f6f6f6;
  }

  .timer-container {
    justify-content: flex-start;
    flex-shrink: 0;
    height: 240px;
    min-width: 280px;
    max-width: calc((100% - 80px) / 5);
    gap: 10px;
    padding: 8px;
    border-radius: 8px;
    background-color: #222;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }

  .timer-controls {
    justify-content: flex-start;
    width: 100%;
    height: 32px;
    gap: 4px;
    padding: 0 8px 4px;
    border-bottom: 1px solid #333;
  }

  .timer-controls button {
    gap: 8px;
    height: 30px;
    width: 30px;
  }

  .timer-controls button img {
    width: 20px;
    height: 20px;
  }

  .timer-state {
    margin: 0 0 0 auto;
    font-weight: bold;
    font-size: clamp(1rem, 0.9cqw, 1.2rem);
  }

  .timer-content {
    justify-content: flex-start;
    height: 100%;
    width: 100%;
    gap: 12px;
    padding: 0 18px 18px;
  }

  .timer-title {
    max-height: 32px;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
    text-align: center;
  }

  .duration-container {
    width: 100%;
    gap: 6px;
  }

  .duration-container > * {
    font-size: 2rem;
    font-weight: bold;
    text-align: center;
  }

  .duration-container .primary-input {
    width: 100%;
    height: 3rem;
  }

  .timer-container textarea {
    height: 100%;
    width: 100%;
  }
</style>