<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import { login } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { togglePasswordVisibility } from "$lib/user";
  import { HoverTitle } from "$lib/actions.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";

  type FormKey = "username" | "password";

  let form = $state<Record<FormKey, string>>({ username: '', password: '' });
  let isMoved = $state<boolean>(false);
  let passwordVisState = $state<boolean>(false);
  const hover = new HoverTitle();
  
  const inputElements = [
    { title: "username.title", key: "username" },
    { title: "password.title", key: "password" },
  ];

  let inputRefs = $state<(HTMLInputElement | null)[]>([]);
  let passwordInput = $state<HTMLInputElement | null>(null);
  let toggleVis = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    if (inputRefs[1] !== null) passwordInput = inputRefs[1];
  });

  $effect(() => {
    return () => { hover.destroy(); };
  });

  const handleSubmit = async () => {
    const result = await login(form.username, form.password);
    form.username = '';
    form.password = '';
    if (!result.success) {
      sendAlert({ message: "alert.login.message.fail", isTimer: true, buttons: false });
    }
  };
</script>

<div id="login-form-container" in:fade={{ duration: 600, easing: cubicInOut }}>
  {#if hover.isHovering}
    <ModalWrapper options={{
      position: { centerElement: true, moveTop: -40 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color2)'},
      }}
    >
      <p id="login-form-hover-title-content">
        {i18n.t[`form.password-visibility.${passwordVisState === true ? 'hide' : 'show'}`]}
      </p>
    </ModalWrapper>
  {/if}

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
              bind:this={toggleVis}
              aria-label="Toggle password visibility"
              class="button-primary transparent form"
              type="button"
              onclick={() => {
                const res = togglePasswordVisibility(passwordInput, toggleVis);
                if (res) passwordVisState = res.result; 
              }}
              onmouseenter={() => hover.enter()}
              onmouseleave={(e) => hover.leave(e)}
            >
              <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
            </button>
          {/if}
        </div>
      </div>
    {/each}
    <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
      {i18n.t["login.button"]}
      <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
    </button>
  </form>
</div>

<style>
  #login-form-container {
    display: flex;
    flex-direction: column;
    gap: 40px;

    #login-form-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
    }
  }
</style>