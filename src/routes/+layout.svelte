<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount, setContext } from "svelte";
  import { beforeNavigate, goto, onNavigate } from "$app/navigation";
  import { page } from "$app/state";
  import { fade, fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { cubicInOut } from "svelte/easing";
  import { emit } from "@tauri-apps/api/event";

  import { lang, t } from "$lib/i18n/i18n";
  import { logout, user, cancelRecoverPassword, updateSession } from "$lib/user";
  import { alerts, sendAlert } from "$lib/alert";
  import { setViewState, viewStore } from "$lib/viewStore";
  import { isNoteUpdateBatchOngoing } from "$lib/notes";
  import { createTimer, getTimers, timers, startTimerBatchFlush, isAutoRun, toggleAutoRun, checkTimerRuntimes, timerRuntimes, isTimerUpdateBatchOngoing } from "$lib/timers";
  import { handleHorizontalScroll, handleAutoScroll } from "$lib/actions";
  import { handlePointerDown, handlePointerMove, handlePointerUp } from "$lib/dragAndDrop";
  import { handleCursorPositionUpdate, viewport } from "$lib/viewport";
  import { ensureUserPrefsLoaded, updateUserPrefs, userPrefs } from "$lib/prefsStore";

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

  let areTimersLoaded = false;
  let unlistenAppClose: (() => void) | undefined;
  let unlistenSessionExpired: (() => void) | undefined;
  let unlistenSessionToExpire: (() => void) | undefined;
  let dragIndex = $state<number | null>(null);
  const isSomeTimerRunning = $derived(checkTimerRuntimes($timerRuntimes));

  let alertsContainer = $state<HTMLDivElement | null>(null);
  let timersCloseBtn = $state<HTMLButtonElement | null>(null);
  let navBarToggleBtn = $state<HTMLButtonElement | null>(null);
  let menuBarButtonRefs = $state<HTMLButtonElement[]>([]);

  const menuBarButtons = [
    { title: "main.layout.button.timers-toggle", getDisabled: () => page.url.pathname === "/timers", alt: "Alarms", getIcon: () => "/alarm-clock.svg", command: () => setViewState({ viewState: "isTimersMenu", toggle: true }) },
    { title: "language.button.title", getDisabled: () => null, getIcon: () => $lang === 'en' ? "EN" : "FI", alt: "Language", command: () => lang.set($lang === 'en' ? 'fi' : 'en') },
    { title: "main.layout.button.menu-toggle", getDisabled: () => $viewStore.isTimersMenu, getIcon: () => "/burger.svg", alt: "Burger", command: () => setViewState({ viewState: "isMenu", toggle: true }) },
  ];

  const viewTitleIdx = $derived(() => {
    switch(page.url.pathname) {
      case "/": return 0;
      case "/transactions-table": return 1;
      case "/calendar": return 2;
      case "/charts": return 3;
      case "/notes": return 4;
      case "/timers": return 5;
      default: return 0;
    }
  });
  const navButtons = [
    { path: "/", img: "/home.svg" },
    { path: "/transactions-table", img: "/credit-card.svg" },
    { path: "/calendar", img: "/calendar.svg" },
    { path: "/charts", img: "/stats.svg" },
    { path: "/notes", img: "/notes.svg" },
    { path: "/timers", img: "/alarm-clock.svg" },
  ];

  onMount(() => {
    (async () => {
      unlistenAppClose = await listen('app-closing', async () => {
        await logout();
        await emit('app-ready-to-close');
      });
      unlistenSessionToExpire = await listen('session-about-to-expire', () => {
        sendAlert({
          message: "alert.session.almost-expired",
          isTimer: false,
          buttons: true,
          onConfirm: () => updateSession(),
          onlyConfirmButton: true,
          confirmButtonI18nKey: "extend.button",
          placeTextOnNewRow: true
        });
      });
      unlistenSessionExpired = await listen('session-expired', async () => {
        await logout();
        sendAlert({ message: "alert.session.expired", isTimer: false, buttons: false });
      });

      if ($user) await ensureUserPrefsLoaded();
    })();
    window.addEventListener('mousemove', handleCursorPositionUpdate, { passive: true });
    return () => { window.removeEventListener('mousemove', handleCursorPositionUpdate); };
  });

  onDestroy(() => {
    unlistenAppClose?.();
    unlistenSessionToExpire?.();
    unlistenSessionExpired?.();
  });

  beforeNavigate(({ to }) => {
    if (to?.url.pathname === "/timers") setViewState({ viewState: "isTimersMenu", state: false });
  });

  onNavigate(({  }) => {
    return new Promise((resolve) => {
      document.startViewTransition(() => {
        resolve();
      });
    });
  });

  $effect(() => {
    if ($user && !$user.requires_password_reset && ($timers.length === 0 && !areTimersLoaded)) {
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
  const getIgnoredElements = () => [alertsContainer, timersCloseBtn, navBarToggleBtn].concat(menuBarButtonRefs);
  setContext('ignoredElements', getIgnoredElements);

  /***********************************************************************************************************************************/

</script>

<svelte:window bind:innerHeight={$viewport.height} bind:innerWidth={$viewport.width} />

<div bind:this={alertsContainer} class="alerts-container vertical-flex-container">
  {#each $alerts as alert (alert.id)}
    <div>
      <Alert {alert} />
    </div>
  {/each}
</div>

{#if !$user}
  <AuthScreen />
  {#if $viewStore.isRecoveryView}
    <RecoveryScreen />
  {/if}
{:else if $user.requires_password_reset}
  <ChangePwModal isRecovery={true} />
  <button id="cancel-recovery-button" class="primary-button" transition:fly={{ y: -40, duration: 600, easing: cubicInOut }}
    onclick={() => { sendAlert({ message: "alert.password.recover.cancel-confirmation-question", isTimer: false, buttons: true, onConfirm: () => cancelRecoverPassword() }); }}
  >
    <span class="span-icon" style="mask-image: url('');"></span>
    <img src="/logout.svg" alt="Logout" class="img-medium" />
    <span>{$t["cancel.button"]}</span>
  </button>
{:else}
  {#if $viewStore.isMenu && !$viewStore.isTimersMenu}
    <SettingsBanner />
  {/if}

  {#if $viewStore.isChangePwOverlay}
    <ChangePwModal switchViewState={true} />
  {/if}

  {#if $viewStore.isAskPassword}
    <AskPassword />
  {/if}

  {#if $viewStore.isTimersMenu}
    <div id="layout-timers-list" class="timers-list vertical-flex-container" use:handleAutoScroll={{ querySelector: "timers-wrapper" }} transition:fly={{ x: $viewport.height * 0.4, duration: 200, easing: cubicInOut}}>
      <div id="layout-timers-list-topbar" class="horizontal-flex-container">
        <button class="primary-button" style="gap: 8px;" onclick={() => createTimer()}>
          <span class="span-icon img-small" style="mask-image: url('/plus.svg');"></span>
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
        <button aria-label="Close timers" bind:this={timersCloseBtn} id="close-button" class="transparent-button-highlight" style="position: absolute; right: 20px; width: 32px; height: 32px;"
          onclick={() => setViewState({ viewState: "isTimersMenu", state: false })}
        >
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
        </button>
      </div>
      <div class="timers-wrapper horizontal-flex-container" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
        {#if !$timers.length}
          <p class="no-timers-paragraph">
            <span class="span-icon img-large" style="mask-image: url('/alarm-clock.svg');"></span>
            {$t["timers.no-timers"]}
          </p>
        {:else}
          {#each $timers as timer, i (timer.id)}
            <div class="timer-container vertical-flex-container" style="position: relative;"
              animate:flip={{ duration: 200, easing: cubicInOut }}
              role="timer"
              class:hovered-over={dragIndex === i}
              data-index={i}
              onpointerup={() => { const res = handlePointerUp(timers, "timers", i, dragIndex); if (res) dragIndex = res.dragIndex; }}
            >
              <button aria-label="Drag handle" class="drag-handle horizontal-flex-container"
                disabled={isSomeTimerRunning}
                onpointerdown={(e) => { const res = handlePointerDown(e, i); if (res) dragIndex = res.dragIndex; }}
                onpointermove={(e) => { const res = handlePointerMove(e, dragIndex, "timers"); if (res) dragIndex = res.dragIndex; }}
              >
                <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg');"></span>
              </button>
              <TimerComponent {timer} />
            </div>
          {/each}
        {/if}
      </div>
    </div>
  {/if}

  <main id="container" style="view-transition-name: container;">
    <div id="layout-grid" style="grid-template-columns: {$userPrefs.mainPrefs.isNavBarCollapsed ? "44px" : "150px"} 1fr;">
      <nav id="nav-bar">
        {#each navButtons as {path, img}, i (i)}
          <button class="transparent-button-highlight" class:current={page.url.pathname === path} onclick={() => { goto(path); }}>
            <span class="span-icon" style="mask-image: url('{img}');"></span>
            {#if !$userPrefs.mainPrefs.isNavBarCollapsed}
              <span in:fade={{ duration: 200, easing: cubicInOut }}>{$t["main.layout.view-title"][i]}</span>
            {/if}
          </button>
        {/each}
        <button aria-label="Toggle navigation bar" class="transparent-button-highlight" onclick={() => updateUserPrefs("mainPrefs", "isNavBarCollapsed", !$userPrefs["mainPrefs"].isNavBarCollapsed)} bind:this={navBarToggleBtn}>
          <span class="span-icon img-small" style="mask-image: url('/arrow.svg'); transition: transform 0.2s; transform: rotate({$userPrefs.mainPrefs.isNavBarCollapsed ? "-90deg" : "90deg"});"></span>
        </button>
      </nav>

      <div id="main-area">
        <div id="menu-bar" class="horizontal-flex-container">
          <h2 id="view-title">{$t["main.layout.view-title"][viewTitleIdx()]}</h2>
          {#each menuBarButtons as button, i (i)}
            <button bind:this={menuBarButtonRefs[i]}
              title={$t[button.title] as string}
              class={i === 2 ? "transparent-button-highlight" : "primary-button"}
              disabled={button.getDisabled()}
              onclick={() => button.command()}
              style={i === 2 ? "width: 32px; height: 32px;" : i === 1 ? "font-weight: 600" : ""}
            >
              {#if i === 1}
                {button.getIcon()}
              {:else}
                <span class="span-icon img-small" style="mask-image: url('{button.getIcon()}');"></span>
              {/if}
            </button>
          {/each}
        </div>

        <div id="content">
          {@render children()}
        </div>
      </div>
    </div>

    <div id="status-bar" class="horizontal-flex-container">
      {#if (page.url.pathname === "/notes" || page.url.pathname === "/timers")}
        <p class:opacity-breathing={$isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing} style="color: {($isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing) ? 'rgb(255, 70, 70)' : '#f6f6f6'};">
            {($isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing) ? $t["saving.saving-in-progress"] : $t["saving.up-to-date"]}
        </p>
      {:else}
        <p></p>
      {/if}
    </div>
  </main>
{/if}

<style>
  .current {
    background-color: rgba(200, 200, 200, 0.2);
  }

  #container {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
  }

  #layout-grid {
    position: relative;
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 44px 1fr;
    transition: grid-template-columns 0.2s;
    contain: layout style;
    will-change: grid-template-columns;
  }

  #main-area {
    position: relative;
    min-width: 0;
  }

  #menu-bar {
    position: absolute;
    inset: 0 0 auto 0;
    justify-content: flex-end;
    height: 50px;
    gap: 12px;
    padding: 8px;
    border-bottom: 1px solid #333;

    button:nth-of-type(-n+2) {
      width: 36px;
      height: 32px;
    }
  }

  #content {
    position: absolute;
    inset: 50px 0 0 0;
  }

  #view-title {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    margin: 0;
  }

  #nav-bar {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    padding: 4px;
    gap: 4px;
    border-right: 1px solid #333;
    user-select: none;
    contain: layout style;
    will-change: width;

    button {
      justify-content: flex-start;
      height: 36px;
      width: 100%;
      padding: 2px 8px;
      gap: 8px;
      border-radius: 4px;
      color: #f6f6f6;
    }

    button:first-of-type {
      margin-top: 0;
    }

    button:last-of-type {
      margin-top: auto;
      justify-content: center;
      max-width: 35px;
      border-radius: 50%;
    }

    button span {
      display: flex;
      align-items: center;
      color: #f6f6f6;
      font-weight: bold;
    }

    button:not(:last-of-type) span {
      width: 20px;
      height: 20px;
    }
  }

  #status-bar {
    flex: 0 0 20px;
    height: 20px;
    padding: 2px 8px;
    border-top: 1px solid #333;
    user-select: none;

    p {
      margin: 0;
      text-align: center;
      line-height: 12px;
      font-size: 12px;
      font-weight: bold;
    }
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

    > * {
      pointer-events: auto;
    }
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

    span {
      display: flex;
      align-items: center;
      height: 20px;
      font-size: 15px;
      color: #f6f6f6;
      font-weight: bold;
    }
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

    .timer-container {
      max-width: calc((100% - 40px) / 3);
    }
  }

  #layout-timers-list-topbar {
    justify-content: flex-start;
    width: 100%;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 2px solid #333;
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