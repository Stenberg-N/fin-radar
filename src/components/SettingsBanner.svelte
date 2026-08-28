<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { appLocalDataDir } from "@tauri-apps/api/path";
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { logout } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { setViewState } from "$lib/viewStore";

  const settingsButtons = [
    {
      id: 1,
      name: "settings-banner.button.open-data",
      command: () => openAppData(),
      icon: "folder.svg",
      alt: "Folder",
    },
    {
      id: 2,
      name: "settings-banner.button.backup-db",
      command: () => backupDatabase(),
      icon: "database.svg",
      alt: "Database",
    },
    {
      id: 3,
      name: "settings-banner.button.delete-user",
      command: () => { setViewState({ viewState: "isAskPassword", state: true }); setViewState({ viewState: "isMenu", state: false }); },
      icon: "user.svg",
      alt: "User",
    },
    {
      id: 4,
      name: "settings-banner.button.change-password",
      command: () => changePassword(),
      icon: "key.svg",
      alt: "Key",
    },
    {
      id: 5,
      name: "main.layout.logout",
      command: () => sendAlert({ message: "alert.logout.confirmation-question", isTimer: false, buttons: true, onConfirm: async () => await logout() }),
      icon: "logout.svg",
      alt: "Logout",
    },
  ];

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const handleOutsideClick = () => { setViewState({ viewState: "isMenu", state: false }); };
  
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

  const changePassword = () => {
    setViewState({ viewState: "isMenu", state: false });
    setViewState({ viewState: "isChangePwOverlay", state: true });
  };
</script>

<div role="menu" tabindex="0" id="settings-banner" class="modal-default vertical-flex-container" onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); setViewState({ viewState: "isMenu", state: false }); }}} transition:fly={{ x: 400, duration: 200, easing: cubicInOut }}
  use:handleClickOutside={{ onOutsideClick: handleOutsideClick, additionalElements: [] }}
>
  <div id="settings-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button id="close-button" class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState({ viewState: "isMenu", state: false })}><img src="close-x.svg" alt="Close" class="img-small" /></button>
  </div>
  <div id="settings-buttons" class="vertical-flex-container">
    {#each settingsButtons as button (button.id)}
      <button class="primary-button" onclick={() => button.command()}><img src={button.icon} alt={button.alt} />{$t[button.name]}</button>
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

  #settings-buttons button img {
    width: 20px;
    height: 20px;
    object-fit: contain;
  }
</style>