<script lang="ts">
  import { t, lang } from "$lib/i18n";
  import { resetPassword, user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { validatePassword } from "$lib/functions";
  import { setViewState } from "$lib/viewStore";
  import { togglePasswordVisibility } from "$lib/functions";

  let {
    switchViewState,
    isRecovery = false,
  }: {
    switchViewState?: boolean;
    isRecovery?: boolean;
  } = $props();

  type FormKey = "currentPassword" | "newPassword" | "confirmNewPassword";

  let form = $state<Record<FormKey, string>>({ currentPassword: '', newPassword: '', confirmNewPassword: '' });
  let isMoved = $state<boolean>(false);
  
  const inputElements = [
    { title: "form.change-password.current-password.title", key: "currentPassword"},
    { title: "form.change-password.new-password.title", key: "newPassword" },
    { title: "form.change-password.confirm-new-password.title", key: "confirmNewPassword" },
  ];

  $effect(() => {
    const pwOverlay = document.getElementById("change-pw-overlay");
    if (pwOverlay) {
      isRecovery ? pwOverlay.style.backgroundColor = "#0f0f0f" : pwOverlay.style.backdropFilter = "blur(24px)";
    }
  });

  const handleSubmit = async () => {
    if (form.newPassword !== form.confirmNewPassword) { sendAlert("alert.password.mismatch", true, false); return; };
    if (!validatePassword(form.newPassword).isValid) { sendAlert("alert.password.requirements-not-met", true, false); return; };

    const result = await resetPassword(isRecovery, $user?.id, $user?.name, form.newPassword, form.confirmNewPassword, isRecovery ? undefined : form.currentPassword);

    if (!result.success) {
      sendAlert("alert.password.change.fail", true, false);
      form.newPassword = '';
      form.confirmNewPassword = '';
      return;
    }

    sendAlert("alert.password.change.success", true, false);
    form.currentPassword = '';
    form.newPassword = '';
    form.confirmNewPassword = '';
    switchViewState ? setViewState("isChangePwOverlay", false) : undefined;
  };

</script>

<div id="change-pw-overlay" class="vertical-flex-container">
  {#if isRecovery}
    <div id="cancel-recovery-paragraph-container" class="vertical-flex-contaier">
      {#each $t["form.change-password.cancel-recovery.message"] as text, i (i)}
        <p class="cancel-recovery-paragraph" style="color: {i === 0 ? "rgba(255, 70, 70, 1)" : "#f6f6f6"}; font-weight: {i === 0 ? 800 : 400};">{text}</p>
      {/each}
    </div>
  {/if}
  <div class="form-outer-container" style="gap: 40px;">
    <div style="position: relative; display: flex; flex-direction: row; align-items: center; justify-content: space-between;">
      <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
      <h1 style="position: absolute; left: 50%; transform: translateX(-50%); margin: 0;">{$t["form.change-password.title"]}</h1>
      {#if switchViewState}
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState("isChangePwOverlay", false)}><img src="close-x.svg" alt="Close" class="img-small" style="filter: brightness(0);" /></button>
      {/if}
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#each isRecovery ? inputElements.slice(1, 3) : inputElements  as input, i (i)}
        <div class="vertical-flex-container" style="align-items: unset;">
          <p class="form-p">{$t[input.title]}</p>
          <div class="form-input-container">
            <input class="primary-input" type="password" placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
            <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
              ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
              <img src="/eye-visible.svg" alt="Eye icon" />
            </button>
          </div>
        </div>
      {/each}
      <button class="primary-button form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["confirm.button"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
    </form>
  </div>
</div>

<style>
  #change-pw-overlay {
    position: fixed;
    z-index: 500;
    inset: 0;
  }

  #cancel-recovery-paragraph-container {
    max-width: 800px;
    margin: 68px 0 20px;
  }

  .cancel-recovery-paragraph {
    margin: 0;
    text-align: center;
    word-wrap: break-word;
    hyphens: auto;
    user-select: none;
  }

  .transparent-button-highlight:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }
</style>