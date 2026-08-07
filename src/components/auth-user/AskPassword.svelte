<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n";
  import { togglePasswordVisibility } from "$lib/user";
  import { setViewState } from "$lib/viewStore";
  import { sendAlert } from "$lib/alert";
  import { deleteUser } from "$lib/user";

  let isMoved = $state<boolean>(false);
  let passwordInput = $state<string>("");

  const handleSubmit = async () => {
    if (passwordInput?.trim() === '') { sendAlert({ message: "alert.input-missing", isTimer: true, buttons: false }); return; }

    await deleteUser(passwordInput);
  };
</script>

<div id="ask-password-modal" class="vertical-flex-container" transition:fade={{ duration: 200, easing: cubicInOut }}>
  <div class="form-outer-container" transition:fly={{ y: 40, duration: 600, easing: cubicInOut }}>
    <div class="vertical-flex-container">
      <div class="horizontal-flex-container" style="position: relative; justify-content: space-between; width: 100%; margin-bottom: 40px;">
        <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button-dark" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
        <h1 style="position: absolute; left: 50%; transform: translateX(-50%); margin: 0;">{$t["form.account-deletion.title"]}</h1>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState({ viewState: "isAskPassword", state: false })}><img src="close-x.svg" alt="Close" class="img-small" style="filter: brightness(0);" /></button>
      </div>
      {#each $t["form.account-deletion.message"] as text, i (i)}
        <p class="delete-account-paragraph">{text}</p>
      {/each}
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <div class="vertical-flex-container" style="align-items: unset;">
        <p class="form-p">{$t["form.password.title"]}</p>
        <div class="form-input-container">
          <input class="primary-input" type="password" placeholder={$t["form.password.title"] as string} bind:value={passwordInput} required />
          <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
            ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
            <img src="/eye-visible.svg" alt="Eye icon" />
          </button>
        </div>
      </div>

      <button class="primary-button-dark form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["confirm.button"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
    </form>
  </div>
</div>

<style>
  #ask-password-modal {
    position: fixed;
    z-index: 500;
    inset: 0;
    backdrop-filter: blur(48px);
  }

  .transparent-button-highlight:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }

  .delete-account-paragraph {
    margin: 0;
    text-align: center;
    word-wrap: break-word;
    hyphens: auto;
    user-select: none;
  }
</style>