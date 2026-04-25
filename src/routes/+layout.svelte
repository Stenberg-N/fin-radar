<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount, setContext } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto, onNavigate } from "$app/navigation";

  import { lang, t } from "$lib/i18n";
  import { logout, user } from "$lib/user";
  import { alerts, sendAlert } from "$lib/alert";
  import { viewStore } from "$lib/viewStore";

  import AuthScreen from "../components/AuthScreen.svelte";
  import Alert from "../components/Alert.svelte";
  import SettingsBanner from "../components/SettingsBanner.svelte";
  import ChangePwModal from "../components/ChangePwModal.svelte";
  import RecoveryScreen from "../components/RecoveryScreen.svelte";
  import "../styles.css";
  import { page } from "$app/state";

  let { children } = $props();
  let isMenu = $derived($viewStore.isMenu);
  let isChangePwOverlay = $derived($viewStore.isChangePwOverlay);
  let isRecoveryView = $derived($viewStore.isRecoveryView);

  let menuToggleBtn = $state<HTMLButtonElement | null>(null);
  let alertsContainer = $state<HTMLDivElement | null>(null);
  let langToggleBtn = $state<HTMLButtonElement | null>(null);

  const viewTitleIdx = $derived(() => {
    switch(page.url.pathname) {
      case "/": return 0;
      case "/transactions-table": return 1;
      default: return -1;
    }
  });
  const navButtons = [
    { path: "/", img: "/home.svg" },
    { path: "/transactions-table", img: "/coins.svg" }
  ];

  onMount(async () => {
    await listen('app-closing', () => {
      logout();
    });
  });

  /***********************************************************************************************************************************
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = () => [menuToggleBtn, alertsContainer, langToggleBtn];
  setContext('ignoredElements', getIgnoredElements);

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

  onNavigate(({ from, to}) => {
    return new Promise((resolve) => {
      document.startViewTransition(() => {
        resolve();
      });
    });
  });

</script>

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

  <nav id="nav-bar">
    {#each navButtons as {path, img}, i (i)}
      <button class="transparent-button-highlight" class:current={page.url.pathname === path} onclick={() => { goto(path); }}><img src={img} alt="nav-icon" /><span>{$t["main.layout.view-title"][i]}</span></button>
    {/each}
  </nav>

  <div id="menu-bar" class="horizontal-flex-container">
    <h2 id="view-title">{$t["main.layout.view-title"][viewTitleIdx()]}</h2>
    <button bind:this={langToggleBtn} title={$t["language.button.title"] as string} class="primary-button" style="width: 36px; height: 32px; font-weight: 600;" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
    <button bind:this={menuToggleBtn} title={$t["main.layout.button.menu-toggle"] as string} class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => isMenu = !isMenu}><img style="width: 20px; height: 20px;" src="burger.svg" alt="Menu" /></button>
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