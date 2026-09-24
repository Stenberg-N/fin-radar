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
  import { initTransactionsFeed } from "$lib/transactions";

  import "../styles.css";
  import AuthScreen from "../components/auth-user/AuthScreen.svelte";
  import Alert from "../components/Alert.svelte";
  import SettingsBanner from "../components/SettingsBanner.svelte";
  import ChangePwModal from "../components/auth-user/ChangePwModal.svelte";
  import RecoveryScreen from "../components/auth-user/RecoveryScreen.svelte";
  import TimerComponent from "../components/timers/Timer.svelte";
  import ToggleSwitch from "../components/ToggleSwitch.svelte";
  import AskPassword from "../components/auth-user/AskPassword.svelte";
  import SettingsOverlay from "../components/settings-overlay/SettingsOverlay.svelte";

  let { children } = $props();

  let areTimersLoaded = false;
  let arePrefsLoaded = false;
  let isTransactionsFeedLoaded = false;
  let unlistenAppClose: (() => void) | undefined;
  let unlistenSessionExpired: (() => void) | undefined;
  let unlistenSessionToExpire: (() => void) | undefined;
  let unlistenSessionCleared: (() => void) | undefined;
  let dragIndex = $state<number | null>(null);
  const isSomeTimerRunning = $derived(checkTimerRuntimes($timerRuntimes));

  let alertsContainer = $state<HTMLDivElement | null>(null);
  let timersCloseBtn = $state<HTMLButtonElement | null>(null);
  let navBarToggleBtn = $state<HTMLButtonElement | null>(null);
  let menuBarButtonRefs = $state<HTMLButtonElement[]>([]);

  const menuBarButtons = [
    {
      get title() { return $t["main.layout.button.timers-toggle"]; },
      get disabled() { return page.url.pathname === "/timers"; },
      icon: "/alarm-clock.svg",
      command: () => { setViewState({ viewState: "isTimersMenu", toggle: true }); setViewState({ viewState: "isMenu", state: false }); },
      get toggled() { return $viewStore["isTimersMenu"] ? true : false; },
    },
    {
      get title() { return $t["language.button.title"]; },
      disabled: null,
      get icon() { return $lang === 'en' ? "EN" : "FI"; },
      command: () => lang.set($lang === 'en' ? 'fi' : 'en'),
      toggled: null,
    },
    {
      get title() { return $t["main.layout.button.menu-toggle"]; },
      get disabled() { return $viewStore.isTimersMenu; },
      icon: "/burger.svg",
      command: () => setViewState({ viewState: "isMenu", toggle: true }),
      get toggled() { return $viewStore["isMenu"] ? true : false; },
    },
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
      unlistenSessionCleared = await listen('session-cleared', async () => {
        await logout();
      });
    })();
    window.addEventListener('mousemove', handleCursorPositionUpdate, { passive: true });
    return () => { window.removeEventListener('mousemove', handleCursorPositionUpdate); };
  });

  onDestroy(() => {
    unlistenAppClose?.();
    unlistenSessionToExpire?.();
    unlistenSessionExpired?.();
    unlistenSessionCleared?.();
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

  $effect(() => {
    if ($user && !arePrefsLoaded) {
      arePrefsLoaded = true;
      (async () => await ensureUserPrefsLoaded())();
    }
  });

  $effect(() => {
    if ($user && !isTransactionsFeedLoaded) {
      isTransactionsFeedLoaded = true;
      (async () => await initTransactionsFeed())();
    }
  });

  $effect(() => {
    if ($user && $userPrefs.mainPrefs.lang !== null) {
      const lang = $userPrefs.mainPrefs.lang;
      document.documentElement.lang = lang;
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

<div bind:this={alertsContainer} class="alerts-container flex column">
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
  <div class="flex column" style="position: fixed; z-index: 1000; inset: 0; background-color: var(--color-primary2);" transition:fade={{ duration: 200, easing: cubicInOut }}>
    <ChangePwModal options={{ isRecovery: true, theme: "light", enableTransitions: true }} />
  </div>
  <button id="cancel-recovery-button" class="button-primary" transition:fly={{ y: -40, duration: 600, easing: cubicInOut }}
    onclick={() => { sendAlert({ message: "alert.password.recover.cancel-confirmation-question", isTimer: false, buttons: true, onConfirm: () => cancelRecoverPassword() }); }}
  >
    <span class="span-icon img-medium" style="mask-image: url('/logout.svg');"></span>
    {$t["cancel.button"]}
  </button>
{:else}
  {#if $viewStore.isMenu && !$viewStore.isTimersMenu}
    <SettingsBanner />
  {/if}

  {#if $viewStore.isAskPassword}
    <AskPassword />
  {/if}

  {#if $viewStore.isSettingsOverlay}
    <SettingsOverlay />
  {/if}

  {#if $viewStore.isTimersMenu}
    <div id="layout-timers-list" class="timers-list flex column" use:handleAutoScroll={{ querySelector: "timers-wrapper" }} transition:fly={{ x: $viewport.height * 0.4, duration: 200, easing: cubicInOut}}>
      <div id="layout-timers-list-topbar" class="flex row">
        <button class="button-primary" onclick={() => createTimer()}>
          <span class="span-icon img-small" style="mask-image: url('/plus.svg');"></span>
          {$t["add.button"]}
        </button>
        <div class="element-wrapper-for-title flex column">
          <p class="element-paragraph-title">{$t["timers.toggle-autorun.description"]}</p>
          <ToggleSwitch
            activeDerivedFrom={$isAutoRun}
            onClickCommand={toggleAutoRun}
            translationKey={"timers.toggle-autorun.title"}
            height={25}
          />
        </div>
        <button aria-label="Close timers" bind:this={timersCloseBtn} id="close-button" class="button-primary transparent highlight static" style="position: absolute; right: 20px;"
          onclick={() => setViewState({ viewState: "isTimersMenu", state: false })}
        >
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
        </button>
      </div>
      <div class="timers-wrapper flex row" use:handleHorizontalScroll={{ scrollMultiplier: 0.4 }}>
        {#if !$timers.length}
          <p class="no-timers-paragraph">
            <span class="span-icon img-large" style="mask-image: url('/alarm-clock.svg');"></span>
            {$t["timers.no-timers"]}
          </p>
        {:else}
          {#each $timers as timer, i (timer.id)}
            <div class="timer-container flex column" style="position: relative;"
              animate:flip={{ duration: 200, easing: cubicInOut }}
              role="timer"
              class:hovered-over={dragIndex === i}
              data-index={i}
              onpointerup={() => { const res = handlePointerUp(timers, "timers", i, dragIndex); if (res) dragIndex = res.dragIndex; }}
            >
              <button aria-label="Drag handle" class="drag-handle flex row"
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
          <button class="button-primary transparent highlight" class:current={page.url.pathname === path} onclick={() => { goto(path); }}>
            <span class="span-icon img-small-medium" style="mask-image: url('{img}');"></span>
            {#if !$userPrefs.mainPrefs.isNavBarCollapsed}
              <span in:fade={{ duration: 200, easing: cubicInOut }}>
                {$t["main.layout.view-title"][i]}
              </span>
            {/if}
          </button>
        {/each}
        <button aria-label="Toggle navigation bar" class="button-primary transparent highlight" onclick={() => updateUserPrefs("mainPrefs", "isNavBarCollapsed", !$userPrefs["mainPrefs"].isNavBarCollapsed)} bind:this={navBarToggleBtn}>
          <span class="span-icon img-small" style="mask-image: url('/arrow.svg'); transition: transform 0.2s; transform: rotate({$userPrefs.mainPrefs.isNavBarCollapsed ? "-90deg" : "90deg"});"></span>
        </button>
      </nav>

      <div id="main-area">
        <div id="menu-bar" class="flex row">
          <h2 id="view-title">{$t["main.layout.view-title"][viewTitleIdx()]}</h2>
          {#each menuBarButtons as button, i (i)}
            <button bind:this={menuBarButtonRefs[i]}
              title={button.title as string}
              class="button-primary transparent highlight { [0, 1].includes(i) && 'outline'}"
              class:toggled={button.toggled}
              disabled={button.disabled}
              onclick={button.command}
              style={i === 1 ? "font-weight: bold" : ""}
            >
              {#if i === 1}
                {button.icon}
              {:else}
                <span class="span-icon img-small" style="mask-image: url('{button.icon}');"></span>
              {/if}
            </button>
          {/each}
        </div>

        <div id="content">
          {@render children()}
        </div>

        <div id="status-bar" class="flex row">
          {#if (page.url.pathname === "/notes" || page.url.pathname === "/timers")}
            <p class:opacity-breathing={$isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing} style="color: {($isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing) ? 'rgb(255, 70, 70)' : '#f6f6f6'};">
                {($isNoteUpdateBatchOngoing || $isTimerUpdateBatchOngoing) ? $t["saving.saving-in-progress"] : $t["saving.up-to-date"]}
            </p>
          {:else}
            <p></p>
          {/if}
        </div>
      </div>
    </div>
  </main>
{/if}

<style>
  .current {
    background-color: var(--color-highlight2);
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
    padding: 0.5rem;
    gap: 0.5rem;
    transition: grid-template-columns 0.2s;
    contain: layout style;
    will-change: grid-template-columns;
  }

  #main-area {
    position: relative;
    min-width: 0;
    border-radius: 0.5rem;
    background-color: var(--color-primary2);
  }

  #menu-bar {
    position: absolute;
    inset: 0 0 auto 0;
    justify-content: flex-end;
    height: 50px;
    gap: 0.75rem;
    padding: 0.5rem;
    border-bottom: 1px solid var(--outline-color1);

    button.toggled {
      &:first-of-type { background-color: var(--color-secondary3); }
      &:last-of-type { background-color: var(--color-highlight2); }
    }

    button:nth-of-type(-n+2) {
      width: 2.25rem;
      height: 2rem;
      border-radius: 0.25rem;
    }
  }

  #content {
    position: absolute;
    inset: 50px 0 20px 0;
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
  padding: 0.25rem;
  gap: 2px;
  border-radius: 0.5rem;
  background-color: var(--color-primary2);
  user-select: none;
  contain: layout style;
  will-change: width;

  button {
    justify-content: flex-start;
    height: 36px;
    width: 100%;
    padding: 2px 0.5rem;
    border-radius: 0.25rem;

    &:first-of-type {
      margin-top: 0;
    }

    &:last-of-type {
      margin-top: auto;
      justify-content: center;
      max-width: 35px;
      border-radius: 50%;
    }
  }
}

  #status-bar {
    position: absolute;
    inset: auto 0 0 0;
    height: 20px;
    padding: 2px 0.5rem;
    border-top: 1px solid var(--outline-color1);
    user-select: none;

    p {
      margin: 0;
      text-align: center;
      line-height: 0.75rem;
      font-size: 0.75rem;
      font-weight: bold;
    }
  }

  .alerts-container {
    position: fixed;
    z-index: 10000;
    bottom: 30px;
    left: 50%;
    justify-content: unset;
    transform: translateX(-50%);
    gap: 0.75rem;
    pointer-events: none;

    > * {
      pointer-events: auto;
    }
  }

  #cancel-recovery-button {
    position: fixed;
    z-index: 1000;
    top: 30px;
    width: 300px;
    height: 3rem;
    justify-self: center;
    justify-content: flex-start;
    padding: 2px 0.5rem;

    span {
      display: flex;
      align-items: center;
      height: 20px;
      font-size: 15px;
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
    outline: 1px solid var(--outline-color1);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);

    .timer-container {
      max-width: calc((100% - 40px) / 3);
    }
  }

  #layout-timers-list-topbar {
    justify-content: flex-start;
    width: 100%;
    gap: 0.75rem;
    padding-bottom: 0.75rem;
    border-bottom: 2px solid var(--outline-color1);
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