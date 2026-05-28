<script lang="ts">
  import { timers, createTimer } from "$lib/timers";
  import { user } from "$lib/user";
  import { t } from "$lib/i18n";
  import { sendAlert } from "$lib/alert";
  import { handleHorizontalScroll } from "$lib/functions";

  import TimerComponent from "../../components/timers/Timer.svelte";

  const timersToolbarButtons = [
    { titleKey: "add.button", icon: "/plus.svg", command: async () => handleAddTimer() },
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

  /***********************************************************************************************************************************/
</script>

<div id="timers-main-container" class="vertical-flex-container">
  <div id="timers-main-toolbar" class="primary-toolbar horizontal-flex-container">
    {#each timersToolbarButtons as button, i (i)}
      <button class="primary-button horizontal-flex-container" onclick={() => button.command()}><img src={button.icon} alt={button.icon.slice(1, 5)} class="img-small" />{$t[button.titleKey]}</button>
    {/each}
  </div>
  <div id="timers-main-content" class="vertical-flex-container">
    <div class="timers-list horizontal-flex-container">
      <div class="timers-wrapper horizontal-flex-container" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
        {#each $timers as timer (timer.id)}
          <TimerComponent {timer} />
        {/each}
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