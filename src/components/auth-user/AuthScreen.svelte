<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { onMount } from "svelte";

  import { lang, t } from "$lib/i18n/i18n";
  import { setViewState } from "$lib/viewStore";
  import { HoverTitle } from "$lib/actions.svelte";

  import LoginForm from "./LoginForm.svelte";
  import RegistrationForm from "./RegistrationForm.svelte";
  import ModalWrapper from "../ModalWrapper.svelte";

  let isLoginView = $state<boolean>(true);
  let isVisible = $state(false);

  const hover = new HoverTitle();

  onMount(() => {
    isVisible = true;
  });

  $effect(() => {
    return () => hover.destroy();
  });

</script>

<main id="main-auth-container" class="flex column">
  {#if hover.isHovering}
    <ModalWrapper options={{
      position: { moveTop: -30 },
      transition: { type: "fade", duration: 200, easing: "cubic-in-out" },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color2)'} 
      }}
    >
      <p id="main-auth-hover-title-content">
        {$t["language.button.title"]}
      </p>
    </ModalWrapper>
  {/if}

  {#if isVisible}
    <div class="form-outer-container" transition:fly={{ y: 40, duration: 1200, easing: cubicInOut }}>
      <div id="main-auth-topbar" class="flex row">
        <button
          id="button-lang"
          class="button-primary transparent highlight outline default-corners"
          onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}
          onmouseenter={() => hover.enter()}
          onmouseleave={hover.leave}
        >
          {$lang === 'en' ? 'EN' : 'FI'}
        </button>
        <h1>
          {$t[isLoginView ? "login.title" : "register.title"]}
        </h1>
      </div>
      {#if isLoginView}
        <LoginForm />
      {:else}
        <RegistrationForm setLoginView={(state) => { isLoginView = state; }} />
      {/if}
      <div class="form-question-container">
        <div class="flex row">
          <p class="form-p">
            {isLoginView ? $t["form.no-account.question"] : $t["form.already-account.question"]}
          </p>
          <button class="button-primary transparent form-text" onclick={() => isLoginView = !isLoginView}>
            {isLoginView ? $t["form.no-account.button"] : $t["form.already-account.button"]}
          </button>
        </div>
        <div class="flex row">
          <p class="form-p">
            {$t["form.forgot-password.question"]}
          </p>
          <button class="button-primary transparent form-text" onclick={() => setViewState({ viewState: "isRecoveryView", state: true })}>
            {$t["form.forgot-password.button"]}
          </button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  #main-auth-container {
    position: fixed;
    inset: 0;
    background-image: radial-gradient(ellipse at center, var(--color-secondary1-a) 6%, var(--color-primary2) 24%, var(--color-primary2-a) 50%, var(--color-primary1) 72%);

    #main-auth-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary2);
    }

    #main-auth-topbar {
      position: relative;
      width: 100%;
      justify-content: unset;
      margin-bottom: 40px;

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

    .form-outer-container {
      flex-shrink: 0;
      height: 646px;

      .flex.row {
        align-self: flex-start;
        gap: 10px;
      }
    }
  }

  @media (max-height: 990px) {
    #main-auth-container {
      margin: auto;
    }
  }
</style>