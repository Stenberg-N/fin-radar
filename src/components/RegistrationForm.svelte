<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  import { sendAlert } from "$lib/alert";
  import { t, lang } from "$lib/i18n";
  import { validatePassword, togglePasswordVisibility } from "$lib/functions";

  type FormKey = "username" | "password" | "confirmPassword";

  let {
    setLoginView,
  }: {
    setLoginView: (state: boolean) => void;
  } = $props();

  let form = $state<Record<FormKey, string>>({ username: '', password: '', confirmPassword: '' });
  let isMoved = $state<boolean>(false);
  let result = $state<string | null>(null);
  const inputElements = [
    { title: "form.username.title", key: "username"},
    { title: "form.password.title", key: "password"},
    { title: "form.confirm-password.title", key: "confirmPassword"},
  ];

  const handleSubmit = async () => {
    if (form.password !== form.confirmPassword) { sendAlert("alert.password.mismatch", true, false); return; }
    if (!validatePassword(form.password).isValid) { sendAlert("alert.password.requirements-not-met", true, false); return; }

    try {
      result = await invoke('create_user', { name: form.username, password: form.password, confirmPassword: form.confirmPassword });
      sendAlert("alert.registration.message.success", true, false);
      form.username = '';
      form.password = '';
      form.confirmPassword = '';
    } catch (error) {
      sendAlert("alert.registration.message.fail", true, false);
    }
  };

  const copyText = () => {
    if (!result || result === null) { sendAlert("alert.copy-text.fail", true, false); return; };

    navigator.clipboard.writeText(result);
    sendAlert("alert.copy-text.success", true, false);
  };
</script>

{#if result !== null}
  <div class="vertical-flex-container" style="position: fixed; z-index: 500; inset: 0; backdrop-filter: blur(24px); margin: 100px auto; user-select: none;">
    <div class="form-outer-container">
      <div class="horizontal-flex-container" style="justify-content: space-between;">
        <h2>{$t["recovery-key.modal.title"]}</h2>
        <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button" type="button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
      </div>
      <p>{$t["recovery-key.modal.paragraph"]}</p>
      <div id="recovery-key-container" class="horizontal-flex-container">
        <p style="margin: 0; font-size: 18px; user-select: text;">{result}</p>
        <button id="copy-key-button" class="transparent-button-highlight" onclick={() => copyText()}><img src="/copy.svg" alt="Copy" /></button>
      </div>
      <button class="primary-button form-primary-button" type="button" onclick={() => { result = null; setLoginView(true); }}>{$t["recovery-key.modal.confirm"]}</button>
    </div>
  </div>
{/if}

<div style="display: flex; flex-direction: column; gap: 40px;">
  <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    {#each inputElements as input, i (i)}
      <div class="vertical-flex-container" style="align-items: unset;">
        <p class="form-p">{$t[input.title]}</p>
        <div class="form-input-container">
          <input class="form-input" type={i === 0 ? "text" : "password"} placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
          {#if i === 0}
            <div class="form-input-spacer"></div>
          {:else}
            <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
              ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
              <img src="/eye-visible.svg" alt="Eye icon" />
            </button>
          {/if}
        </div>
      </div>
    {/each}
    <button class="primary-button form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["form.register.button"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
  </form>

  <div class="form-question-container">
    <p class="form-p">{$t["form.already-account.question"]}</p>
    <button class="form-button transparent-button" style="outline: none;" onclick={() => setLoginView(true)}>{$t["form.already-account.button"]}</button>
  </div>
</div>

<style>
  .transparent-button-highlight:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }

  #recovery-key-container {
    margin: 0 0 1em;
    min-height: 64px;
    height: 64px;
    gap: 20px;
    padding: 12px;
    background-color: rgba(180, 180, 180);
    border-radius: 8px;
    outline: 1px solid #333;
    justify-content: space-between;
    overflow-y: hidden;
    overflow-x: auto;
  }

  #copy-key-button {
    height: 40px;
    width: 40px;
    padding: 6px;
    border-radius: 6px;
    outline: 2px solid #333;
  }

  #copy-key-button:hover {
    transform: scale(1.05);
  }

  #copy-key-button img {
    max-width: 28px;
    max-height: 28px;
    filter: brightness(0);
  }
</style>