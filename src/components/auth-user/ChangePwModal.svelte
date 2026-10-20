<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import { resetPassword } from "$lib/user";
  import { togglePasswordVisibility } from "$lib/user";

  import ModalWrapper from "../ModalWrapper.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";

  type FormKey = "currentPassword" | "newPassword" | "confirmNewPassword";
  type HoverTarget = { element: "lang" | "eye"; idx: 0 | 1 | 2 };

  interface Props {
    /** Used to set the component to recovery context, which hides the close button. Defaults to `false` */
    isRecovery?: boolean;
    /** Used to hide or show the translations toggle button. Defaults to `true` */
    isTranslationButtonVisible?: boolean;
    theme?: "dark" | "light" | "transparent" | "lighter-dark" | "lighter-dark1-a";
    /** Used to toggle the `box-shadow` property. Defaults to `true`. */
    isBoxShadow?: boolean;
    /** Used to set lower padding to the component. */
    isLowerPadding?: boolean;
    /** Used to select where the component title/header gets positioned. Defaults to `center`. */
    justifyHeader?: "right" | "left" | "center";
    /** Used to select where the form's content is positioned. Defaults to `unset`. */
    justifyForm?: "right" | "left";
    /** Determines what transitions get enabled. Use `fade` for fading the parent element in and out, or `true` to also enable a fly transition to the form. */
    enableTransitions?: "fade" | true;
    /** Used to disable the `max-width` constraint. If omitted, max width is set to 800px. */
    disableMaxWidth?: boolean;
  }

  let {
    isRecovery = false,
    isTranslationButtonVisible = true,
    theme = "dark",
    isBoxShadow = true,
    isLowerPadding = false,
    justifyHeader = "center",
    justifyForm,
    enableTransitions,
    disableMaxWidth,
  }: Props = $props();

  const flyTransition = (node: HTMLElement) => enableTransitions ? fly(node, { y: 40, duration: 600, easing: cubicInOut }) : {};
  const fadeTransition = (node: HTMLElement) => enableTransitions === true || enableTransitions === "fade" ? fade(node, { duration: 200, easing: cubicInOut }) : {};
  const maxWidth = $derived(disableMaxWidth === true ? 'unset' : '800px');
  const justifyFormOption = $derived.by(() => {
    const pad = isLowerPadding ? 1 : 2;
    switch (justifyForm) {
      case "left": return `padding: ${pad}rem 0 ${pad}rem 2px; align-items: flex-start;`;
      case "right": return `padding: ${pad}rem 2px ${pad}rem 0; align-items: flex-end;`;
      default: return `padding: ${pad}rem; align-items: unset;`;
    }
  });

  let form = $state<Record<FormKey, string>>({ currentPassword: '', newPassword: '', confirmNewPassword: '' });
  let isMoved = $state<boolean>(false);
  const hover = new HoverTitle<HoverTarget>();
  let passwordVisState = $state<boolean[]>([false, false, false]);

  const themes = {
    dark: {
      bg: "var(--color-primary2)",
      text: "var(--color-white-primary1)",
      outline: "var(--outline-color1)",
      button: "button-primary transparent highlight outline default-corners",
      submit: "button-primary white-bg",
      img: "var(--color-white-primary1)",
      "pw-change-bg": "var(--hover-color-transparent)",
    },
    "lighter-dark": {
      bg: "var(--color-secondary1)",
      text: "var(--color-white-primary1)",
      outline: "var(--outline-color1)",
      button: "button-primary transparent highlight outline default-corners",
      submit: "button-primary white-bg",
      img: "var(--color-white-primary1)",
      "pw-change-bg": "var(--hover-color-transparent)",
    },
    "lighter-dark1-a": {
      bg: "var(--color-secondary1-a)",
      text: "var(--color-white-primary1)",
      outline: "var(--outline-color1)",
      button: "button-primary transparent highlight outline default-corners",
      submit: "button-primary white-bg",
      img: "var(--color-white-primary1)",
      "pw-change-bg": "var(--hover-color-transparent)",
    },
    transparent: {
      bg: "transparent",
      text: "var(--color-white-primary1)",
      outline: "var(--outline-color1)",
      button: "button-primary transparent highlight outline default-corners",
      submit: "button-primary white-bg",
      img: "var(--color-white-primary1)",
      "pw-change-bg": "var(--hover-color-transparent)",
    },
    light: {
      bg: "var(--color-primary3)",
      text: "black",
      outline: "var(--outline-color-white)",
      button: "button-primary dark",
      submit: "button-primary dark",
      img: "black",
      "pw-change-bg": "var(--hover-color-transparent-white)",
    },
  } as const;
  const t = $derived(themes[theme] ?? themes.dark);
  
  const inputElements = [
    { get title() { return i18n.t["change-password.current-password.title"] as string; }, key: "currentPassword"},
    { get title() { return i18n.t["change-password.new-password.title"] as string; }, key: "newPassword" },
    { get title() { return i18n.t["change-password.confirm-new-password.title"] as string; }, key: "confirmNewPassword" },
  ];

  let inputRefs = $state<(HTMLInputElement | null)[]>([]);
  let buttonRefs = $state<(HTMLButtonElement | null)[]>([]);

  $effect(() => {
    const pwOverlay = document.getElementById("change-pw-overlay");
    if (pwOverlay) {
      isRecovery ? pwOverlay.style.backgroundColor = "#0f0f0f" : pwOverlay.style.backdropFilter = "blur(24px)";
    }
  });

  $effect(() => {
    return () => { hover.destroy(); };
  });

  const handleSubmit = async () => {

    const result = await resetPassword(isRecovery, form.newPassword, form.confirmNewPassword, isRecovery ? undefined : form.currentPassword);

    if (!result.success) return;

    form.currentPassword = '';
    form.newPassword = '';
    form.confirmNewPassword = '';
  };

