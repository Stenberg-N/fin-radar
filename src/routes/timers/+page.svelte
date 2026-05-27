<script lang="ts">
  import { onMount } from "svelte";

  import { timers, createTimer, getTimers, updateTimer } from "$lib/timers";
  import { user } from "$lib/user";
  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import type { Timer } from "$lib/types";

  import TimerComponent from "../../components/timers/Timer.svelte";

  let timerUpdateBatch = $state<Timer[]>([]);

  const timersToolbarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: async () => handleAddTimer() },
  ];

  onMount(() => {
    (async () => {
      if (!$user) return;
      await getTimers($user.id, $user.name);
    })();
  });

  $effect(() => {
    const interval = setInterval(async () => {
      if (timerUpdateBatch.length === 0 || !$user) return;

      const batch = timerUpdateBatch.splice(0);
      const result = await updateTimer($user.id, $user.name, batch);
      if (!result.success) sendAlert("alert.update-timer.fail", true, false);
    }, 2000);

    return () => clearInterval(interval)
  });

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

  const handleTimerUpdate = (updatedTimer: Timer) => {
    const idx = timerUpdateBatch.findIndex(t => t.id === updatedTimer.id);
    if (idx !== -1) timerUpdateBatch[idx] = updatedTimer;
    else timerUpdateBatch.push(updatedTimer);
  };

  /***********************************************************************************************************************************/
</script>

<div id="timers-main-container" class="vertical-flex-container">
  <div id="timers-main-toolbar" class="primary-toolbar horizontal-flex-container">
    {#each timersToolbarButtons as button, i (i)}
      <button class="primary-button horizontal-flex-container" onclick={() => button.command()}><img src={button.icon} alt={button.icon.slice(1, 5)} class="img-small" />{$t[button.titleKey]}</button>
    {/each}
  </div>
  <div id="timers-list" class="horizontal-flex-container">
    {#each $timers as timer (timer.id)}
      <TimerComponent {timer} onUpdate={handleTimerUpdate}/>
    {/each}
  </div>
</div>

<style>
  #timers-main-container {
    justify-content: flex-start;
    width: 100%;
    height: 100%;
  }

  #timers-main-toolbar button {
    gap: 8px;
  }

  #timers-list {
    justify-content: flex-start;
    width: 100%;
    height: 25%;
    padding: 20px;
  }
</style>