<script lang="ts">
  import { t, lang } from "$lib/i18n";
  import { resetPassword, user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { validatePassword } from "$lib/functions";

  let {
    switchViewState,
    isRecovery = false,
  }: {
    switchViewState?: (command: string, state: boolean) => void;
    isRecovery?: boolean;
  } = $props();

  let currentPassword = $state<string>('');
  let newPassword = $state<string>('');
  let confirmNewPassword = $state<string>('');
  let isMoved = $state<boolean>(false);

  $effect(() => {
    const pwOverlay = document.getElementById("change-pw-overlay");
    if (pwOverlay) {
      isRecovery ? pwOverlay.style.backgroundColor = "#0f0f0f" : pwOverlay.style.backdropFilter = "blur(24px)";
    }
  });

  const handleSubmit = async () => {
    if (newPassword !== confirmNewPassword) { sendAlert("alert.password.mismatch", true, false); return; };
    if (!validatePassword(newPassword).isValid) { sendAlert("alert.password.requirements-not-met", true, false); return; };

    try {
      await resetPassword(isRecovery, $user?.id, $user?.name, newPassword, confirmNewPassword, isRecovery ? undefined : currentPassword);
      sendAlert("alert.password.change.success", true, false);

      currentPassword = '';
      newPassword = '';
      confirmNewPassword = '';
      switchViewState ? switchViewState("setPwOverlayVisibility", false) : undefined;
    } catch (error) {
      sendAlert("alert.password.change.fail", true, false);
      newPassword = '';
      confirmNewPassword = '';
    }
  };

  const togglePasswordVisibility = (button: EventTarget | null) => {
    if (!button) return;

    const node = button as HTMLButtonElement;
    const passwordInput = node.previousElementSibling as HTMLInputElement | null;
    const img = node.firstChild as HTMLImageElement | null;

    if (passwordInput && img) {
      const isPassword = passwordInput.type === "password";
      passwordInput.type = isPassword ? "text" : "password";
      img.src = isPassword ? "/eye-hidden.svg" : "/eye-visible.svg";
      node.title = isPassword ? String($t["form.password-visibility.hide"]) : String($t["form.password-visibility.show"]);
    }
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
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => switchViewState("setPwOverlayVisibility", false)}><img src="close-x.svg" alt="Close" style="width: 16px; height: 16px; filter: brightness(0);" /></button>
      {/if}
    </div>
    <form class="auth-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#if !isRecovery}
        <div class="vertical-flex-container" style="align-items: unset;">
          <p class="form-p" style="padding: 0 6px;">{$t["form.change-password.current-password.title"]}</p>
          <div class="form-input-container">
            <input class="form-input" type="password" placeholder={$t["form.change-password.current-password.title"] as string} bind:value={currentPassword} required />
            <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
          </div>
        </div>
      {/if}
      <div class="vertical-flex-container" style="align-items: unset;">
        <p class="form-p" style="padding: 0 6px;">{$t["form.change-password.new-password.title"]}</p>
        <div class="form-input-container">
          <input class="form-input" type="password" placeholder={$t["form.change-password.new-password.title"] as string} bind:value={newPassword} required />
          <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
        </div>
      </div>
      <div class="vertical-flex-container" style="align-items: unset;">
        <p class="form-p" style="padding: 0 6px;">{$t["form.change-password.confirm-new-password.title"]}</p>
        <div class="form-input-container">
          <input class="form-input" type="password" placeholder={$t["form.change-password.confirm-new-password.title"] as string} bind:value={confirmNewPassword} required />
          <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
        </div>
      </div>
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