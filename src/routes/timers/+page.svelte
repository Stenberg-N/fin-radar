<script lang="ts">
  import { flip } from "svelte/animate";
  import { cubicInOut } from "svelte/easing";
  import { beforeNavigate, goto } from "$app/navigation";

  import { timers, createTimer, deleteTimer, checkTimerRuntimes, timerRuntimes, isAutoRun, toggleAutoRun, isTimerUpdateBatchOngoing } from "#lib/timers.ts";
  import { i18n } from "#lib/i18n/i18n.svelte.ts";
  import { sendAlert } from "#lib/classes/alertStore.svelte.ts";
  import { handleAutoScroll, handleHorizontalScroll } from "#lib/utils/actions.svelte.ts";
  import { handlePointerDown, handlePointerMove, handlePointerUp } from "#lib/utils/dragAndDrop.ts";

  import TimerComponent from "../../components/timers/Timer.svelte";
  import ToggleSwitch from "../../components/elements/ToggleSwitch.svelte";

  const timersToolbarButtons = [
    {
      get title() { return i18n.t["add.button"] as string; },
      icon: "/plus.svg",
      command: async () => { await createTimer(); },
    },
    {
      get title() { return i18n.t["delete.button"] as string; },
      icon: "/trash-can.svg",
      command: () => sendAlert({
        message: "alert.delete-all-timers.confirmation",
        isTimer: false,
        buttons: true,
        onConfirm: () => handleDeleteAllTimers()
      }),
    },
    {
      get title() { return i18n.t[`${areTimersExpanded ? "minimize" : "expand"}.button`] as string; },
      icon: "/expand.svg",
      command: () => areTimersExpanded = !areTimersExpanded,
    }
  ];

  let dragIndex = $state<number | null>(null);
  const isSomeTimerRunning = $derived(checkTimerRuntimes($timerRuntimes));
  let pendingNavigation = $state<string | null>(null);
  let areTimersExpanded = $state<boolean>(false);
  const autoScroll = $derived(!areTimersExpanded ? handleAutoScroll : () => {});
  const hScroll = $derived(!areTimersExpanded ? handleHorizontalScroll : () => {});

  beforeNavigate(({ to, cancel, shallow, type }) => {
    if (shallow && type === 'goto') return;
    if (!to || !$isTimerUpdateBatchOngoing) return;

    cancel();
    pendingNavigation = to.url.pathname;
    sendAlert({ message: "alert.unsaved-changes", isTimer: true, buttons: false });
  });

  $effect(() => {
    if (pendingNavigation !== null && !$isTimerUpdateBatchOngoing) {
      goto(pendingNavigation);
      pendingNavigation = null;
    }
  });

  const handleDeleteAllTimers = () => {
    if (!$timers.length) return;

    $timers.forEach(async (timer) => { await deleteTimer(timer); });
  };
</script>

<div id="timers-main-container" class="flex column">
  <div id="timers-main-toolbar" class="primary-toolbar flex row">
    {#each timersToolbarButtons as button, i (i)}
      <button class="button-primary transparent highlight outline default-corners" disabled={i === 1 && !$timers.length} onclick={() => button.command()}>
        <span class="span-icon img-small" style="mask-image: url('{button.icon}');"></span>
        {button.title}
      </button>
    {/each}
    <div class="element-wrapper-for-title flex column">
      <p class="element-paragraph-title">{i18n.t["timers.toggle-autorun.description"]}</p>
      <ToggleSwitch
        activeStateFrom={$isAutoRun}
        onClickCommand={toggleAutoRun}
        translationKey="timers.toggle-autorun.title"
      />
    </div>
  </div>
  <div id="timers-main-content" class="flex column">
    <div class="flex">
      <div class="timers-list flex row">
        <div
          class="timers-wrapper flex row"
          class:expanded={areTimersExpanded}
          use:hScroll={{ scrollMultiplier: 0.4 }}
          use:autoScroll
        >
          {#if !$timers.length}
            <p class="no-timers-paragraph">
              <span class="span-icon large-small" style="mask-image: url('/alarm-clock.svg');"></span>
              {i18n.t["timers.no-timers"]}
            </p>
          {:else}
            {#each $timers.sort((a, b) => a.order_id - b.order_id) as timer, i (timer.id)}
              <div class="timer-container flex column" style="position: relative;"
                animate:flip={{ duration: 200, easing: cubicInOut }}
                role="timer"
                class:hovered-over={dragIndex === i}
                data-index={i}
                onpointerup={() => {
                  const res = handlePointerUp({ array: timers, arrayType: "timers", idx: i, dragIndex });
                  if (res) dragIndex = res.dragIndex;
                }}
              >
                <button aria-label="Drag handle" class="drag-handle flex row"
                  disabled={isSomeTimerRunning}
                  class:disabled={isSomeTimerRunning}
                  onpointerdown={(e) => { const res = handlePointerDown(e, i); if (res) dragIndex = res.dragIndex; }}
                  onpointermove={(e) => { const res = handlePointerMove(e, dragIndex, "timers"); if (res) dragIndex = res.dragIndex; }}
                >
                  <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg');"></span>
                </button>
                <TimerComponent timer={timer} />
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  #timers-main-container, #timers-main-content {
    justify-content: flex-start;
    width: 100%;
    height: 100%;
  }

  #timers-main-content {
    padding: 4rem;
    overflow-y: auto;
    scrollbar-gutter: stable both-edges;
    --timer-w: calc((1360px - 4.5rem - 6rem) / 4);
    --timer-gap: 1.5rem;
    --chrome: 6rem;

    > div {
      max-width: 1360px;
      width: max(
        calc(var(--timer-w) + var(--chrome)),
        calc(round(down, 100% - var(--chrome) + var(--timer-gap), var(--timer-w) + var(--timer-gap)) + var(--chrome) - var(--timer-gap))
      );
      padding: 1rem;
      border-radius: 0.5rem;
      background-color: var(--color-secondary2);
    }

    .timers-wrapper.expanded {
      flex-wrap: wrap;
      overflow-x: hidden;
    }
  }

</style>
