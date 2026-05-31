<script lang="ts">
  import { getContext } from "svelte";

  import { deleteTimer, timerRuntimes, queueTimerUpdate, startTimerCountdown, stopTimerCountdown } from "$lib/timers";
  import { user } from "$lib/user";
  import type { Timer } from "$lib/types";
  import { beforeNavigate, goto } from "$app/navigation";
  import { t, lang } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { handleClickOutside } from "$lib/functions";

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
  let selectedDurationEl = $state<{idx: number, inputEl: HTMLInputElement} | null>(null);
  let stepperButtonRefs = $state<HTMLButtonElement[]>([]);

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
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');

  const scheduleUpdate = () => {
    isScheduledUpdate = true;
    clearTimeout(updateDebounce);
    updateDebounce = setTimeout(() => {
      queueTimerUpdate({ ...timer, duration: timerDuration, title: timerTitle, message: timerMessage });
      isScheduledUpdate = false;
    }, 400);
  };

  const handleTimerInput = (event: KeyboardEvent) => {
    const allowedKeys = ["Backspace", "ArrowLeft", "ArrowRight"];
    const regex = /^[0-9]+$/g;
    if (allowedKeys.includes(event.key)) return;
    if (!regex.test(event.key)) event.preventDefault();
  };

  const handleTimerDurationStep = (delta: number) => {
    if (!selectedDurationEl || isTimerRunning) return;
    const { idx, inputEl } = selectedDurationEl;
    let newValue = Number(inputEl.value) + delta;

    if (newValue < 0) newValue = 0;
    if (Number(inputEl.value) === Number(newValue)) return;

    inputEl.value = String(newValue).padStart(2, '0');
    idx === 0 ? updateTimerDuration(Number(inputEl.value), displaySeconds) : updateTimerDuration(displayMinutes, Number(inputEl.value));
  };

  const handleEditAttemptWhileRunning = () => {
    sendAlert("alert.cannot-edit.timer-running", true, false);
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

  const handleTimerDelete = async () => {
    if (!$user) return;
    const result = await deleteTimer($user.id, $user.name, timer.id);
    if (!result.success) sendAlert("alert.delete-timer.fail", true, false);
  };

  const updateTimerDuration = (newMinutes: number, newSeconds: number) => {
    const current = $timerRuntimes.get(timer.id)!;
    timerRuntimes.update((map) => map.set(timer.id, { ...current, currentDuration: newMinutes * 60 + newSeconds }));
    scheduleUpdate();
  };
</script>

<div class="timer-controls horizontal-flex-container">
  <button class="transparent-button-highlight" onclick={(e) => { e.stopPropagation(); toggleTimer(); }}>
    <img src={isTimerRunning ? "/pause.svg" : "/play.svg"} alt={isTimerRunning ? "Pause" : "Play"} class="img-small" />
  </button>

  <button class="transparent-button-highlight horizontal-flex-container" onclick={() => sendAlert("alert.delete-timer.confirmation", false, true, () => handleTimerDelete(), undefined, timer.title)}>
    <img src="/trash-can.svg" alt="Trash can" class="img-small" />
  </button>

  {#each [{ command: () => handleTimerDurationStep(1) }, { command: () => handleTimerDurationStep(-1) }] as stepper, i (i)}
    <button bind:this={stepperButtonRefs[i]} class="transparent-button-highlight" class:disabled={!selectedDurationEl} disabled={!selectedDurationEl} onclick={() => stepper.command()} onmousedown={(e) => e.preventDefault()}>
      <img src="arrow.svg" alt="Arrow" class="img-small" style="transform: {i === 0 ? 'rotate(180deg)' : ''};" />
    </button>
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
  <div class="timer-duration-title-container horizontal-flex-container">
    <div class="timer-title-container vertical-flex-container">
      <p class="element-paragraph-title">{$lang === 'en' ? "Title" : "Otsikko"}</p>
      <input class="timer-title primary-input"
        class:no-interaction={isTimerRunning}
        disabled={isTimerRunning}
        oninput={() => scheduleUpdate()} bind:value={timerTitle}
      />
    </div>

    <div class="timer-duration-container horizontal-flex-container">
      {#each [{ value: displayMinutes, unit: "MM" }, { value: displaySeconds, unit: "SS" }] as input, i (i)}
        <div class="timer-duration vertical-flex-container">
          <p class="element-paragraph-title">{input.unit}</p>
          <input type="number" min="0" class="primary-input"
            class:no-interaction={isTimerRunning}
            disabled={isTimerRunning}
            use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => selectedDurationEl = null, additionalElements: stepperButtonRefs }}
            onkeydown={(e) => handleTimerInput(e)}
            oninput={(e) => i === 0 ? updateTimerDuration(+e.currentTarget.value, displaySeconds) : updateTimerDuration(displayMinutes, +e.currentTarget.value)}
            onclick={(e) => selectedDurationEl = { idx: i, inputEl: e.target as HTMLInputElement }}
            value={String(input.value).padStart(2, '0')}
          />
        </div>
        {#if i === 0}
          <span style="user-select: none; margin-top: 10px;">:</span>
        {/if}
      {/each}
    </div>
  </div>

  <textarea
    class="timer-textarea"
    class:no-interaction={isTimerRunning}
    disabled={isTimerRunning}
    placeholder={$lang === 'en' ? "Add an optional timer message..." : "Lisää vaihtoehtoinen viesti ajastimeen..."}
    oninput={() => scheduleUpdate()} bind:value={timerMessage}
  ></textarea>

  {#if isTimerRunning}
    <div
      class="disabled-overlay"
      role="button"
      tabindex="0"
      onclick={() => handleEditAttemptWhileRunning()}
      onkeydown={(e) => e.key === 'Enter' && handleEditAttemptWhileRunning()}
    ></div>
  {/if}
</div>

<style>
  .no-interaction {
    pointer-events: none;
  }

  .primary-input {
    color: #f6f6f6;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }

  .disabled-overlay {
    position: absolute;
    inset: 0;
    cursor: not-allowed;
    z-index: 1;
    background: transparent;
  }

  .timer-controls {
    justify-content: flex-start;
    width: 100%;
    height: 32px;
    gap: 4px;
    padding: 0 8px 8px;
    border-bottom: 1px solid #333;
  }

  .timer-controls button {
    min-height: 24px;
    max-height: 24px;
    min-width: 24px;
    max-width: 24px;
    border-radius: 4px;
  }
  .timer-controls button.disabled {
    box-shadow: none;
  }
  .timer-controls button:not(.disabled):hover {
    outline: 1px solid rgba(255, 70, 70, 1);
  }

  .timer-state {
    margin: 0 0 0 auto;
    font-weight: bold;
    font-size: clamp(0.75rem, 0.9cqw, 1rem);
  }

  .timer-content {
    position: relative;
    justify-content: flex-start;
    height: 100%;
    width: 100%;
    gap: 12px;
    padding: 0 18px 18px;
  }

  .timer-title {
    align-self: flex-end;
    min-width: 100px;
    max-height: 32px;
  }

  .timer-duration-title-container {
    justify-content: flex-start;
    width: 100%;
    gap: 12px;
  }

  .timer-duration-container, .timer-title-container {
    gap: 6px;
    padding: 16px;
    border-radius: 8px;
    border: 1px solid #333;
  }

  .timer-duration, .timer-title-container {
    gap: 6px;
    height: 100%;
  }

  .timer-duration-title-container .primary-input:not(.timer-title) {
    min-width: 2rem;
    max-width: 4rem;
    height: 2rem;
  }

  .timer-duration-title-container > *, .timer-duration .primary-input {
    font-weight: bold;
    text-align: center;
  }

  .timer-textarea {
    height: 100%;
    width: 100%;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    padding: 6px;
    resize: none;
    border: 1px solid #333;
    outline: none;
    border-radius: 8px;
    background-color: transparent;
    color: #f6f6f6;
  }
</style>