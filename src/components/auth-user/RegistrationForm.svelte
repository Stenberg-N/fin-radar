<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { sendAlert } from "$lib/alert";
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { createUser } from "$lib/user";
  import { validatePassword, togglePasswordVisibility } from "$lib/user";
  import { HoverTitle } from "$lib/actions.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";

  type FormKey = "username" | "password" | "confirmPassword";
  type HoverTarget = { element: "lang" | "eye"; idx: 0 | 1 };

  let {
    setLoginView,
  }: {
    setLoginView: (state: boolean) => void;
  } = $props();

  let form = $state<Record<FormKey, string>>({ username: '', password: '', confirmPassword: '' });
  let result = $state<string | null>(null);

  let isMoved = $state<boolean>(false);
  const hover = new HoverTitle<HoverTarget>();
  let passwordVisState = $state<boolean[]>([false, false]);

  const duration = 4000;
  let remainingDuration = $state(duration);
  let durationInterval: ReturnType<typeof setInterval> | null = null;

  const inputElements = [
    { title: "username.title", key: "username"},
    { title: "password.title", key: "password"},
    { title: "confirm-password.title", key: "confirmPassword"},
  ];

  let inputRefs = $state<(HTMLInputElement | null)[]>([]);
  let buttonRefs = $state<(HTMLButtonElement | null)[]>([]);
  let recoveryConfirmButton = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    if (!result || !recoveryConfirmButton) return;

    const progress = `${((duration - remainingDuration) / duration) * 100}%`;
    recoveryConfirmButton.style.setProperty('--progress-bar-width', progress);
  });

  $effect(() => {
    return () => {
      hover.destroy();
    };
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

{#if hover.isHovering}
  {@const content = (() => {
    switch (hover.target?.element) {
      case "eye": return i18n.t[`form.password-visibility.${passwordVisState[hover.target?.idx] === true ? 'hide' : 'show'}`];
      case "lang": return i18n.t["language.button.title"]
    }
  })()}
  <ModalWrapper
    attributes={{ "hover-title-owner": hover.id }}
    options={{
      position: hover.target?.element === "eye" ? { centerElement: true, moveTop: -40 } : { moveTop: -30 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color2)'},
    }}
  >
    <p id="registration-form-hover-title-content">
      {content}
    </p>
  </ModalWrapper>
{/if}

{#if result !== null}
  <div id="recovery-key-modal" class="flex column">
    <div class="form-outer-container">
      <div class="flex row" style="justify-content: space-between;">
        <h2>{i18n.t["recovery-key.modal.title"]}</h2>
        <button
          id="button-lang"
          class="button-primary transparent highlight outline default-corners"
          type="button"
          onclick={() => i18n.lang = i18n.lang === 'en' ? 'fi' : 'en'}
          onmouseenter={() => hover.enter({ element: "lang", idx: 0 })}
          onmouseleave={(e) => hover.leave(e)}
        >
          {i18n.lang.toUpperCase()}
        </button>
      </div>
      <p>{i18n.t["recovery-key.modal.paragraph"]}</p>
      <div id="recovery-key-container" class="flex row">
        <p style="margin: 0; font-size: 18px; user-select: text;">{result}</p>
        <button aria-label="Copy recovery key" id="copy-key-button" class="button-primary transparent highlight outline default-corners" onclick={copyText}>
          <span class="span-icon img-medium-large" style="mask-image: url('/copy.svg');"></span>
        </button>
      </div>
      <button bind:this={recoveryConfirmButton} id="recovery-modal-confirm-button" class="button-primary white-bg form" type="button" onclick={() => { result = null; setLoginView(true); }} disabled={remainingDuration > 0}>
        {i18n.t["recovery-key.modal.confirm"]}
      </button>
    </div>
  </div>
{/if}

<div id="registration-form-container" in:fade={{ duration: 600, easing: cubicInOut }}>
  <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    {#each inputElements as input, i (i)}
      <div class="flex column" style="align-items: unset;">
        <p class="form-p">
          {i18n.t[input.title]}
        </p>
        <div class="input-container-wrapper flex row">
          <div class="input-container">
            <input bind:this={inputRefs[i]} class="primary-input" type={i === 0 ? "text" : "password"} placeholder={i18n.t[input.title] as string} bind:value={form[input.key as FormKey]} required />
          </div>
          {#if i === 0}
            <div class="form-input-spacer"></div>
          {:else}
            <button
              bind:this={buttonRefs[i]}
              aria-label="Toggle password visibility"
              class="button-primary transparent form"
              type="button"
              onclick={() => {
                const res = togglePasswordVisibility(inputRefs[i], buttonRefs[i]);
                if (res) passwordVisState[i] = res.result;
              }}
              onmouseenter={() => hover.enter({ element: "eye", idx: i as 0 | 1 })}
              onmouseleave={(e) => hover.leave(e)}
            >
              <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
            </button>
          {/if}
        </div>
      </div>
    {/each}
    <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
      {i18n.t["register.button"]}
      <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
    </button>
  </form>
</div>

<style>
  #registration-form-container {
    display: flex;
    flex-direction: column;
    gap: 40px;
  }

  #registration-form-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary2);
  }

  #recovery-key-modal {
    position: fixed;
    z-index: 500;
    inset: 0;
    backdrop-filter: blur(24px);
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