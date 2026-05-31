<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount, setContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { beforeNavigate, goto, onNavigate } from "$app/navigation";
  import { page } from "$app/state";
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { emit } from "@tauri-apps/api/event";

  import { lang, t } from "$lib/i18n";
  import { logout, user } from "$lib/user";
  import { alerts, sendAlert } from "$lib/alert";
  import { setViewState, viewStore } from "$lib/viewStore";
  import { createTimer, getTimers, timers, startTimerBatchFlush } from "$lib/timers";
  import { handleHorizontalScroll } from "$lib/functions";

  import AuthScreen from "../components/auth-user/AuthScreen.svelte";
  import Alert from "../components/Alert.svelte";
  import SettingsBanner from "../components/SettingsBanner.svelte";
  import ChangePwModal from "../components/auth-user/ChangePwModal.svelte";
  import RecoveryScreen from "../components/auth-user/RecoveryScreen.svelte";
  import TimerComponent from "../components/timers/Timer.svelte";
  import "../styles.css";

  let { children } = $props();
  let isMenu = $derived($viewStore.isMenu);
  let isChangePwOverlay = $derived($viewStore.isChangePwOverlay);
  let isRecoveryView = $derived($viewStore.isRecoveryView);
  let isTimersMenu = $derived($viewStore.isTimersMenu);
  let areTimersLoaded = false;
  let unlisten: (() => void) | undefined;
  let windowInnerHeight = $state<number>(0);
  let windowInnerWidth = $state<number>(0);

  let menuToggleBtn = $state<HTMLButtonElement | null>(null);
  let alertsContainer = $state<HTMLDivElement | null>(null);
  let langToggleBtn = $state<HTMLButtonElement | null>(null);

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
    { path: "/transactions-table", img: "/coins.svg" },
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
    if ($user && !areTimersLoaded) {
      areTimersLoaded = true;
      (async () => await getTimers($user.id, $user.name))();
      startTimerBatchFlush($user.id, $user.name);
    }
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = () => [menuToggleBtn, alertsContainer, langToggleBtn];
  setContext('ignoredElements', getIgnoredElements);
  setContext('windowDimensions', { getWindowHeight: () => windowInnerHeight, getWindowWidth: () => windowInnerWidth });

  /***********************************************************************************************************************************/

  const cancelPwRecovery = async () => {
    if (!$user) return;

    try {
      await invoke('cancel_password_recovery', { id: $user.id, name: $user.name });
      logout();
      sendAlert("alert.password-recover.cancel.success", true, false);
    } catch (error) {
      sendAlert("alert.password-recover.cancel.fail", true, false);
    }
  };

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
  <button id="cancel-recovery-button" class="horizontal-flex-container primary-button" onclick={() => { sendAlert("alert.password.recover.cancel-confirmation-question", false, true, () => cancelPwRecovery()); }}><img src="/logout.svg" alt="Logout" class="img-medium" /><span>{$t["cancel.button"]}</span></button>
{:else}
  {#if isMenu}
    <SettingsBanner />
  {/if}

  {#if isChangePwOverlay}
    <ChangePwModal switchViewState={true} />
  {/if}

  {#if isTimersMenu}
    <div id="layout-timers-list" class="timers-list vertical-flex-container" transition:fly={{ x: (windowInnerWidth - 150) * 0.4, duration: 200, easing: cubicInOut}}>
      <div class="horizontal-flex-container" style="justify-content: flex-start; width: 100%; padding-bottom: 12px; border-bottom: 1px solid #333;">
        <h2 style="margin: 0; position: absolute; left: 50%; transform: translateX(-50%);">{$t["main.layout.view-title"][4]}</h2>
        <button class="primary-button horizontal-flex-container" style="gap: 8px;" onclick={() => createTimer($user.id)}>
          <img src="/plus.svg" alt="Plus" class="img-small" />
          {$t["add.button"]}
        </button>
      </div>
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
  {/if}

  <nav id="nav-bar">
    {#each navButtons as {path, img}, i (i)}
      <button class="transparent-button-highlight" class:current={page.url.pathname === path} onclick={() => { goto(path); }}><img src={img} alt="nav-icon" /><span>{$t["main.layout.view-title"][i]}</span></button>
    {/each}
  </nav>

  <div id="menu-bar" class="horizontal-flex-container">
    <h2 id="view-title">{$t["main.layout.view-title"][viewTitleIdx()]}</h2>
    <button class="primary-button" onclick={() => setViewState("isTimersMenu", undefined, true)} class:disabled={page.url.pathname === "/timers"} disabled={page.url.pathname === "/timers"}><img src="/alarm-clock.svg" alt="Clock" class="img-small" /></button>
    <button bind:this={langToggleBtn} title={$t["language.button.title"] as string} class="primary-button" style="font-weight: 600;" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
    <button bind:this={menuToggleBtn} title={$t["main.layout.button.menu-toggle"] as string} class="transparent-button-highlight" style="width: 32px; height: 32px;"
      class:disabled={isTimersMenu}
      disabled={isTimersMenu}
      onclick={() => isMenu = !isMenu}
    >
      <img style="width: 20px; height: 20px;" src="burger.svg" alt="Menu" />
    </button>
  </div>

  <div id="status-bar">
    <p>Status bar</p>
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

  #menu-bar button:last-child.disabled:hover {
    background-color: transparent;
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
    display: flex;
    align-items: center;
    height: 20px;
    padding: 4px 8px;
    border-top: 1px solid #333;
  }

  #status-bar p {
    margin: 0;
    text-align: center;
    line-height: 12px;
    font-size: 12px;
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

  @keyframes fade-out {
    to { opacity: 0; }
  }
  @keyframes fade-in {
    from { opacity: 0; }
  }

  :root::view-transition-old(container) {
    animation: fade-out 250ms both;
  }
  :root::view-transition-new(container) {
    animation: fade-in 250ms both;
  }
</style>