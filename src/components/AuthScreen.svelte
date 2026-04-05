<script lang="ts">
  import { lang, t } from "$lib/i18n";

  import LoginForm from "./LoginForm.svelte";
  import RegistrationForm from "./RegistrationForm.svelte";

  let isLoginView = $state<boolean>(true);

  const setLoginView = (state: boolean) => {
    isLoginView = state;
  }

</script>

<main style="position: fixed; inset: 0; display: flex; align-items: center; justify-content: center; margin: 100px auto;">
  <div id="form-outer-container">
    <div style="position: relative; display: flex; flex-direction: row; align-items: center;">
      <button title={$t["language.button.title"] as string} style="width: 40px; font-weight: 600;" class="primary-button" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>{$lang === 'en' ? 'FI' : 'EN'}</button>
      <h1 style="position: absolute; left: 50%; transform: translateX(-50%); margin: 0;">{isLoginView ? $t["form.login.title"] : $t["form.register.title"]}</h1>
    </div>
    {#if isLoginView}
      <LoginForm setLoginView={setLoginView} />
    {:else}
      <RegistrationForm setLoginView={setLoginView} />
    {/if}
  </div>
</main>

<style>

@media (max-width: 800px) {
  #form-outer-container {
    max-width: 85%;
  }
}
</style>