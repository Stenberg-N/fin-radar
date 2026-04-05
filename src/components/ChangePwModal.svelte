<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  import { t, lang } from "$lib/i18n";
  import { user } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { validatePassword } from "$lib/functions";

  let {
    setPwOverlayVisibility,
  }: {
    setPwOverlayVisibility: (state: boolean) => void;
  } = $props();

  let currentPassword = $state<string>('');
  let newPassword = $state<string>('');
  let confirmNewPassword = $state<string>('');
  let isMoved = $state<boolean>(false);

  const handleSubmit = async () => {
    if (newPassword !== confirmNewPassword) { sendAlert("alert.password.mismatch", true, false); return; };
    if (!validatePassword(newPassword).isValid) { sendAlert("alert.password.requirements-not-met", true, false); return; };

    try {
      await invoke('change_password', { id: $user?.id, name: $user?.name, currentPassword: currentPassword, newPassword: newPassword, confirmNewPassword: confirmNewPassword });
      sendAlert("alert.password.change.success", true, false);
      currentPassword = '';
      newPassword = '';
      confirmNewPassword = '';
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

<div id="change-pw-overlay">
  <div id="form-outer-container" style="pointer-events: auto;">
    <div style="position: relative; display: flex; flex-direction: row; align-items: center; justify-content: space-between;">
      <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
      <h1 style="position: absolute; left: 50%; transform: translateX(-50%); margin: 0;">{$t["form.change-password.title"]}</h1>
      <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setPwOverlayVisibility(false)}><img src="close-x.svg" alt="Close" style="width: 16px; height: 16px; filter: brightness(0);" /></button>
    </div>
    <form class="auth-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <div class="vertical-flex-container" style="align-items: unset;">
        <p class="form-p" style="padding: 0 6px;">{$t["form.change-password.current-password.title"]}</p>
        <div class="form-input-container">
          <input class="form-input" type="password" placeholder={$t["form.change-password.current-password.title"] as string} bind:value={currentPassword} required />
          <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
        </div>
      </div>
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
      <button class="primary-button form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["form.change-password.button.confirm"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
    </form>
  </div>
</div>

<style>
  #change-pw-overlay {
    position: fixed;
    z-index: 500;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(24px);
    pointer-events: none;
  }

  .transparent-button-highlight:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }
</style>