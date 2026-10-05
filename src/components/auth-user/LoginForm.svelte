<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t } from "$lib/i18n/i18n";
  import { login } from "$lib/user";
  import { sendAlert } from "$lib/alert";
  import { togglePasswordVisibility } from "$lib/user";

  import ModalWrapper from "../ModalWrapper.svelte";

  type FormKey = "username" | "password";

  let form = $state<Record<FormKey, string>>({ username: '', password: '' });
  let isMoved = $state<boolean>(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let passwordVisState = $state<boolean>(false);
  let isHovering = $state<boolean>(false);
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
    return () => { if (timeout) clearTimeout(timeout); };
  });

  const handleSubmit = async () => {
    const result = await login(form.username, form.password);
    form.username = '';
    form.password = '';
    if (!result.success) {
      sendAlert({ message: "alert.login.message.fail", isTimer: true, buttons: false });
    }
  };

  const handleMouseEnter = () => {
    if (timeout) clearTimeout(timeout);

    timeout = setTimeout(() => {
      isHovering = true;
    }, 300);
  };

  const handleMouseLeave = () => {
    if (timeout) clearTimeout(timeout);
    timeout = null;

    isHovering = false;
  };
</script>

<div id="login-form-container" in:fade={{ duration: 600, easing: cubicInOut }}>
  {#if isHovering}
    <ModalWrapper options={{
      position: { centerElement: true, moveTop: -40 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color2)'} 
      }}
    >
      <p id="login-form-hover-title-content">
        {$t[`form.password-visibility.${passwordVisState === true ? 'hide' : 'show'}`]}
      </p>
    </ModalWrapper>
  {/if}

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
              aria-label="Toggle password visibility"
              class="button-primary transparent form"
              type="button"
              onclick={() => {
                const res = togglePasswordVisibility(passwordInput, toggleVis);
                if (res) passwordVisState = res.result; 
              }}
              onmouseenter={handleMouseEnter}
              onmouseleave={handleMouseLeave}
            >
              <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
            </button>
          {/if}
        </div>
      </div>
    {/each}
    <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
      {$t["login.button"]}
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