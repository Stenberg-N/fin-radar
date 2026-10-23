<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { appLocalDataDir, appDataDir } from "@tauri-apps/api/path";
  import { onMount } from 'svelte';

  import { updateUsername, user } from "$lib/user";
  import { i18n, type Language } from "$lib/i18n/i18n.svelte";
  import { setViewState } from "$lib/utils/viewStore";
  import { sendAlert } from '$lib/alert';
  import { updateUserPrefs, userPrefs } from '$lib/utils/prefsStore';

  import ChangePwModal from "../../auth-user/ChangePwModal.svelte";
  import ModalWrapper from '../../ModalWrapper.svelte';

  type Currency = "$" | "€" | "£";
  type HourFormat = "12" | "24";

  let isChangePwVisible = $state<boolean>(false);
  let isRecoveryKeyUsed = $state<boolean>(false);
  let isIconMoved = $state<boolean>(false);
  let isFileOptionsVisible = $state<boolean>(false);
  let usernameInput = $state<string>($user?.name ?? '');

  let controlsRefs = $state<(HTMLButtonElement | null)[]>([]);
  let openFileOptionsButton = $state<HTMLButtonElement | null>(null);

  const controls = [
    {
      get title() { return i18n.t["settings.pages.account.button.backup-db"]; },
      command: () => backupDatabase(),
      img: "/database.svg",
      state: null,
    },
    {
      get title() { return i18n.t[`${isFileOptionsVisible ? "cancel" : "open"}.button`]; },
      command: () => { isFileOptionsVisible = !isFileOptionsVisible; },
      img: "/folder.svg",
      state: null,
    },
    {
      get title() { return i18n.t["settings.pages.account.user-info.title"]; },
      img: "/user.svg",
      command: () => { isChangePwVisible = false; },
      get state() { return !isChangePwVisible; },
    },
    {
      get title() { return i18n.t["change-password.title"]; },
      img: "/key.svg",
      command: () => { isChangePwVisible = true; },
      get state() { return isChangePwVisible; },
    },
    {
      get title() { return i18n.t["settings.pages.account.delete-account"]; },
      img: "/user.svg",
      command: () => { setViewState({ viewState: "isAskPassword", state: true }); },
      state: null,
    },
  ];

  const prefs = [
    {
      get title() { return i18n.t["settings.pages.account.language"]; },
      get value() { return $userPrefs.mainPrefs.lang; },
      get items() { return Object.entries(i18n.t["settings.pages.account.available-languages"] as Record<string, string>); },
      onchange: (e: Event & { currentTarget: EventTarget & HTMLSelectElement }) => {
        i18n.lang = e.currentTarget?.value as Language;
      },
    },
    {
      get title() { return i18n.t["settings.pages.account.currency"]; },
      get value() { return $userPrefs.mainPrefs.currency; },
      items: [["$", "$"], ["€", "€"], ["£", "£"]] as [string, string][], // Made to be identical in structure as the first item.
      onchange: (e: Event & { currentTarget: EventTarget & HTMLSelectElement }) => {
        updateUserPrefs("mainPrefs", "currency", e.currentTarget.value as Currency);
      },
    },
    {
      get title() { return i18n.t["settings.pages.account.time-format"]; },
      get value() { return $userPrefs.mainPrefs.hourFormat; },
      items: [["12", "12h"], ["24", "24h"]] as [string, string][], // Made to be identical in structure as the first item.
      onchange: (e: Event & { currentTarget: EventTarget & HTMLSelectElement }) => {
        updateUserPrefs("mainPrefs", "hourFormat", e.currentTarget.value as HourFormat);
      },
    },
  ];

  const userInfo = [
    {
      get title() { return i18n.t["username.title"]; },
      get content() { return $user?.name; },
    },
    {
      get title() { return i18n.t["settings.pages.account.user-info.created-at"]; },
      get content() { return $user?.created_at; },
    },
    {
      get title() { return i18n.t["settings.pages.account.user-info.last-password-change"]; },
      get content() { return $user?.last_password_change; },
    },
    {
      get title() { return i18n.t["settings.pages.account.user-info.recovery-key-status"]; },
      get content() { return i18n.t[`settings.pages.account.user-info.recovery-key-status.${isRecoveryKeyUsed === false ? 'not-' : ''}used`]; },
    },
  ];

  const fileOptions = [
    {
      get title() { return i18n.t["settings.pages.account.user-prefs.title"] as string },
      command: () => { openPrefsData(); isFileOptionsVisible = false; },
    },
    {
      get title() { return i18n.t["settings.pages.account.user-data.title"] as string },
      command: () => { openAppData(); isFileOptionsVisible = false; },
    }
  ];

  onMount(() => {
    (async () => isRecoveryKeyUsed = await invoke<boolean>('query_is_recovery_key_used'))();
  });

  $effect(() => {
    if (controlsRefs[1] !== null) openFileOptionsButton = controlsRefs[1];
  })

  const backupDatabase = async () => {
    try {
      await invoke('backup_database');
      sendAlert({ message: "alert.backup-db.success", isTimer: true, buttons: false });
    } catch (error) {
      sendAlert({ message: "alert.backup-db.fail", isTimer: true, buttons: false });
    }
  };

  const openAppData = async () => {
    await openPath(await appLocalDataDir());
  };

  const openPrefsData = async () => {
    await openPath(await appDataDir());
  };

  const handleUpdateUsername = async () => {
    sendAlert({
      message: "alert.update-username.confirmation",
      isTimer: false,
      buttons: true,
      onConfirm: () => updateUsername(usernameInput),
      additionalText: usernameInput
    });
  };