</script>

<div
  id="change-pw-container"
  class="flex column"
  style="max-width: {maxWidth};"
  style:--change-pw-transparent-button-bg-color={t["pw-change-bg"]}
  transition:fadeTransition>
  {#if hover.isHovering}
    {@const content = (() => {
      switch (hover.target?.element) {
        case "eye": return i18n.t[`form.password-visibility.${passwordVisState[hover.target?.idx] === true ? 'hide' : 'show'}`];
        case "lang": return i18n.t["language.button.title"]
      }
    })()}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={hover.target?.element === "eye" ? { centerElement: true, moveTop: -40 } : { moveTop: -30 }}
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      borderRadius={8}
      outline={{ width: 1, color: 'var(--outline-color2)'}}
    >
      <p id="change-pw-hover-title-content">
        {content}
      </p>
    </ModalWrapper>
  {/if}

  {#if isRecovery}
    <div id="cancel-recovery-paragraph-container" class="flex column" transition:fly={{ y: -40, duration: 600, easing: cubicInOut }}>
      {#each (i18n.t["change-password.cancel-recovery.message"] as string[]) as text, i (i)}
        <p class="cancel-recovery-paragraph" style="color: {i === 0 ? "var(--color-highlight1)" : "var(--color-white-primary1)"}; font-weight: {i === 0 ? 800 : 400};">
          {text}
        </p>
      {/each}
    </div>
  {/if}
  <div class="form-outer-container" transition:flyTransition
    style="
      gap: {isLowerPadding ? '1rem' : '40px'};;
      background-color: {t.bg};
      box-shadow: {isBoxShadow ? '0 4px 8px rgba(0, 0, 0, 0.8)' : 'unset'};
      padding: {isLowerPadding ? '1rem' : '40px'};
      max-width: {maxWidth};
    "
  >
    <div id="change-pw-header-container" class="flex row">
      {#if isTranslationButtonVisible}
        <button
          class={t.button}
          onclick={() => i18n.lang = i18n.lang === 'en' ? 'fi' : 'en'}
          onmouseenter={() => hover.enter({ element: "lang", idx: 0 })}
          onmouseleave={(e) => hover.leave(e)}
        >
          {i18n.lang.toUpperCase()}
        </button>
      {/if}
      <h1
        style="
          color: {t.text};
          max-width: {isTranslationButtonVisible ? 'calc(100% - 120px)' : ''};
          text-align: {justifyHeader};
        "
      >
        {i18n.t["change-password.title"]}
      </h1>
    </div>
    <form class="form-bg" style="{justifyFormOption}" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#each isRecovery ? inputElements.slice(1, 3) : inputElements  as input, i (input.key)}
        <div class="flex column" style="align-items: unset; width: 100%;">
          <p class="form-p" style="color: {t.text};">
            {input.title}
          </p>
          <div class="input-container-wrapper flex row">
            <div class="input-container" style="outline: 2px solid {t.outline};">
              <input bind:this={inputRefs[i]} class="primary-input" style="color: {t.text};" type="password" placeholder={input.title} bind:value={form[input.key as FormKey]} />
            </div>
            <button
              bind:this={buttonRefs[i]}
              aria-label="Toggle password visibility"
              class="button-primary transparent form"
              type="button"
              onclick={() => {
                const res = togglePasswordVisibility(inputRefs[i], buttonRefs[i]);
                if (res) passwordVisState[i] = res.result;
              }}
              onmouseenter={() => hover.enter({ element: "eye", idx: i as 0 | 1 | 2 })}
              onmouseleave={(e) => hover.leave(e)}
            >
              <span class="span-icon" style="mask-image: url('/eye-visible.svg'); background-color: {t.img};"></span>
            </button>
          </div>
        </div>
      {/each}
      <button class="{t.submit} form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
        {i18n.t["confirm.button"]}
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

    #change-pw-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
    }

    button.button-primary.transparent:hover {
      background-color: var(--change-pw-transparent-button-bg-color);
    }

    .input-container:focus-within {
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