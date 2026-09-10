<script lang="ts">
  import { user } from "$lib/user";
  import { t } from "$lib/i18n/i18n";
  import ChangePwModal from "../../auth-user/ChangePwModal.svelte";
  import { setViewState } from "$lib/viewStore";

  let isChangePwVisible = $state<boolean>(false);

  const controls = $state([
    {
      get title() { return $t["change-password.title"]; },
      img: "/key.svg",
      command: () => { isChangePwVisible = !isChangePwVisible; },
    },
    {
      get title() { return $t["settings.pages.account.delete-account"]; },
      img: "/user.svg",
      command: () => { setViewState({ viewState: "isAskPassword", state: true }); },
    }
  ]);

</script>

<div id="main-settings-account-page-container">
  <div id="main-settings-account-page-info-container" class="vertical-flex-container">
    <div class="wrapper-div horizontal-flex-container" style="padding: 16px; gap: 16px; width: 100%; justify-content: flex-start;">
      {#each controls as button, i (i)}
        <button class="transparent-button-highlight" style="flex-shrink: 0;" onclick={button.command}>
          <span class="span-icon img-small" style="mask-image: url('{button.img}');"></span>
          {button.title}
        </button>
      {/each}
    </div>
    <div class="wrapper-div" style="width: 100%;">
      <h2 style="margin: 0;">{$t["username.title"]}</h2>
      <p>{$user?.name}</p>
    </div>
  </div>
  {#if isChangePwVisible}
    <ChangePwModal options={{ theme: "lighter-dark", isTranslationButtonVisible: false, justifyHeader: "left", justifyForm: "left" }} />
  {/if}
</div>

<style>
  #main-settings-account-page-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    width: 100%;
    padding: 48px;
    gap: 48px;
    background-color: #222;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

    .wrapper-div {
      padding: 32px;
      background-color: #333;
      border-radius: 8px;
      box-shadow: 0 4px 8px rgba(0, 0, 0, 0.8);

      button.transparent-button-highlight {
        border-radius: 4px;
        padding: 8px;
      }
    }
  }

  #main-settings-account-page-info-container {
    align-items: flex-start;
    justify-content: flex-start;
    gap: 24px;
  }
</style>