<script lang="ts">
  import { sendAlert } from "$lib/alert";
  import { lang, t } from "$lib/i18n";
  import { recoverPassword } from "$lib/user";
  import { setViewState } from "$lib/viewStore";
  import { togglePasswordVisibility } from "$lib/functions";

  type FormKey = "accountName" | "recoveryKey";

  let form = $state<Record<FormKey, string>>({ accountName: '', recoveryKey: '' });
  let isMoved = $state<boolean>(false);
  const inputElements = [
    { title: "form.username.title", key: "accountName" },
    { title: "form.forgot-password.recovery-key.title", key: "recoveryKey" },
  ];
  
  const handleSubmit = async () => {
    if (form.accountName.trim() === '' || form.recoveryKey.trim() === '') { sendAlert("alert.password.recover.missing-info", true, false); return; };

    const result = await recoverPassword(form.accountName, form.recoveryKey);
    setViewState("isRecoveryView", false);
    if (!result.success) {
      sendAlert("alert.password.recover.fail", true, false);
    }
    form.accountName = '';
    form.recoveryKey = '';
  };
</script>

<div class="vertical-flex-container" style="position: fixed; z-index: 500; inset: 0; backdrop-filter: blur(48px); padding: 100px 0; pointer-events: none;">
  <div class="form-outer-container" style="pointer-events: auto;">
    <div class="vertical-flex-container">
      <div class="horizontal-flex-container" style="justify-content: space-between; width: 100%;">
        <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button" type="button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
        <button class="transparent-button-highlight" style="width: 32px; height: 32px;" type="button" onclick={() => setViewState("isRecoveryView", false)}><img src="close-x.svg" alt="Close" class="img-small" style="filter: brightness(0);" /></button>
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
              <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
                ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
                <img src="/eye-visible.svg" alt="Eye icon" />
              </button>
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