<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { appLocalDataDir } from "@tauri-apps/api/path";
  import { onMount } from 'svelte';

  import { updateUsername, user } from "$lib/user";
  import { t } from "$lib/i18n/i18n";
  import { setViewState } from "$lib/viewStore";
  import { sendAlert } from '$lib/alert';

  import ChangePwModal from "../../auth-user/ChangePwModal.svelte";

  let isChangePwVisible = $state<boolean>(false);
  let isRecoveryKeyUsed = $state<boolean>(false);
  let isIconMoved = $state<boolean>(false);
  let usernameInput = $state<string>($user?.name ?? '');

  const controls = [
    {
      get title() { return $t["settings.pages.account.button.backup-db"]; },
      command: () => backupDatabase(),
      img: "/database.svg",
      state: null,
    },
    {
      get title() { return $t["settings-banner.button.open-data"]; },
      command: () => openAppData(),
      img: "/folder.svg",
      state: null,
    },
    {
      get title() { return $t["settings.pages.account.user-info.title"]; },
      img: "/user.svg",
      command: () => { isChangePwVisible = false; },
      get state() { return !isChangePwVisible; },
    },
    {
      get title() { return $t["change-password.title"]; },
      img: "/key.svg",
      command: () => { isChangePwVisible = true; },
      get state() { return isChangePwVisible; },
    },
    {
      get title() { return $t["settings.pages.account.delete-account"]; },
      img: "/user.svg",
      command: () => { setViewState({ viewState: "isAskPassword", state: true }); },
      state: null,
    },
  ];

  const userInfo = [
    {
      get title() { return $t["username.title"]; },
      get content() { return $user?.name; },
    },
    {
      get title() { return $t["settings.pages.account.user-info.created-at"]; },
      get content() { return $user?.created_at; },
    },
    {
      get title() { return $t["settings.pages.account.user-info.last-password-change"]; },
      get content() { return $user?.last_password_change; },
    },
    {
      get title() { return $t["settings.pages.account.user-info.recovery-key-status"]; },
      get content() { return $t[`settings.pages.account.user-info.recovery-key-status.${isRecoveryKeyUsed === false ? 'not-' : ''}used`]; },
    },
  ];

  onMount(() => {
    (async () => isRecoveryKeyUsed = await invoke<boolean>('query_is_recovery_key_used'))();
  });

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
  <div class="wrapper-div flex row">
    {#each controls.slice(0, 2) as button, i (i)}
      <button class="button-primary transparent highlight" onclick={button.command}>
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
      <ChangePwModal options={{ theme: "dark", isTranslationButtonVisible: false, isBoxShadow: false, isLowerPadding: true }} />
    {:else}
      <div id="main-settings-account-page-user-info" class="flex column sub-wrapper-div">
        <h1>{$t["settings.pages.account.user-info.title"]}</h1>
        <div id="main-settings-account-page-user-info-wrapper" class="flex column">
          {#each userInfo as info, i (i)}
            <div class="flex column outline" style="align-items: unset; width: 100%;">
              <p>{i === 0 ? info.title + ':' : info.title}</p>
              <div class="flex row" style="justify-content: flex-start;">
                {#if i === 0}
                  <input class="primary-input" style="width: fit-content;" bind:value={usernameInput} />
                  <button class="button-primary light form" disabled={(info.content as string).trim() === usernameInput.trim()}
                    onclick={async () => await handleUpdateUsername()}
                    onmouseenter={() => isIconMoved = true}
                    onmouseleave={() => isIconMoved = false}
                  >
                    {$t["commit.button"]}
                    <span class="span-icon" class:moveRight={isIconMoved && (info.content as string).trim() !== usernameInput.trim()} style="mask-image: url('/arrow.svg');"></span>
                  </button>
                {:else}
                  <p style="{i === 3 ? `color: ${isRecoveryKeyUsed ? 'var(--color-positive)' : 'var(--color-negative)'}; font-weight: bold;` : ''}">{info.content}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
      <div class="flex column sub-wrapper-div">
        <h1>{$t["settings.pages.account.delete-account"]}</h1>
        <div class="flex column" style="align-items: unset; width: 100%;">
          <button class="button-primary light" style="height: unset;" onclick={() => setViewState({ viewState: "isAskPassword", state: true })}>
            <span class="span-icon img-medium" style="mask-image: url('trash-can.svg');"></span>
            {$t["settings.pages.account.delete-account"]}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  #main-settings-account-page-container {

    .wrapper-div {
      padding: 0.75rem;
      gap: 1rem;
      background-color: var(--color-secondary1);
      border-radius: 0.5rem;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button.button-primary.transparent.highlight {
        border-radius: 0.25rem;
        padding: 0.5rem;
      }
    }
  }

  #main-settings-account-page-switchbox {
    width: 100%;
    padding: 0.5rem;
    padding-bottom: 20px;
    gap: 3.75rem;
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
        padding: 6px;
        border-radius: 0.5rem;

        &.toggled {
          background-color: var(--color-highlight2);
        }
      }
    }

    .sub-wrapper-div {
      align-items: flex-start;
      align-self: stretch;
      padding: 20px 40px 40px;
      margin: 0 0.75rem;
      gap: 3rem;
      background-color: var(--color-primary2);
      border-radius: 0.5rem;
    }
  }

  #main-settings-account-page-user-info {

    #main-settings-account-page-user-info-wrapper {
      width: 100%;
      gap: 1rem;

      .outline {
        padding: 1rem;
        outline: 2px solid var(--outline-color1);
        border-radius: 0.5rem;
      }

      > div {
        justify-content: flex-start;
        height: 7rem;

        > p {
          margin: 0;
          font-weight: bold;
          user-select: none;
        }
        > div {
          gap: 1.5rem;

          input {
            height: 40px;
          }
        }
      }
    }
  }
</style>