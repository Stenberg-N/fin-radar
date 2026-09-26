<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n/i18n";
  import { resetPassword } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { validatePassword, togglePasswordVisibility } from "$lib/user";
  import { onMount } from "svelte";

  let {
    options,
  }: {
    options?: {
      isRecovery?: boolean;
      isTranslationButtonVisible?: boolean;
      theme?: "dark" | "light" | "transparent" | "lighter-dark" | "lighter-dark1-a";
      isBoxShadow?: boolean;
      isLowerPadding?: boolean;
      justifyHeader?: "right" | "left" | "center";
      justifyForm?: "right" | "left";
      /** Determines what transitions get enabled. Use `fade` for fading the parent element in and out, or `true` to enable a fly transition to the form. */
      enableTransitions?: "fade" | true;
      disableMaxWidth?: boolean;
    }
  } = $props();

  type FormKey = "currentPassword" | "newPassword" | "confirmNewPassword";

  let form = $state<Record<FormKey, string>>({ currentPassword: '', newPassword: '', confirmNewPassword: '' });
  let isMoved = $state<boolean>(false);

  const isRecovery = $derived(options?.isRecovery ? options.isRecovery : false);
  const isTranslationButtonVisible = $derived(options?.isTranslationButtonVisible !== undefined ? options.isTranslationButtonVisible : true);
  const isLowerPadding = $derived(options?.isLowerPadding !== undefined ? options.isLowerPadding : false);
  const textColor = $derived(options?.theme !== undefined ? (["dark", "lighter-dark", "lighter-dark1-a", "transparent"].includes(options.theme) ? 'var(--color-white-primary1)' : 'black') : 'black');
  const imgColor = $derived(options?.theme !== undefined ? (["dark", "lighter-dark", "lighter-dark1-a", "transparent"].includes(options.theme) ? 'var(--color-white-primary1)' : 'black') : 'black');
  const outlineColor = $derived(options?.theme !== undefined ? (["dark", "lighter-dark", "lighter-dark1-a", "transparent"].includes(options.theme) ? 'var(--outline-color1)' : 'var(--outline-color-white)') : 'var(--outline-color-white)');
  const buttonStyle = $derived(options?.theme !== undefined ? (["dark", "lighter-dark", "lighter-dark1-a", "transparent"].includes(options.theme) ? 'button-primary transparent highlight outline default-corners' : 'button-primary dark') : 'button-primary dark');
  const submitButtonStyle = $derived(options?.theme !== undefined ? (["dark", "lighter-dark", "lighter-dark1-a", "transparent"].includes(options.theme) ? 'button-primary white-bg' : 'button-primary dark') : 'button-primary dark');
  const justifyHeader = $derived(options?.justifyHeader !== undefined ? options.justifyHeader : "center");
  const flyTransition = (node: HTMLElement) => { return options?.enableTransitions === true ? fly(node, { y: 40, duration: 600, easing: cubicInOut }) : {} };
  const fadeTransition = (node: HTMLElement) => { return options?.enableTransitions ? ( ["fade", true].includes(options.enableTransitions) ? fade(node, { duration: 200, easing: cubicInOut }) : {} ) : {} };
  const maxWidth = $derived(options?.disableMaxWidth === true ? 'unset' : '800px');
  const justifyForm = $derived.by(() => {
    switch (options?.justifyForm) {
      case "left": return `padding: ${isLowerPadding ? '1rem 0 1rem 2px' : '2rem 0 2rem 2px'}; align-items: flex-start;`;
      case "right": return `padding: ${isLowerPadding ? '1rem 2px 1rem 0' : '2rem 2px 2rem 0'}; align-items: flex-end;`;
      default: return `padding: ${isLowerPadding ? '1rem' : '2rem'}; align-items: unset;`;
    }
  });
  const backgroundColor = $derived.by(() => {
    switch (options?.theme) {
      case "dark": return "var(--color-primary2)";
      case "light": return "var(--color-primary3)";
      case "transparent": return "transparent";
      case "lighter-dark": return "var(--color-secondary1)";
      case "lighter-dark1-a": return "var(--color-secondary1-a)";
      default: return "var(--color-primary2)";
    }
  });
  
  const inputElements = [
    { title: "change-password.current-password.title", key: "currentPassword"},
    { title: "change-password.new-password.title", key: "newPassword" },
    { title: "change-password.confirm-new-password.title", key: "confirmNewPassword" },
  ];

  onMount(() => {
    document.documentElement.style.setProperty('--change-pw-transparent-button-bg-color', options?.theme !== undefined
      ? (["dark", "lighter-dark", "lighter-dark1-a", "transparent"].includes(options.theme)
        ? 'var(--hover-color-transparent)'
        : 'var(--hover-color-transparent-white)')
      : 'var(--hover-color-transparent-white)');
  });

  $effect(() => {
    const pwOverlay = document.getElementById("change-pw-overlay");
    if (pwOverlay) {
      options?.isRecovery ? pwOverlay.style.backgroundColor = "#0f0f0f" : pwOverlay.style.backdropFilter = "blur(24px)";
    }
  });

  const handleSubmit = async () => {
    if (form.newPassword !== form.confirmNewPassword) { sendAlert({ message: "alert.password.mismatch", isTimer: true, buttons: false}); return; };
    if (!validatePassword(form.newPassword).isValid) { sendAlert({ message: "alert.password.requirements-not-met", isTimer: true, buttons: false }); return; };

    const result = await resetPassword(isRecovery, form.newPassword, form.confirmNewPassword, isRecovery ? undefined : form.currentPassword);

    if (!result.success) {
      sendAlert({ message: "alert.password.change.fail", isTimer: true, buttons: false });
      form.newPassword = '';
      form.confirmNewPassword = '';
      return;
    }

    sendAlert({ message: "alert.password.change.success", isTimer: true, buttons: false });
    form.currentPassword = '';
    form.newPassword = '';
    form.confirmNewPassword = '';
  };

