<script lang="ts">
  import { t } from "$lib/i18n";
  import { login } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { togglePasswordVisibility } from "$lib/functions";

  type FormKey = "username" | "password";

  let {
    setLoginView,
  }: {
    setLoginView: (state: boolean) => void;
  } = $props();

  let form = $state<Record<FormKey, string>>({ username: '', password: '' });
  let isMoved = $state<boolean>(false);
  const inputElements = [
    { title: "form.username.title", key: "username" },
    { title: "form.password.title", key: "password" },
  ];

  const handleSubmit = async () => {
    const result = await login(form.username, form.password);
    form.username = '';
    form.password = '';
    if (!result.success) {
      sendAlert("alert.login.message.fail", true, false);
    }
  };
</script>

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
    <button class="primary-button form-primary-button" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>{$t["form.login.button"]}<img class:moveRight={isMoved} src="/arrow.svg" alt="nextArrow" /></button>
  </form>

  <div class="form-question-container">
    <p class="form-p">{$t["form.no-account.question"]}</p>
    <button class="form-button transparent-button" style="outline: none;" onclick={() => setLoginView(false)}>{$t["form.no-account.button"]}</button>
  </div>
</div>

<style>

</style>