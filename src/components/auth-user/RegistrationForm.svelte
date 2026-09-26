<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { sendAlert } from "$lib/alert";
  import { t, lang } from "$lib/i18n/i18n";
  import { createUser } from "$lib/user";
  import { validatePassword, togglePasswordVisibility } from "$lib/user";

  type FormKey = "username" | "password" | "confirmPassword";

  let {
    setLoginView,
  }: {
    setLoginView: (state: boolean) => void;
  } = $props();

  let form = $state<Record<FormKey, string>>({ username: '', password: '', confirmPassword: '' });
  let isMoved = $state<boolean>(false);
  let result = $state<string | null>(null);
  let recoveryConfirmButton = $state<HTMLButtonElement | null>(null);
  const duration = 4000;
  let remainingDuration = $state(duration);
  let durationInterval: ReturnType<typeof setInterval> | null = null;
  const inputElements = [
    { title: "username.title", key: "username"},
    { title: "password.title", key: "password"},
    { title: "confirm-password.title", key: "confirmPassword"},
  ];

  $effect(() => {
    if (!result || !recoveryConfirmButton) return;

    const progress = `${((duration - remainingDuration) / duration) * 100}%`;
    recoveryConfirmButton.style.setProperty('--progress-bar-width', progress);
  });

  const handleSubmit = async () => {
    if (form.password !== form.confirmPassword) { sendAlert({ message: "alert.password.mismatch", isTimer: true, buttons: false }); return; }
    if (!validatePassword(form.password).isValid) { sendAlert({ message: "alert.password.requirements-not-met", isTimer: true, buttons: false }); return; }

    const res = await createUser(form.username, form.password, form.confirmPassword);

    if (res.success) sendAlert({ message: "alert.registration.message.success", isTimer: true, buttons: false });
    else sendAlert({ message: "alert.registration.message.fail", isTimer: true, buttons: false });

    result = res.result;
    res.result = null;
    form.username = '';
    form.password = '';
    form.confirmPassword = '';
    timeoutProceeding();
  };

  const copyText = async () => {
    if (!result || result === null) { sendAlert({ message: "alert.copy-text.fail", isTimer: true, buttons: false }); return; };

    try {
      await navigator.clipboard.writeText(result);
      sendAlert({ message: "alert.copy-text.success", isTimer: true, buttons: false });
    } catch (_) {
      sendAlert({ message: "alert.copy-text.fail", isTimer: true, buttons: false });
    }
  };

  const timeoutProceeding = () => {
    if (durationInterval !== null) return;

    const start = Date.now();

    durationInterval = setInterval(() => {
      const newDuration = Math.max(0, duration - (Date.now() - start));

      if (newDuration <= 0 && durationInterval !== null) {
        remainingDuration = 0;
        clearInterval(durationInterval);
        durationInterval = null;
      } else {
        remainingDuration = newDuration;
      }
    }, 5);
  };
</script>

{#if result !== null}
  <div id="recovery-key-modal" class="flex column">
    <div class="form-outer-container">
      <div class="flex row" style="justify-content: space-between;">
        <h2>{$t["recovery-key.modal.title"]}</h2>
        <button id="button-lang" title={$t["language.button.title"] as string} class="button-primary transparent highlight outline default-corners" type="button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>
          {$lang === 'en' ? 'FI' : 'EN'}
        </button>
      </div>
      <p>{$t["recovery-key.modal.paragraph"]}</p>
      <div id="recovery-key-container" class="flex row">
        <p style="margin: 0; font-size: 18px; user-select: text;">{result}</p>
        <button aria-label="Copy recovery key" id="copy-key-button" class="button-primary transparent highlight outline default-corners" onclick={copyText}>
          <span class="span-icon img-medium-large" style="mask-image: url('/copy.svg');"></span>
        </button>
      </div>
      <button bind:this={recoveryConfirmButton} id="recovery-modal-confirm-button" class="button-primary white-bg form" type="button" onclick={() => { result = null; setLoginView(true); }} disabled={remainingDuration > 0}>
        {$t["recovery-key.modal.confirm"]}
      </button>
    </div>
  </div>
{/if}

<div style="display: flex; flex-direction: column; gap: 40px;" in:fade={{ duration: 600, easing: cubicInOut }}>
  <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    {#each inputElements as input, i (i)}
      <div class="flex column" style="align-items: unset;">
        <p class="form-p">
          {$t[input.title]}
        </p>
        <div class="form-input-container">
          <input class="primary-input" type={i === 0 ? "text" : "password"} placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
          {#if i === 0}
            <div class="form-input-spacer"></div>
          {:else}
            <button title={$t["form.password-visibility.show"] as string} class="button-primary transparent form" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
              ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
              <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
            </button>
          {/if}
        </div>
      </div>
    {/each}
    <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
      {$t["register.button"]}
      <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
    </button>
  </form>
</div>

<style>
  #recovery-key-modal {
    position: fixed;
    z-index: 500;
    inset: 0;
    backdrop-filter: blur(24px);
    margin: 100px auto;
    user-select: none;

    #button-lang {
      width: 36px;
      font-weight: bold;
    }

    #recovery-key-container {
      margin: 0 0 1em;
      min-height: 4rem;
      height: 4rem;
      gap: 20px;
      padding: 12px;
      background-color: var(--color-secondary1);
      border-radius: 0.5rem;
      outline: 1px solid var(--outline-color1);
      justify-content: space-between;
      overflow-y: hidden;
      overflow-x: auto;
    }

    #recovery-modal-confirm-button {
      position: relative;

      &:disabled::after {
        position: absolute;
        content: '';
        bottom: 0;
        top: 0;
        left: 0;
        height: 100%;
        width: var(--progress-bar-width);
        background-color: rgba(0, 0, 0, 0.8);
        border-radius: 0;
      }
    }

    #copy-key-button {
      height: 40px;
      width: 40px;

      &:hover {
        transform: scale(1.05);
      }
    }
  }
</style>