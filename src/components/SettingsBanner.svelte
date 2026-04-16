<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { appLocalDataDir } from "@tauri-apps/api/path";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { getContext } from "svelte";

  import { logout, user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { t } from "$lib/i18n";
  import { handleClickOutside } from "$lib/functions";
  import { setViewState } from "$lib/viewStore";

  // Context, Helper & Wrapper functions
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const handleOutsideClick = () => { setViewState("isMenu", false); };

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
      command: () => sendAlert("alert.delete-user.confirmation-question", false, true, () => deleteUser()),
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
      command: () => sendAlert("alert.logout.confirmation-question", false, true, () => logout()),
      icon: "logout.svg",
      alt: "Logout",
    },
  ];

  const openAppData = async () => {
    await openPath(await appLocalDataDir());
  };

  const backupDatabase = async () => {
    try {
      await invoke('backup_database');
      sendAlert("alert.backup-db.success", true, false);
    } catch (error) {
      sendAlert("alert.backup-db.fail", true, false);
    }
  };

  const changePassword = () => {
    setViewState("isMenu", false);
    setViewState("isChangePwOverlay", true);
  };

  const deleteUser = async () => {
    if (!user) return;

    try {
      await invoke('delete_user', { id: $user?.id });
      logout();
      sendAlert("alert.delete-user.message.success", true, false);
    } catch (error) {
      sendAlert("alert.delete-user.message.fail", true, false);
    }
  };
</script>

<div role="menu" tabindex="0" id="settings-banner" class="vertical-flex-container" onkeydown={(e) => { if (e.key === 'Escape') { e.preventDefault(); setViewState("isMenu", false); }}} transition:slide={{ duration: 200, easing: cubicInOut }}
  use:handleClickOutside={{ getIgnoredElements, onOutsideClick: handleOutsideClick, additionalElements: [] }}
>
  <div id="settings-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button id="close-button" class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState("isMenu", false)}><img src="close-x.svg" alt="Close" class="img-small" style="filter: brightness(0) invert(0.9);" /></button>
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
    align-items: flex-start;
    justify-content: flex-start;
    width: 400px;
    padding: 12px;
    gap: 12px;
    border: none;
    outline: 1px solid #333;
    border-radius: 8px;
    background-color: #181818;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);
  }

  #settings-topbar {
    width: 100%;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 1px solid #333;
  }

  #settings-buttons {
    width: 100%;
    gap: 6px;
  }

  #settings-buttons button {
    display: flex;
    align-items: center;
    width: 100%;
    gap: 12px;
    text-align: left;
  }

  #settings-buttons button img {
    width: 20px;
    height: 20px;
    filter: brightness(0) invert(0.9);
    object-fit: contain;
  }

  #settings-buttons button:hover {
    background-color: #555;
  }
</style>