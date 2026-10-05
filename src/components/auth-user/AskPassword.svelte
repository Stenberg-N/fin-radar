<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n/i18n";
  import { togglePasswordVisibility } from "$lib/user";
  import { setViewState } from "$lib/viewStore";
  import { sendAlert } from "$lib/alert";
  import { deleteUser } from "$lib/user";

  import ModalWrapper from "../ModalWrapper.svelte";

  let isMoved = $state<boolean>(false);
  let passwordInputValue = $state<string>("");

  let timeout: ReturnType<typeof setTimeout> | null = null;
  let passwordVisState = $state<boolean>(false);
  let hoverTitle = $state<{ state: boolean, element: "lang" | "eye" }>({ state: false, element: "lang" });

  let passwordInput = $state<HTMLInputElement | null>(null);
  let toggleVis = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    return () => { if (timeout) clearTimeout(timeout); };
  });

  const handleSubmit = async () => {
    if (passwordInputValue?.trim() === '') { sendAlert({ message: "alert.input-missing", isTimer: true, buttons: false }); return; }

    await deleteUser(passwordInputValue);
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

<div id="ask-password-modal" class="flex column" transition:fade={{ duration: 200, easing: cubicInOut }}>
  {#if hoverTitle.state}
    {@const content = (() => {
      switch (hoverTitle.element) {
        case "eye": return $t[`form.password-visibility.${passwordVisState === true ? 'hide' : 'show'}`];
        case "lang": return $t["language.button.title"]
      }
    })()}
    <ModalWrapper options={{
      position: { centerElement: true, moveTop: -40 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color2)'} 
      }}
    >
      <p id="ask-password-hover-title-content">
        {content}
      </p>
    </ModalWrapper>
  {/if}

  <div class="form-outer-container" transition:fly={{ y: 40, duration: 600, easing: cubicInOut }}>
    <div id="ask-password-intro" class="flex column">
      <div class="flex row">
        <button
          id="button-lang"
          class="button-primary transparent highlight outline default-corners"
          onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}
          onmouseenter={() => handleMouseEnter("lang")}
          onmouseleave={handleMouseLeave}
        >
          {$lang === 'en' ? 'FI' : 'EN'}
        </button>
        <h1>{$t["form.account-deletion.title"]}</h1>
        <button aria-label="Close modal" class="button-primary transparent highlight static" onclick={() => setViewState({ viewState: "isAskPassword", state: false })}>
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
        </button>
      </div>
      {#each ($t["form.account-deletion.message"] as string[]) as text, i (i)}
        <p class="delete-account-paragraph">
          {text}
        </p>
      {/each}
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <div class="flex column" style="align-items: unset;">
        <p class="form-p">
          {$t["password.title"]}
        </p>
        <div class="input-container-wrapper flex row">
          <div class="input-container">
            <input bind:this={passwordInput} bind:value={passwordInputValue} class="primary-input" type="password" placeholder={$t["password.title"] as string} required />
          </div>
          <button
            bind:this={toggleVis}
            aria-label="Toggle password visibility"
            class="button-primary transparent form"
            type="button"
            onclick={() => {
              const res = togglePasswordVisibility(passwordInput, toggleVis);
              if (res) passwordVisState = res.result; 
            }}
            onmouseenter={() => handleMouseEnter("eye")}
            onmouseleave={handleMouseLeave}
          >
            <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
          </button>
        </div>
      </div>

      <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
        {$t["confirm.button"]}
        <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
      </button>
    </form>
  </div>
</div>

<style>
  #ask-password-modal {
    position: fixed;
    z-index: 1001;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(24px);

    #ask-password-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
    }

    #ask-password-intro {

      > div {
        position: relative;
        justify-content: space-between;
        width: 100%;
        margin-bottom: 40px;
      }

      #button-lang {
        width: 36px;
        font-weight: bold;
      }

      h1 {
        position: absolute;
        left: 50%;
        transform: translateX(-50%);
        margin: 0;
      }
    }

    .delete-account-paragraph {
      margin: 0;
      text-align: center;
      word-wrap: break-word;
      hyphens: auto;
      user-select: none;
    }
  }
</style>