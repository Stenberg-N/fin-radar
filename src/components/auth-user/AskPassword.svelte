<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n/i18n";
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

<div id="ask-password-modal" class="flex column" transition:fade={{ duration: 200, easing: cubicInOut }}>
  <div class="form-outer-container" transition:fly={{ y: 40, duration: 600, easing: cubicInOut }}>
    <div id="ask-password-intro" class="flex column">
      <div class="flex row">
        <button id="button-lang" title={$t["language.button.title"] as string} class="button-primary transparent highlight outline default-corners" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>
          {$lang === 'en' ? 'FI' : 'EN'}
        </button>
        <h1>{$t["form.account-deletion.title"]}</h1>
        <button aria-label="Close modal" class="button-primary transparent highlight static" onclick={() => setViewState({ viewState: "isAskPassword", state: false })}>
          <span class="span-icon img-small" style="mask-image: url('/close-x.svg');"></span>
        </button>
      </div>
      {#each ($t["form.account-deletion.message"] as string[]) as text, i (i)}
        <p class="delete-account-paragraph">{text}</p>
      {/each}
    </div>
    <form class="form-bg" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <div class="flex column" style="align-items: unset;">
        <p class="form-p">
          {$t["password.title"]}
        </p>
        <div class="form-input-container">
          <input class="primary-input" type="password" placeholder={$t["password.title"] as string} bind:value={passwordInput} required />
          <button title={$t["form.password-visibility.show"] as string} class="button-primary transparent form" type="button" onclick={(e) => { togglePasswordVisibility(e.target);
            ((e.target as HTMLButtonElement).previousElementSibling as HTMLInputElement).type === "text" ? (e.target as HTMLButtonElement).title = $t["form.password-visibility.hide"] as string : (e.target as HTMLButtonElement).title = $t["form.password-visibility.show"] as string; }}>
            <span class="span-icon" style="mask-image: url('/eye-visible.svg');"></span>
          </button>
        </div>
      </div>

      <button class="button-primary white-bg form" type="submit" onmouseenter={() => isMoved = true} onmouseleave={() => isMoved = false}>
        {$t["confirm.button"]}
        <span class="span-icon" class:moveRight={isMoved} style="mask-image: url('/arrow.svg');"></span>
      </button>
    </form>
  </div>
</div>

<style>
  #ask-password-modal {
    position: fixed;
    z-index: 1001;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(24px);

    #ask-password-intro {

      > div {
        position: relative;
        justify-content: space-between;
        width: 100%;
        margin-bottom: 40px;
      }

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

    .delete-account-paragraph {
      margin: 0;
      text-align: center;
      word-wrap: break-word;
      hyphens: auto;
      user-select: none;
    }
  }
</style>