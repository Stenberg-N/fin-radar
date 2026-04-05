<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  import { lang, t } from "$lib/i18n";
  import { logout, user } from "$lib/user";
  import { alerts, sendAlert } from "$lib/alert";

  import AuthScreen from "../components/AuthScreen.svelte";
  import Alert from "../components/Alert.svelte";
  import SettingsBanner from "../components/SettingsBanner.svelte";
  import ChangePwModal from "../components/ChangePwModal.svelte";
  import "../styles.css";

  let { children } = $props();
  let isMenu = $state<boolean>(false);
  let isChangePwOverlay = $state<boolean>(false);
  let menuToggleBtn = $state<HTMLButtonElement | null>(null);
  let alertsContainer = $state<HTMLDivElement | null>(null);
  let langToggleBtn = $state<HTMLButtonElement | null>(null);

  // Wrapper/helper functions
  const setMenuVisibility = (state: boolean) => {
    isMenu = state;
  };

  const setPwOverlayVisibility = (state: boolean) => {
    isChangePwOverlay = state;
  };

  onMount(async () => {
    await listen('app-closing', () => {
      logout();
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
{:else}
  {#if isMenu}
    <SettingsBanner setMenuVisibility={setMenuVisibility} setPwOverlayVisibility={setPwOverlayVisibility} {menuToggleBtn} {alertsContainer} {langToggleBtn} />
  {/if}

  {#if isChangePwOverlay}
    <ChangePwModal setPwOverlayVisibility={setPwOverlayVisibility} />
  {/if}

  <nav id="nav-bar">
    <button class="horizontal-flex-container transparent-button" onclick={() => { sendAlert("alert.logout.confirmation-question", false, true, () => logout()); }}><img src="logout.svg" alt="Logout" /><span>{$t["main.layout.logout"]}</span></button>
  </nav>

  <div id="menu-bar" class="horizontal-flex-container">
    <h2 id="view-title">{$t["main.layout.view-title"][0]}</h2>
    <button bind:this={langToggleBtn} title={$t["language.button.title"] as string} class="primary-button" style="width: 36px; height: 32px; font-weight: 600;" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
    <button bind:this={menuToggleBtn} title={$t["main.layout.button.menu-toggle"] as string} class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => isMenu = !isMenu}><img style="width: 20px; height: 20px; filter: brightness(0) invert(0.9);" src="burger.svg" alt="Menu" /></button>
  </div>

  <div id="status-bar">
    <p>Status bar</p>
  </div>

  <main id="container" class="vertical-flex-container">
    {@render children()}
  </main>
{/if}

<style>
  #container {
    position: fixed;
    top: 50px;
    left: 140px;
    right: 0;
    bottom: 20px;
    margin: 0;
    text-align: center;
    border-left: 1px solid red;
    border-top: 1px solid red;
    border-radius: 12px 0 0 0;
  }

  #menu-bar {
    position: fixed;
    left: 140px;
    right: 0;
    top: 0;
    justify-content: flex-end;
    height: 50px;
    gap: 12px;
    padding: 8px;
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
    width: 140px;
    
  }

  #nav-bar button {
    height: 32px;
    justify-content: flex-start;
    gap: 8px;
    padding: 2px 8px;
  }

  #nav-bar button:hover {
    background-color: rgba(200, 200, 200, 0.2);
  }

  #nav-bar button span {
    display: flex;
    align-items: center;
    height: 20px;
    font-size: 15px;
    color: #f6f6f6;
    font-weight: bold;
  }

  #nav-bar button img {
    width: 20px;
    height: 20px;
    filter: brightness(0) invert(0.9);
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
    border-top: 1px solid blue;
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
</style>