</script>

<div id="main-settings-account-page-container" class="main-settings-page-container flex column">
  {#if isFileOptionsVisible}
    <ModalWrapper
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      onOutsideClick={() => { isFileOptionsVisible = !isFileOptionsVisible; }}
      ignorableEls={[openFileOptionsButton]}
      outline={{ width: 1, color: "var(--outline-color2)" }}
      borderRadius={8}
    >
      <div id="open-file-options-container" class="flex column">
        {#each fileOptions as option, i (i)}
          <button class="file-option button-primary transparent highlight default-corners" onclick={option.command}>
            {option.title}
          </button>
        {/each}
      </div>
    </ModalWrapper>
  {/if}

  <div class="wrapper-div flex row">
    {#each controls.slice(0, 2) as button, i (i)}
      <button class="button-primary transparent highlight outline default-corners" onclick={button.command} bind:this={controlsRefs[i]}>
        <span class="span-icon img-small" style="mask-image: url('{button.img}');"></span>
        {button.title}
      </button>
    {/each}
  </div>
  <div id="main-settings-account-page-switchbox" class="flex column">
    <div class="wrapper-div flex row">
      {#each controls.slice(2, 4) as button, i (i)}
        <button class="button-primary transparent highlight" class:toggled={button.state} onclick={button.command}>
          <span class="span-icon img-small" style="mask-image: url('{button.img}');"></span>
          {button.title}
        </button>
      {/each}
    </div>
    {#if isChangePwVisible}
      <ChangePwModal theme="dark" isTranslationButtonVisible={false} isBoxShadow={false} isLowerPadding={true} />
    {:else}
      <div class="flex column sub-wrapper-div">
        <h1>{i18n.t["settings.pages.account.user-prefs.title"]}</h1>
        <div class="flex column">
          {#each prefs as entry, i (entry)}
            <div class="setting-container flex column">
              <p>{i === 0 ? entry.title + ':' : entry.title}</p>
              <select class="primary-input" value={entry.value} onchange={(e) => entry.onchange(e)}>
                {#each entry.items as item (item)}
                  <option value={item[0]}>
                    {item[1]}
                  </option>
                {/each}
              </select>
            </div>
          {/each}
        </div>
      </div>
      <div id="main-settings-account-page-user-info" class="flex column sub-wrapper-div">
        <h1>{i18n.t["settings.pages.account.user-info.title"]}</h1>
        <div id="main-settings-account-page-user-info-wrapper" class="flex column">
          {#each userInfo as info, i (i)}
            <div class="setting-container flex column">
              <p>{i === 0 ? info.title + ':' : info.title}</p>
              <div class="flex row" style="justify-content: flex-start;">
                {#if i === 0}
                  <input class="primary-input" bind:value={usernameInput} />
                  <button class="button-primary white-bg form" disabled={(info.content as string).trim() === usernameInput.trim()}
                    onclick={async () => await handleUpdateUsername()}
                    onmouseenter={() => isIconMoved = true}
                    onmouseleave={() => isIconMoved = false}
                  >
                    {i18n.t["commit.button"]}
                    <span class="span-icon" class:moveRight={isIconMoved && (info.content as string).trim() !== usernameInput.trim()} style="mask-image: url('/arrow.svg');"></span>
                  </button>
                {:else}
                  <p style="{i === 3 ? `color: ${isRecoveryKeyUsed ? 'var(--color-negative)' : 'var(--color-positive)'}; font-weight: bold;` : ''}">{info.content}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
      <div class="flex column sub-wrapper-div">
        <h1>{i18n.t["settings.pages.account.delete-account"]}</h1>
        <div class="flex column" style="align-items: unset; width: 100%;">
          <button class="button-primary" style="height: unset;" onclick={() => setViewState({ viewState: "isAskPassword", state: true })}>
            <span class="span-icon img-medium" style="mask-image: url('trash-can.svg');"></span>
            {i18n.t["settings.pages.account.delete-account"]}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  #main-settings-account-page-container {

    .setting-container {
      align-items: unset;

      select {
        padding: 0 0.25rem;

        &:hover {
          cursor: pointer;
        }

        option {
          background-color: var(--color-primary1);
        }
      }

      .primary-input {
        height: 32px;
        max-width: 180px;
      }

      > div {
        gap: 1.5rem;
      }
    }

    .wrapper-div {
      padding: 0.75rem;
      gap: 1rem;
      background-color: var(--color-secondary1);
      border-radius: 0.5rem;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button.button-primary.transparent.highlight {
        border-radius: 0.25rem;
        padding: 0.5rem;
        white-space: nowrap;
      }
    }

    #open-file-options-container {
      align-items: unset;
      padding: 0.5rem;
      gap: 0.25rem;
      background-color: var(--color-secondary2);

      .file-option {
        justify-content: flex-start;
      }
    }
  }

  #main-settings-account-page-switchbox {
    width: 100%;
    padding: 0.5rem;
    padding-bottom: 20px;
    gap: 3rem;
    background-color: var(--color-secondary1);
    border-radius: 1rem;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    > .wrapper-div {
      width: 100%;
      padding: 0.25rem;
      gap: 0.25rem;
      border-radius: 0.75rem;
      background-color: var(--color-primary2);
      box-shadow: none;

      button.button-primary.transparent.highlight {
        width: 100%;
        height: 100%;
        padding: 0.25rem;
        border-radius: 0.5rem;

        &.toggled {
          background-color: var(--color-highlight2);
        }
      }
    }

    .sub-wrapper-div {
      align-items: flex-start;
      align-self: stretch;
      padding: 20px 2.5rem 2.5rem;
      margin: 0 0.75rem;
      gap: 3rem;
      background-color: var(--color-primary2);
      border-radius: 0.5rem;

      > div {
        width: 100%;
        gap: 1rem;
      }
    }
  }

  #main-settings-account-page-user-info {

    #main-settings-account-page-user-info-wrapper {

      .button-primary.form {
        align-self: center;
        margin-top: unset;
      }
    }
  }
</style>