<script lang="ts">
  import { timers, createTimer, deleteTimer } from "$lib/timers";
  import { user } from "$lib/user";
  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { handleHorizontalScroll } from "$lib/functions";

  import TimerComponent from "../../components/timers/Timer.svelte";

  const timersToolbarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: () => handleAddTimer() },
    { titleKey: "delete.button", icon: "/trash-can.svg", command: () => sendAlert("alert.delete-all-timers.confirmation", false, true, () => handleDeleteAllTimers()) },
  ];

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
        <img src={button.icon} alt={button.icon.slice(1, 5)} class="img-small" />
        {$t[button.titleKey]}
      </button>
    {/each}
  </div>
  <div id="timers-main-content" class="vertical-flex-container">
    <div class="timers-list horizontal-flex-container">
      <div class="timers-wrapper horizontal-flex-container" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
        {#if !$timers.length}
          <p class="no-timers-paragraph"><img src="alarm-clock.svg" alt="Alarm clock" class="img-large" />{$t["timers.no-timers"]}</p>
        {:else}
          {#each $timers as timer (timer.id)}
            <div class="timer-container vertical-flex-container">
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