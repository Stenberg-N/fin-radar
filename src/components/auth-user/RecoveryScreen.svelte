<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { sendAlert } from "$lib/alert";
  import { lang, t } from "$lib/i18n/i18n";
  import { recoverPassword } from "$lib/user";
  import { setViewState } from "$lib/viewStore";
  import { togglePasswordVisibility } from "$lib/user";

  type FormKey = "accountName" | "recoveryKey";

  let form = $state<Record<FormKey, string>>({ accountName: '', recoveryKey: '' });
  let isMoved = $state<boolean>(false);
  const inputElements = [
    { title: "form.username.title", key: "accountName" },
    { title: "form.forgot-password.recovery-key.title", key: "recoveryKey" },
  ];
  
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
</script>

<div class="vertical-flex-container" style="position: fixed; z-index: 500; inset: 0; backdrop-filter: blur(48px); padding: 100px 0; pointer-events: none;" transition:fade={{ duration: 200, easing: cubicInOut }}>
  <div class="form-outer-container" style="pointer-events: auto;" transition:fly={{ y: 40, duration: 600, easing: cubicInOut }}>
    <div class="vertical-flex-container">
      <div class="horizontal-flex-container" style="justify-content: space-between; width: 100%;">
        <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button-dark" type="button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
        <button aria-label="Close recovery screen" class="transparent-button-highlight" style="width: 32px; height: 32px;" type="button" onclick={() => setViewState({ viewState: "isRecoveryView", state: false })}>
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg'); background-color: black;"></span>
        </button>
      </div>
      <h2>{$t["form.forgot-password.title"]}</h2>
      <p>{$t["form.forgot-password.paragraph"]}</p>
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      {#each inputElements as input, i (i)}
        <div class="vertical-flex-container" style="align-items: unset;">
          <p class="form-p">{$t[input.title]}</p>
          <div class="form-input-container">
            <input class="primary-input" style="color: black;" type={i === 0 ? "text" : "password"} placeholder={$t[input.title] as string} bind:value={form[input.key as FormKey]} required />
            {#if i === 0}
              <div class="form-input-spacer"></div>
            {:else}
              <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
                ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
                <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
              </button>
            {/if}
          </div>
        </div>
      {/each}
      <button class="primary-button-dark form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
        {$t["confirm.button"]}
        <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
      </button>
    </form>
  </div>
</div>

<style>
  .transparent-button-highlight:hover {
    background-color: rgba(0, 0, 0, 0.2);
  }
</style>