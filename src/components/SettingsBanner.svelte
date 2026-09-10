<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { appLocalDataDir } from "@tauri-apps/api/path";
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { logout } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { setViewState } from "$lib/viewStore";

  const settingsButtons = [
    {
      get name() { return $t["settings-banner.button.open-settings"]; },
      command: () => handleSettingsOverlay(),
      icon: "/settings-cog.svg",
    },
    {
      get name() { return $t["settings-banner.button.open-data"]; },
      command: () => openAppData(),
      icon: "/folder.svg",
    },
    {
      get name() { return $t["settings-banner.button.backup-db"]; },
      command: () => backupDatabase(),
      icon: "/database.svg",
    },
    {
      get name() { return $t["main.layout.logout"]; },
      command: () => sendAlert({ message: "alert.logout.confirmation-question", isTimer: false, buttons: true, onConfirm: async () => await logout() }),
      icon: "/logout.svg",
    },
  ];

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleOutsideClick = () => { setViewState({ viewState: "isMenu", state: false }); };
  const handleSettingsOverlay = () => {
    setViewState({ viewState: "isSettingsOverlay", state: true });
    setViewState({ viewState: "isMenu", state: false });
  };
  
  /***********************************************************************************************************************************/
  const openAppData = async () => {
    await openPath(await appLocalDataDir());
  };

  const backupDatabase = async () => {
    try {
      await invoke('backup_database');
      sendAlert({ message: "alert.backup-db.success", isTimer: true, buttons: false });
    } catch (error) {
      sendAlert({ message: "alert.backup-db.fail", isTimer: true, buttons: false });
    }
  };
</script>

<div role="menu" tabindex="0" id="settings-banner" class="modal-default vertical-flex-container" transition:fly={{ x: 400, duration: 200, easing: cubicInOut }}
  onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); setViewState({ viewState: "isMenu", state: false }); }}}
  use:handleClickOutside={{ onOutsideClick: handleOutsideClick, additionalElements: [] }}
>
  <div id="settings-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button aria-label="Close menu" id="close-button" class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState({ viewState: "isMenu", state: false })}>
      <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
    </button>
  </div>
  <div id="settings-buttons" class="vertical-flex-container">
    {#each settingsButtons as button, i (i)}
      <button class="primary-button" onclick={() => button.command()}>
        <span class="span-icon" style="mask-image: url('{button.icon}');"></span>
        {button.name}
      </button>
    {/each}
  </div>
</div>

<style>
  #settings-banner {
    position: fixed;
    z-index: 1000;
    right: 10px;
    top: 45px;
    justify-content: flex-start;
    width: 400px;
  }

  #settings-topbar {
    width: 100%;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 2px solid #333;
  }

  #settings-buttons {
    width: 100%;
    gap: 6px;
  }

  #settings-buttons button span {
    width: 20px;
    height: 20px;
    object-fit: contain;
  }
</style>