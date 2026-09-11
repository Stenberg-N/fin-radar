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

  let controls = $state([
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
  ]);

  const userInfo = $derived([
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
  ]);

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

<div id="main-settings-account-page-container" class="vertical-flex-container">
  <div class="wrapper-div horizontal-flex-container">
    {#each controls.slice(0, 2) as button, i (i)}
      <button class="transparent-button-highlight" onclick={button.command}>
        <span class="span-icon img-small" style="mask-image: url('{button.img}');"></span>
        {button.title}
      </button>
    {/each}
  </div>
  <div id="main-settings-account-page-switchbox" class="vertical-flex-container">
    <div class="wrapper-div horizontal-flex-container">
      {#each controls.slice(2, 4) as button, i (i)}
        <button class="transparent-button-highlight" class:toggled={button.state} onclick={button.command}>
          <span class="span-icon img-small" style="mask-image: url('{button.img}');"></span>
          {button.title}
        </button>
      {/each}
    </div>
    {#if isChangePwVisible}
      <ChangePwModal options={{ theme: "dark", isTranslationButtonVisible: false, isBoxShadow: false, isLowerPadding: true }} />
    {:else}
      <div id="main-settings-account-page-user-info" class="vertical-flex-container">
        <h1>{$t["settings.pages.account.user-info.title"]}</h1>
        <div id="main-settings-account-page-user-info-wrapper" class="vertical-flex-container">
          {#each userInfo as info, i (i)}
            <div class="vertical-flex-container outline" style="align-items: unset; width: 100%;">
              <p>{i === 0 ? info.title + ':' : info.title}</p>
              <div class="horizontal-flex-container" style="justify-content: flex-start;">
                {#if i === 0}
                  <input class="primary-input" style="width: fit-content;" bind:value={usernameInput} />
                  <button class="primary-button-light form-primary-button" disabled={(info.content as string).trim() === usernameInput.trim()}
                    onclick={async () => await handleUpdateUsername()}
                    onmouseenter={() => isIconMoved = true}
                    onmouseleave={() => isIconMoved = false}
                  >
                    {$t["commit.button"]}
                    <span class="span-icon" class:moveRight={isIconMoved && (info.content as string).trim() !== usernameInput.trim()} style="mask-image: url('/arrow.svg');"></span>
                  </button>
                  <button class="primary-button-light" style="height: unset; margin-left: auto;" onclick={() => setViewState({ viewState: "isAskPassword", state: true })}>
                    <span class="span-icon img-medium" style="mask-image: url('trash-can.svg');"></span>
                    {$t["settings.pages.account.delete-account"]}
                  </button>
                {:else}
                  <p>{info.content}</p>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  #main-settings-account-page-container {
    justify-content: flex-start;
    align-items: flex-start;
    max-width: 1360px;
    width: 100%;
    padding: 60px;
    gap: 60px;
    background-color: #222;
    border-radius: 16px;

    .wrapper-div {
      padding: 12px;
      gap: 16px;
      background-color: #333;
      border-radius: 8px;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button.transparent-button-highlight {
        border-radius: 4px;
        padding: 8px;
      }
    }
  }

  #main-settings-account-page-switchbox {
    width: 100%;
    padding: 8px;
    padding-bottom: 20px;
    gap: 60px;
    background-color: #333;
    border-radius: 16px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    > .wrapper-div {
      width: 100%;
      padding: 4px;
      gap: 4px;
      border-radius: 12px;
      background-color: #222;
      box-shadow: none;

      button.transparent-button-highlight {
        width: 100%;
        height: 100%;
        padding: 6px;
        border-radius: 8px;

        &.toggled {
          background-color: rgba(200, 200, 200, 0.2);
        }
      }
    }
  }

  #main-settings-account-page-user-info {
    align-items: flex-start;
    align-self: stretch;
    padding: 20px 40px 40px;
    margin: 0 12px;
    gap: 48px;
    background-color: #222;
    border-radius: 8px;

    #main-settings-account-page-user-info-wrapper {
      width: 100%;
      gap: 32px;

      .outline {
        padding: 16px;
        outline: 2px solid #333;
        border-radius: 8px;
      }

      > div {
        justify-content: flex-start;
        height: 112px;

        > p {
          margin: 0;
          font-weight: bold;
          user-select: none;
        }
        > div {
          gap: 24px;

          input {
            height: 40px;
          }
        }
      }
    }
  }
</style>