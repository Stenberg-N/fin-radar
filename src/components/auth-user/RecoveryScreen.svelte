<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { sendAlert } from "$lib/alert";
  import { lang, t } from "$lib/i18n/i18n";
  import { recoverPassword } from "$lib/user";
  import { setViewState } from "$lib/viewStore";
  import { togglePasswordVisibility } from "$lib/user";

  import ModalWrapper from "../ModalWrapper.svelte";

  type FormKey = "accountName" | "recoveryKey";

  let form = $state<Record<FormKey, string>>({ accountName: '', recoveryKey: '' });
  let isMoved = $state<boolean>(false);

  let hoverTitle = $state<{ state: boolean, element: "lang" | "eye" }>({ state: false, element: "lang" });
  let recoveryKeyVisState = $state<boolean>(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;

  const inputElements = [
    { title: "username.title", key: "accountName" },
    { title: "form.forgot-password.recovery-key.title", key: "recoveryKey" },
  ];

  let inputRefs = $state<(HTMLInputElement | null)[]>([]);
  let recoveryKeyInput = $state<HTMLInputElement | null>(null);
  let toggleVis = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    if (inputRefs[1] !== null) recoveryKeyInput = inputRefs[1];
  });

  $effect(() => {
    return () => { if (timeout) clearTimeout(timeout); };
  });
  
  const handleSubmit = async () => {
    if (form.accountName.trim() === '' || form.recoveryKey.trim() === '') { sendAlert({ message: "alert.input-missing", isTimer: true, buttons: false }); return; };

    const result = await recoverPassword(form.accountName, form.recoveryKey);
    setViewState({ viewState: "isRecoveryView", state: false });
    if (!result.success) {
      sendAlert({ message: "alert.password.recover.fail", isTimer: true, buttons: false });
    }
    form.accountName = '';
    form.recoveryKey = '';
  };

  const handleMouseEnter = (el: "lang" | "eye") => {
    if (timeout) clearTimeout(timeout);
    hoverTitle.element = el;

    timeout = setTimeout(() => {
      hoverTitle.state = true;
    }, 300);
  };

  const handleMouseLeave = () => {
    if (timeout) clearTimeout(timeout);
    timeout = null;

    hoverTitle.state = false;
  };
</script>

<div id="recover-account-container" class="flex column" transition:fade={{ duration: 200, easing: cubicInOut }}>
  {#if hoverTitle.state}
    {@const content = (() => {
      switch (hoverTitle.element) {
        case "eye": return $t[`form.password-visibility.${recoveryKeyVisState === true ? 'hide' : 'show'}`];
        case "lang": return $t["language.button.title"]
      }
    })()}
    <ModalWrapper options={{
      position: hoverTitle.element === "eye" ? { centerElement: true, moveTop: -40 } : { moveTop: -30 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color2)'} 
      }}
    >
      <p id="account-recovery-hover-title-content">
        {content}
      </p>
    </ModalWrapper>
  {/if}

  <div class="form-outer-container" style="pointer-events: auto;" transition:fly={{ y: 40, duration: 600, easing: cubicInOut }}>
    <div class="flex column">
      <div class="flex row" style="justify-content: space-between; width: 100%;">
        <button
          id="button-lang"
          class="button-primary transparent highlight outline default-corners"
          type="button"
          onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}
          onmouseenter={() => handleMouseEnter("lang")}
          onmouseleave={handleMouseLeave}
        >
          {$lang === 'en' ? 'FI' : 'EN'}
        </button>
        <button aria-label="Close recovery screen" class="button-primary transparent highlight static" type="button" onclick={() => setViewState({ viewState: "isRecoveryView", state: false })}>
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
        </button>
      </div>
      <h2>
        {$t["forgot-password.title"]}
      </h2>
      <p>
        {$t["forgot-password.paragraph"]}
      </p>
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#each inputElements as input, i (i)}
        <div class="flex column" style="align-items: unset;">
          <p class="form-p">
            {$t[input.title]}
          </p>
          <div class="input-container-wrapper flex row">
            <div class="input-container">
              <input bind:this={inputRefs[i]} class="primary-input" type={i === 0 ? "text" : "password"} placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
            </div>
            {#if i === 0}
              <div class="form-input-spacer"></div>
            {:else}
              <button
                bind:this={toggleVis}
                aria-label="Toggle recovery key visibility"
                class="button-primary transparent form"
                type="button"
                onclick={() => {
                  const res = togglePasswordVisibility(recoveryKeyInput, toggleVis);
                  if (res) recoveryKeyVisState = res.result; 
                }}
                onmouseenter={() => handleMouseEnter("eye")}
                onmouseleave={handleMouseLeave}
              >
                <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
              </button>
            {/if}
          </div>
        </div>
      {/each}
      <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
        {$t["confirm.button"]}
        <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
      </button>
    </form>
  </div>
</div>

<style>
  #recover-account-container {
    position: fixed;
    z-index: 500;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(24px);
    padding: 100px 0;
    pointer-events: none;

    #account-recovery-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
    }

    #button-lang {
      width: 36px;
      font-weight: bold;
    }
  }
</style>