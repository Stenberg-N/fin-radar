<script lang="ts">
  import { t } from "$lib/i18n";
  import { login } from "$lib/user";
  import { sendAlert } from "$lib/alert";

  type FormKey = "username" | "password";

  let {
    setLoginView,
  }: {
    setLoginView: (state: boolean) => void;
  } = $props();

  let form = $state<Record<FormKey, string>>({ username: '', password: '' });

  const inputElements = [
    { title: "form.username.title", key: "username" },
    { title: "form.password.title", key: "password" },
  ];

  let isMoved = $state<boolean>(false);

  const handleSubmit = async () => {
    const result = await login(form.username, form.password);
    form.username = '';
    form.password = '';
    if (!result.success) {
      sendAlert("alert.login.message.fail", true, false);
    }
  };

  const togglePasswordVisibility = (button: EventTarget | null) => {
    if (!button) return;

    const node = button as HTMLButtonElement;
    const passwordInput = node.previousElementSibling as HTMLInputElement | null;
    const img = node.firstChild as HTMLImageElement | null;

    if (passwordInput && img) {
      const isPassword = passwordInput.type === "password";
      passwordInput.type = isPassword ? "text" : "password";
      img.src = isPassword ? "/eye-hidden.svg" : "/eye-visible.svg";
      node.title = isPassword ? String($t["form.password-visibility.hide"]) : String($t["form.password-visibility.show"]);
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
            <button title={$t["form.password-visibility.show"] as string} class="form-button transparent-button" type="button" onclick={(e) => togglePasswordVisibility(e.target)}><img src="/eye-visible.svg" alt="Eye icon" /></button>
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