</script>

<div id="change-pw-container" class="flex column" style="max-width: {maxWidth};" transition:fadeTransition>
  {#if isRecovery}
    <div id="cancel-recovery-paragraph-container" class="flex column" transition:fly={{ y: -40, duration: 600, easing: cubicInOut }}>
      {#each ($t["change-password.cancel-recovery.message"] as string[]) as text, i (i)}
        <p class="cancel-recovery-paragraph" style="color: {i === 0 ? "var(--color-highlight1)" : "var(--color-white-primary1)"}; font-weight: {i === 0 ? 800 : 400};">
          {text}
        </p>
      {/each}
    </div>
  {/if}
  <div class="form-outer-container" transition:flyTransition
    style="
      gap: {isLowerPadding ? '1rem' : '40px'};;
      background-color: {backgroundColor};
      box-shadow: {options?.isBoxShadow === false ? 'unset' : '0 4px 8px rgba(0, 0, 0, 0.8)'};
      padding: {isLowerPadding ? '1rem' : '40px'};
      max-width: {maxWidth};
    "
  >
    <div id="change-pw-header-container" class="flex row">
      {#if isTranslationButtonVisible}
        <button title={$t["language.button.title"] as string} class={buttonStyle}
          onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}
        >
          {$lang === 'en' ? 'FI' : 'EN'}
        </button>
      {/if}
      <h1
        style="
          color: {textColor};
          max-width: {isTranslationButtonVisible ? 'calc(100% - 120px)' : ''};
          text-align: {justifyHeader};
        "
      >
        {$t["change-password.title"]}
      </h1>
    </div>
    <form class="form-bg" style="{justifyForm !== undefined ? justifyForm : `padding: ${isLowerPadding ? '1rem' : '2rem'};`}" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#each isRecovery ? inputElements.slice(1, 3) : inputElements  as input, i (i)}
        <div class="flex column" style="align-items: unset; width: 100%;">
          <p class="form-p" style="color: {textColor};">
            {$t[input.title]}
          </p>
          <div class="form-input-container">
            <input class="primary-input" style="color: {textColor}; outline: 2px solid {outlineColor};" type="password" placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
            <button title={$t["form.password-visibility.show"] as string} class="button-primary transparent form" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
              ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
              <span class="span-icon" style="mask-image: url('/eye-visible.svg'); background-color: {imgColor};"></span>
            </button>
          </div>
        </div>
      {/each}
      <button class="{submitButtonStyle} form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
        {$t["confirm.button"]}
        <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
      </button>
    </form>
  </div>
</div>

<style>
  #change-pw-container {
    max-width: 800px;
    width: 100%;
    min-width: fit-content;

    button.button-primary.transparent:hover {
      background-color: var(--change-pw-transparent-button-bg-color);
    }

    .primary-input:focus {
      outline-color: var(--color-highlight1) !important;
    }
  }

  #change-pw-header-container {
    justify-content: unset;
    min-height: 2rem;
    gap: 2rem;

    button {
      justify-self: flex-end;
      width: 36px;
      height: 2rem;
      font-weight: bold;
    }

    h1 {
      flex: 1;
      text-align: center;
      margin: 0;
    }
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
</style>