<script lang="ts">
  import { getContext } from "svelte";
  import { beforeNavigate, goto } from "$app/navigation";

  import { deleteTimer, timerRuntimes, queueTimerUpdate, startTimerCountdown, stopTimerCountdown, isTimerUpdateBatchOngoing } from "$lib/timers";
  import { user } from "$lib/user";
  import type { Timer } from "$lib/types";
  import { t, lang } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { handleClickOutside } from "$lib/actions";

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
  const timerDuration = $derived($timerRuntimes.get(timer.id)?.currentDuration ?? timer.duration);
  const isTimerRunning = $derived($timerRuntimes.get(timer.id)?.isRunning ?? false);
  const displayMinutes = $derived(Math.floor(timerDuration / 60));
  const displaySeconds = $derived(timerDuration % 60);

  let updateDebounce: number;
  let pendingNavigation = $state<string | null>(null);
  let selectedDurationEl = $state<{idx: number, inputEl: HTMLInputElement} | null>(null);
  let stepperButtonRefs = $state<HTMLButtonElement[]>([]);
  let isTimerTitleEmpty = $state<boolean>(false);

  beforeNavigate(({ to, cancel }) => {
    if (!to || !isTimerTitleEmpty) return;

    cancel();
    pendingNavigation = to.url.pathname;
  });

  $effect(() => {
    if (pendingNavigation !== null && !isTimerTitleEmpty) {
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
    if (timerTitle.trim() === '') return;

    isTimerUpdateBatchOngoing.update(() => true);
    clearTimeout(updateDebounce);
    updateDebounce = setTimeout(() => {
      queueTimerUpdate({ ...timer, duration: timerDuration, title: timerTitle, message: timerMessage });
    }, 400);
  };

  const handleTimerInput = (event: KeyboardEvent) => {
    const allowedKeys = ["Backspace", "ArrowLeft", "ArrowRight", "Control"];
    const regex = /^[0-9]+$/g;
    if (allowedKeys.includes(event.key)) return;
    if (event.ctrlKey && (event.key.toLowerCase() === 'z' || event.key.toLowerCase() === 'a')) return;
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
    sendAlert({ message: "alert.cannot-edit.timer-running", isTimer: true, buttons: false });
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
    const result = await deleteTimer(timer.id);
    if (!result.success) sendAlert({ message: "alert.delete-timer.fail", isTimer: true, buttons: false });
  };

  const updateTimerDuration = (newMinutes: number, newSeconds: number) => {
    const current = $timerRuntimes.get(timer.id)!;
    timerRuntimes.update((map) => map.set(timer.id, { ...current, currentDuration: newMinutes * 60 + newSeconds }));
    scheduleUpdate();
  };

  const checkTitle = (e: EventTarget) => {
    if (timerTitle.trim() === '') {
      isTimerTitleEmpty = true;
      (e as HTMLInputElement).focus();
      sendAlert({ message: "alert.timer.no-title", isTimer: true, buttons: false });
    } else {
      isTimerTitleEmpty = false; 
    }
  };
</script>

<div class="timer-controls horizontal-flex-container">
  <button class="transparent-button-highlight" onclick={(e) => { e.stopPropagation(); toggleTimer(); }}>
    <img src={isTimerRunning ? "/pause.svg" : "/play.svg"} alt={isTimerRunning ? "Pause" : "Play"} class="img-small" />
  </button>
  <button class="transparent-button-highlight horizontal-flex-container"
    onclick={() => sendAlert({
      message: "alert.delete-timer.confirmation",
      isTimer: false,
      buttons: true,
      onConfirm: () => handleTimerDelete(),
      additionalText: timer.title,
    })}
  >
    <img src="/trash-can.svg" alt="Trash can" class="img-small" />
  </button>
  {#each [{ command: () => handleTimerDurationStep(1) }, { command: () => handleTimerDurationStep(-1) }] as stepper, i (i)}
    <button bind:this={stepperButtonRefs[i]} class="transparent-button-highlight" disabled={!selectedDurationEl} onclick={() => stepper.command()} onmousedown={(e) => e.preventDefault()}>
      <img src="arrow.svg" alt="Arrow" class="img-small" style="transform: {i === 0 ? 'rotate(180deg)' : ''};" />
    </button>
  {/each}
  <p class="timer-state" style="color: {!isTimerRunning && timerDuration > 0 ? "#f6f6f6" : isTimerRunning ? "rgb(255, 70, 70)" : "rgb(170, 255, 170)"}; user-select: none;">
    {(!isTimerRunning && timerDuration > 0)
      ? $t["timers.state.paused"]
      : isTimerRunning
        ? $t["timers.state.running"]
        : $t["timers.state.finished"]}
  </p>
</div>

<div class="timer-content vertical-flex-container">
  {#if isTimerRunning}
    <div
      class="disabled-overlay"
      role="button"
      tabindex="0"
      onclick={() => handleEditAttemptWhileRunning()}
      onkeydown={(e) => e.key === 'Enter' && handleEditAttemptWhileRunning()}
    ></div>
  {/if}

  <div class="timer-duration-title-container horizontal-flex-container">
    <div class="timer-title-container vertical-flex-container">
      <p class="element-paragraph-title">{$lang === 'en' ? "Title" : "Otsikko"}</p>
      <input class="timer-title primary-input"
        class:no-interaction={isTimerRunning}
        oninput={() => scheduleUpdate()} bind:value={timerTitle}
        onblur={(e) => checkTitle(e.currentTarget)}
      />
    </div>
    <div class="timer-duration-container horizontal-flex-container">
      {#each [{ value: displayMinutes, unit: "MM" }, { value: displaySeconds, unit: "SS" }] as input, i (i)}
        <div class="timer-duration vertical-flex-container">
          <p class="element-paragraph-title">{input.unit}</p>
          <input type="number" min="0" class="primary-input"
            class:no-interaction={isTimerRunning}
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
    placeholder={$lang === 'en' ? "Add an optional timer message..." : "Lisää vaihtoehtoinen viesti ajastimeen..."}
    oninput={() => scheduleUpdate()} bind:value={timerMessage}
  ></textarea>
</div>

<style>
  .no-interaction {
    pointer-events: none;
  }

  .primary-input {
    color: #f6f6f6;
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

  .timer-state {
    margin: 0 0 0 auto;
    padding-right: 20px;
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