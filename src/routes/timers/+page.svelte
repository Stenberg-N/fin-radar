<script lang="ts">
  import { flip } from "svelte/animate";
  import { cubicInOut } from "svelte/easing";

  import { timers, createTimer, deleteTimer, checkTimerRuntimes, timerRuntimes, isAutoRun, toggleAutoRun } from "$lib/timers";
  import { user } from "$lib/user";
  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { handleHorizontalScroll } from "$lib/functions";
  import { handlePointerDown, handlePointerMove, handlePointerUp } from "$lib/dragAndDrop";

  import TimerComponent from "../../components/timers/Timer.svelte";
  import ToggleSwitch from "../../components/ToggleSwitch.svelte";

  const timersToolbarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: () => handleAddTimer() },
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => sendAlert("alert.delete-all-timers.confirmation", false, true, () => handleDeleteAllTimers()) },
  ];
  let dragIndex = $state<number | null>(null);
  const isSomeTimerRunning = $derived.by(() => checkTimerRuntimes($timerRuntimes));

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleAddTimer = async () => {
    if (!$user) return;
    const result = await createTimer($user.id);

    if (!result.success) {
      sendAlert("alert.add-timer.fail", true, false);
    }
  };

  const handleDeleteAllTimers = () => {
    if (!$user || !$timers.length) return;

    $timers.forEach(async (timer) => {
      const result = await deleteTimer($user.id, $user.name, timer.id);
      if (!result.success) sendAlert("alert.delete-timer.fail", true, false, undefined, undefined, timer.title, true);
    });
  };
  /***********************************************************************************************************************************/
</script>

<div id="timers-main-container" class="vertical-flex-container">
  <div id="timers-main-toolbar" class="primary-toolbar horizontal-flex-container">
    {#each timersToolbarButtons as button, i (i)}
      <button class="primary-button horizontal-flex-container" class:disabled={i === 1 && !$timers.length} disabled={i === 1 && !$timers.length} onclick={() => button.command()}>
        <img src={button.icon} alt={button.icon.slice(1, (button.icon.length - 4))} class="img-small" />
        {$t[button.titleKey]}
      </button>
    {/each}
    <div class="element-wrapper-for-title vertical-flex-container">
      <p class="element-paragraph-title">{$t["timers.toggle-autorun.description"]}</p>
      <ToggleSwitch
        activeDerivedFrom={$isAutoRun}
        onClickCommand={toggleAutoRun}
        translationKey={"timers.toggle-autorun.title"}
        height={25}
      />
    </div>
  </div>
  <div id="timers-main-content" class="vertical-flex-container">
    <div class="timers-list horizontal-flex-container">
      <div class="timers-wrapper horizontal-flex-container" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
        {#if !$timers.length}
          <p class="no-timers-paragraph"><img src="alarm-clock.svg" alt="Alarm clock" class="img-large" />{$t["timers.no-timers"]}</p>
        {:else}
          {#each $timers as timer, i (timer.id)}
            <div class="timer-container vertical-flex-container" style="position: relative;"
              animate:flip={{ duration: 200, easing: cubicInOut }}
              role="timer"
              class:hovered-over={dragIndex === i}
              data-index={i}
              onpointerup={() => {
                const res = handlePointerUp(timers, "timers", i, dragIndex);
                if (res) dragIndex = res.dragIndex;
              }}
            >
              <button class="drag-handle horizontal-flex-container"
                disabled={isSomeTimerRunning}
                class:disabled={isSomeTimerRunning}
                onpointerdown={(e) => { const res = handlePointerDown(e, i); if (res) dragIndex = res.dragIndex; }}
                onpointermove={(e) => { const res = handlePointerMove(e, dragIndex, "timers"); if (res) dragIndex = res.dragIndex; }}
              ><img src="/grip-dots.svg" alt="Drag handle" class="img-small" /></button>
              <TimerComponent {timer} />
            </div>
          {/each}
        {/if}
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
    padding: 20px;
  }

  #timers-main-toolbar button {
    gap: 8px;
  }

</style>