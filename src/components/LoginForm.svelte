<script lang="ts">
  import { t } from "$lib/i18n";
  import { login } from "$lib/user";
  import { sendAlert } from "$lib/alert";

  let {
    setLoginView,
  }: {
    setLoginView: (state: boolean) => void;
  } = $props();

  let username = $state<string>('');
  let password = $state<string>('');
  let isMoved = $state<boolean>(false);

  const handleSubmit = async () => {
    const result = await login(username, password);
    username = '';
    password = '';
    if (!result.success) {
      sendAlert("alert.login.message.fail", true, false);
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

<form class="auth-form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <div class="vertical-flex-container" style="align-items: unset;">
    <p class="form-p" style="padding: 0 6px;">{$t["form.username.title"]}</p>
    <div class="form-input-container">
      <input class="form-input" placeholder={$t["form.username.title"] as string} bind:value={username} required />
      <div class="form-input-spacer"></div>
    </div>
  </div>
  <div class="vertical-flex-container" style="align-items: unset;">
    <p class="form-p" style="padding: 0 6px;">{$t["form.password.title"]}</p>
    <div class="form-input-container">
      <input class="form-input" type="password" placeholder={$t["form.password.title"] as string} bind:value={password} required />
      <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
    </div>
  </div>
  <button class="primary-button form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["form.login.button"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
</form>

<div class="form-question-container">
  <p class="form-p">{$t["form.no-account.question"]}</p>
  <button class="form-button transparent-button" style="outline: none;" onclick={() => setLoginView(false)}>{$t["form.no-account.button"]}</button>
</div>

<style>

</style>