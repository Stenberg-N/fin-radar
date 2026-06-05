<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount, setContext } from "svelte";
  import { beforeNavigate, goto, onNavigate } from "$app/navigation";
  import { page } from "$app/state";
  import { fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { cubicInOut } from "svelte/easing";
  import { emit } from "@tauri-apps/api/event";

  import { lang, t } from "$lib/i18n";
  import { logout, user, cancelRecoverPassword } from "$lib/user";
  import { alerts, sendAlert } from "$lib/alert";
  import { setViewState, viewStore } from "$lib/viewStore";
  import { isNoteUpdateBatchOngoing } from "$lib/notes";
  import { createTimer, getTimers, timers, startTimerBatchFlush, isAutoRun, toggleAutoRun, checkTimerRuntimes, timerRuntimes, isTimerUpdateBatchOngoing } from "$lib/timers";
  import { handleHorizontalScroll } from "$lib/functions";
  import { handlePointerDown, handlePointerMove, handlePointerUp } from "$lib/dragAndDrop";

  import "../styles.css";
  import AuthScreen from "../components/auth-user/AuthScreen.svelte";
  import Alert from "../components/Alert.svelte";
  import SettingsBanner from "../components/SettingsBanner.svelte";
  import ChangePwModal from "../components/auth-user/ChangePwModal.svelte";
  import RecoveryScreen from "../components/auth-user/RecoveryScreen.svelte";
  import TimerComponent from "../components/timers/Timer.svelte";
  import ToggleSwitch from "../components/ToggleSwitch.svelte";
  import AskPassword from "../components/auth-user/AskPassword.svelte";

  let { children } = $props();
  const isMenu = $derived($viewStore.isMenu);
  const isChangePwOverlay = $derived($viewStore.isChangePwOverlay);
  const isRecoveryView = $derived($viewStore.isRecoveryView);
  const isTimersMenu = $derived($viewStore.isTimersMenu);
  const isAskPasswordModal = $derived($viewStore.isAskPassword);
  let areTimersLoaded = false;
  let unlisten: (() => void) | undefined;
  let windowInnerHeight = $state<number>(0);
  let windowInnerWidth = $state<number>(0);
  let dragIndex = $state<number | null>(null);
  const isSomeTimerRunning = $derived.by(() => checkTimerRuntimes($timerRuntimes));

  let alertsContainer = $state<HTMLDivElement | null>(null);
  let timersCloseBtn = $state<HTMLButtonElement | null>(null);
  let menuBarButtonRefs = $state<HTMLButtonElement[]>([]);

  const menuBarButtons = [
    { title: "main.layout.button.timers-toggle", getDisabled: () => page.url.pathname === "/timers", alt: "Alarms", getIcon: () => "/alarm-clock.svg", command: () => setViewState("isTimersMenu", undefined, true) },
    { title: "language.button.title", getDisabled: () => null, getIcon: () => $lang === 'en' ? "EN" : "FI", alt: "Language", command: () => lang.set($lang === 'en' ? 'fi' : 'en') },
    { title: "main.layout.button.menu-toggle", getDisabled: () => isTimersMenu, getIcon: () => "/burger.svg", alt: "Burger", command: () => setViewState("isMenu", undefined, true) },
  ];

  const viewTitleIdx = $derived(() => {
    switch(page.url.pathname) {
      case "/": return 0;
      case "/transactions-table": return 1;
      case "/charts": return 2;
      case "/notes": return 3;
      case "/timers": return 4;
      default: return -1;
    }
  });
  const navButtons = [
    { path: "/", img: "/home.svg" },
    { path: "/transactions-table", img: "/credit-card.svg" },
    { path: "/charts", img: "/stats.svg" },
    { path: "/notes", img: "/notes.svg" },
    { path: "/timers", img: "/alarm-clock.svg" },
  ];

  onMount(() => {
    (async () => {
      unlisten = await listen('app-closing', async () => {
        await logout();
        await emit('app-ready-to-close');
      });
    })();
  });

  onDestroy(() => {
    unlisten?.();
  });

  $effect(() => {
    if ($user && ($timers.length === 0 && !areTimersLoaded)) {
      areTimersLoaded = true;
      (async () => await getTimers())();
      startTimerBatchFlush();
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = () => [alertsContainer, timersCloseBtn].concat(menuBarButtonRefs);
  setContext('ignoredElements', getIgnoredElements);
  setContext('windowDimensions', { getWindowHeight: () => windowInnerHeight, getWindowWidth: () => windowInnerWidth });

  /***********************************************************************************************************************************/

  beforeNavigate(({ to }) => {
    if (to?.url.pathname === "/timers") setViewState("isTimersMenu", false);
  });

  onNavigate(({ from, to }) => {
    return new Promise((resolve) => {
      document.startViewTransition(() => {
        resolve();
      });
    });
  });

</script>

<svelte:window bind:innerHeight={windowInnerHeight} bind:innerWidth={windowInnerWidth} />

<div bind:this={alertsContainer} class="alerts-container vertical-flex-container">
  {#each $alerts as alert (alert.id)}
    <div>
      <Alert {alert} />
    </div>
  {/each}
</div>

{#if !$user}
  <AuthScreen />
  {#if isRecoveryView}
    <RecoveryScreen />
  {/if}
{:else if $user.requires_password_reset}
  <ChangePwModal isRecovery={true} />
  <button id="cancel-recovery-button" class="horizontal-flex-container primary-button" transition:fly={{ y: -40, duration: 600, easing: cubicInOut }}
    onclick={() => { sendAlert("alert.password.recover.cancel-confirmation-question", false, true, () => cancelRecoverPassword()); }}
  >
    <img src="/logout.svg" alt="Logout" class="img-medium" />
    <span>{$t["cancel.button"]}</span>
  </button>
{:else}
  {#if isMenu}
    <SettingsBanner />
  {/if}

  {#if isChangePwOverlay}
    <ChangePwModal switchViewState={true} />
  {/if}

  {#if isAskPasswordModal}
    <AskPassword />
  {/if}

  {#if isTimersMenu}
    <div id="layout-timers-list" class="timers-list vertical-flex-container" transition:fly={{ x: windowInnerWidth * 0.4, duration: 200, easing: cubicInOut}}>
      <div id="layout-timers-list-topbar" class="horizontal-flex-container">
        <h2 style="margin: 0; position: absolute; left: 50%; transform: translateX(-50%);">{$t["main.layout.view-title"][4]}</h2>
        <button class="primary-button horizontal-flex-container" style="gap: 8px;" onclick={() => createTimer()}>
          <img src="/plus.svg" alt="Plus" class="img-small" />
          {$t["add.button"]}
        </button>
        <div class="element-wrapper-for-title vertical-flex-container">
          <p class="element-paragraph-title">{$t["timers.toggle-autorun.description"]}</p>
          <ToggleSwitch
            activeDerivedFrom={$isAutoRun}
            onClickCommand={toggleAutoRun}
            translationKey={"timers.toggle-autorun.title"}
            height={25}
          />
        </div>
        <button bind:this={timersCloseBtn} id="close-button" class="transparent-button-highlight" style="position: absolute; right: 20px; width: 32px; height: 32px;" onclick={() => setViewState("isTimersMenu", false)}>
          <img src="close-x.svg" alt="Close" class="img-small" />
        </button>
      </div>
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
              onpointerup={() => { const res = handlePointerUp(timers, "timers", i, dragIndex); if (res) dragIndex = res.dragIndex; }}
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
  {/if}

  <nav id="nav-bar">
    {#each navButtons as {path, img}, i (i)}
      <button class="transparent-button-highlight" class:current={page.url.pathname === path} onclick={() => { goto(path); }}><img src={img} alt="nav-icon" /><span>{$t["main.layout.view-title"][i]}</span></button>
    {/each}
  </nav>

  <div id="menu-bar" class="horizontal-flex-container">
    <h2 id="view-title">{$t["main.layout.view-title"][viewTitleIdx()]}</h2>
    {#each menuBarButtons as button, i (i)}
      <button bind:this={menuBarButtonRefs[i]}
        title={$t[button.title] as string}
        class={i === 2 ? "transparent-button-highlight" : "primary-button"}
        class:disabled={button.getDisabled()}
        disabled={button.getDisabled()} onclick={() => button.command()}
        style={i === 2 ? "width: 32px; height: 32px;" : i === 1 ? "font-weight: 600" : ""}
      >
        {#if i === 1}
          {button.getIcon()}
        {:else}
          <img src={button.getIcon()} alt={button.alt} class="img-small" />
        {/if}
      </button>
    {/each}
  </div>

  <div id="status-bar" class="horizontal-flex-container">
    <p class:opacity-breathing={$isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing} style="color: {($isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing) ? 'rgb(255, 70, 70)' : '#f6f6f6'};">
      {#if (page.url.pathname === "/notes" || page.url.pathname === "/timers")}
        {($isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing) ? $t["saving.saving-in-progress"] : $t["saving.up-to-date"]}
      {/if}
    </p>
  </div>

  <main id="container" class="vertical-flex-container" style="view-transition-name: container;">
    {@render children()}
  </main>
{/if}

<style>
  #container {
    position: fixed;
    top: 50px;
    left: 150px;
    right: 0;
    bottom: 20px;
    margin: 0;
  }

  #menu-bar {
    position: fixed;
    left: 150px;
    right: 0;
    top: 0;
    justify-content: flex-end;
    height: 50px;
    gap: 12px;
    padding: 8px;
    border-bottom: 1px solid #333;
  }

  #menu-bar button:nth-of-type(-n+2) {
    width: 36px;
    height: 32px;
  }

  #view-title {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    margin: 0;
  }

  #nav-bar {
    position: fixed;
    left: 0;
    bottom: 20px;
    top: 0;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    width: 150px;
    padding: 4px;
    gap: 4px;
    border-right: 1px solid #333;
    user-select: none;
  }

  #nav-bar button {
    height: 36px;
    justify-content: flex-start;
    padding: 2px 8px;
    gap: 8px;
    border-radius: 6px;
    color: #f6f6f6;
  }

  #nav-bar button:first-child {
    margin-top: 0;
  }

  #nav-bar button:hover {
    background-color: #222;
  }

  #nav-bar button span {
    display: flex;
    align-items: center;
    height: 20px;
    font-size: 14px;
    color: #f6f6f6;
    font-weight: bold;
  }

  #nav-bar button img {
    width: 20px;
    height: 20px;
  }

  #status-bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    height: 20px;
    padding: 2px 8px;
    border-top: 1px solid #333;
    user-select: none;
  }

  #status-bar p {
    margin: 0;
    text-align: center;
    line-height: 12px;
    font-size: 12px;
    font-weight: bold;
  }

  .alerts-container {
    position: fixed;
    z-index: 1000;
    bottom: 30px;
    left: 50%;
    justify-content: unset;
    transform: translateX(-50%);
    gap: 12px;
    pointer-events: none;
  }

  .alerts-container > * {
    pointer-events: auto;
  }

  #cancel-recovery-button {
    position: fixed;
    z-index: 500;
    top: 30px;
    width: 300px;
    height: 48px;
    justify-self: center;
    justify-content: flex-start;
    gap: 8px;
    padding: 2px 8px;
  }

  #cancel-recovery-button span {
    display: flex;
    align-items: center;
    height: 20px;
    font-size: 15px;
    color: #f6f6f6;
    font-weight: bold;
  }

  #layout-timers-list {
    position: fixed;
    z-index: 1000;
    top: 45px;
    right: 10px;
    max-width: 40%;
    border-radius: 8px;
    outline: 1px solid #333;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }

  #layout-timers-list-topbar {
    justify-content: flex-start;
    width: 100%;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid #333;
  }

  #layout-timers-list .timer-container {
    max-width: calc((100% - 40px) / 3);
  }

  .current {
    background-color: #222;
  }

  :root::view-transition-old(container), :root::view-transition-new(container) {
    animation-duration: 0.5s;
    animation-timing-function: cubic-bezier(0.645, 0.045, 0.355, 1);
  }

  :root::view-transition-old(container) {
    animation: fade-out 200ms both;
  }
  :root::view-transition-new(container) {
    animation: fade-in 200ms both;
  }
</style>