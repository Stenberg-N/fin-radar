<script lang="ts">
  import { sendAlert } from "$lib/alert";
  import { lang, t } from "$lib/i18n";
  import { recoverPassword } from "$lib/user";

  type FormKey = "accountName" | "recoveryKey";

  let {
    switchViewState,
  }: {
    switchViewState: (command: string, state: boolean) => void;
  } = $props();

  let form = $state<Record<FormKey, string>>({ accountName: '', recoveryKey: '' });

  const inputElements = [
    { title: "form.username.title", key: "accountName" },
    { title: "form.forgot-password.recovery-key.title", key: "recoveryKey" },
  ];

  let isMoved = $state<boolean>(false);
  
  const handleSubmit = async () => {
    if (form.accountName.trim() === '' || form.recoveryKey.trim() === '') { sendAlert("alert.password.recover.missing-info", true, false); return; };

    const result = await recoverPassword(form.accountName, form.recoveryKey);
    switchViewState("setRecoveryView", false);
    if (!result.success) {
      sendAlert("alert.password.recover.fail", true, false);
    }
    form.accountName = '';
    form.recoveryKey = '';
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

<div class="vertical-flex-container" style="position: fixed; z-index: 500; inset: 0; backdrop-filter: blur(48px); padding: 100px 0; pointer-events: none;">
  <div class="form-outer-container" style="pointer-events: auto;">
    <div class="vertical-flex-container">
      <div class="horizontal-flex-container" style="justify-content: space-between; width: 100%;">
        <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button" type="button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" type="button" onclick={() => switchViewState("setRecoveryView", false)}><img src="close-x.svg" alt="Close" class="img-small" style="filter: brightness(0);" /></button>
      </div>
      <h2>{$t["form.forgot-password.title"]}</h2>
      <p>{$t["form.forgot-password.paragraph"]}</p>
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#each inputElements as input, i (i)}
        <div class="vertical-flex-container" style="align-items: unset;">
          <p class="form-p">{$t[input.title]}</p>
          <div class="form-input-container">
            <input class="form-input" type={i === 0 ? "text" : "password"} placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
            {#if i === 0}
              <div class="form-input-spacer"></div>
            {:else}
              <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
            {/if}
          </div>
        </div>
      {/each}
      <button class="primary-button form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["confirm.button"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
    </form>
  </div>
</div>

<style>
  .transparent-button-highlight:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }
</style>