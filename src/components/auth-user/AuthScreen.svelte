<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";
  import { onMount } from "svelte";

  import { lang, t } from "$lib/i18n/i18n";
  import { setViewState } from "$lib/viewStore";

  import LoginForm from "./LoginForm.svelte";
  import RegistrationForm from "./RegistrationForm.svelte";

  let isLoginView = $state<boolean>(true);
  let isVisible = $state(false);

  onMount(() => {
    isVisible = true;
  });

</script>

<main id="main-auth-container" class="vertical-flex-container">
  {#if isVisible}
    <div class="form-outer-container" transition:fly={{ y: 40, duration: 1200, easing: cubicInOut }}>
      <div style="position: relative; display: flex; flex-direction: row; align-items: center; margin-bottom: 40px;">
        <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button-dark" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
        <h1 style="position: absolute; left: 50%; transform: translateX(-50%); margin: 0;">{isLoginView ? $t["form.login.title"] : $t["form.register.title"]}</h1>
      </div>
      {#if isLoginView}
        <LoginForm />
      {:else}
        <RegistrationForm setLoginView={(state) => isLoginView = state} />
      {/if}
      <div class="form-question-container">
        <div class="horizontal-flex-container">
          <p class="form-p">{isLoginView ? $t["form.no-account.question"] : $t["form.already-account.question"]}</p>
          <button class="form-button transparent-button" style="outline: none;" onclick={() => isLoginView = !isLoginView}>{isLoginView ? $t["form.no-account.button"] : $t["form.already-account.button"]}</button>
        </div>
        <div class="horizontal-flex-container">
          <p class="form-p">{$t["form.forgot-password.question"]}</p>
          <button class="form-button transparent-button" style="outline: none;" onclick={() => setViewState({ viewState: "isRecoveryView", state: true })}>{$t["form.forgot-password.button"]}</button>
        </div>
      </div>
    </div>
  {/if}
</main>

<style>
  #main-auth-container {
    position: fixed;
    inset: 0;
    margin: 190px auto;
  }
  #main-auth-container .form-outer-container {
    min-height: 612px;
    max-height: 612px;
    height: 100%;
  }

  #main-auth-container .form-question-container .horizontal-flex-container {
    align-self: flex-start;
    gap: 10px;
  }

  @media (max-height: 990px) {
    #main-auth-container {
      margin: auto;
    }
  }
</style>