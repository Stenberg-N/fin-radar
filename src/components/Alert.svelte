<script lang="ts">
  import { cubicInOut } from "svelte/easing";
  import { fly } from "svelte/transition";

  import { Alert } from "$lib/classes/alertStore.svelte";
  import { i18n } from "$lib/i18n/i18n.svelte";

  import ItemSelect from "./elements/ItemSelect.svelte";

  interface Props {
    alert: Alert;
    /** Used to return the item from the ItemSelect component if isSelector is set to true. */
    getItem?: (item: string) => void;
  }

  let {
    alert,
    getItem,
  }: Props = $props();

  let alertEl: HTMLDivElement | null = null;

  $effect(() => {
    if (!alertEl) return;
    alertEl.style.setProperty('--progress-width', `${alert.progress}%`);
  });

</script>

<div bind:this={alertEl} role="alert" class="alert" transition:fly={{ y: 50, duration: 200, easing: cubicInOut }} onmouseenter={alert.pause} onmouseleave={alert.resume}>
  <button aria-label="Close alert" class="button-primary transparent highlight" style="position: absolute; top: 2px; right: 2px; width: 20px; height: 20px;" onclick={alert.destroy}>
    <span class="span-icon" style="mask-image: url('/close-x.svg'); width: 10px; height: 10px;"></span>
  </button>
  <p class="alert-message">
    {#if Array.isArray(i18n.t[alert.config.message])}
      {#each (i18n.t[alert.config.message] as string | string[]) as msg, i (i)}
        <span>{msg}</span>
      {/each}
    {:else if i18n.t[alert.config.message]}
      {i18n.t[alert.config.message]}
    {:else}
      {alert.config.message}
    {/if}
    {#if alert.config.addDivider && alert.config.additionalText}
      <br>
      <span style="width: 100%; height: 0; margin: 8px 0; border: 1px solid var(--outline-color2);"></span>
    {/if}
    {#if Array.isArray(alert.config.additionalText)}
      {#each alert.config.additionalText as text, i (i)}
        <span class="list-ignore">{text}</span>
      {/each}
    {:else}
      {alert.config.additionalText}
    {/if}
  </p>
  {#if alert.config.buttons}
    <div class="alert-buttons">
      {#if alert.config.selector && getItem}
        <ItemSelect
          onItemSelect={(item) => { getItem(item); }}
          data={alert.config.selector.selectorData}
          overrideDataValues={alert.config.selector.selectorOverrideData}
          position={{ usePortal: true, isPositionAbsolute: false, moveTop: -44 }}
        />
      {/if}
      <button class="button-primary {alert.config.isConfirmButtonWhite ? 'white-bg' : 'transparent highlight outline default-corners'}" onclick={() => { alert.config.onConfirm(); alert.destroy(); }}>
        {i18n.t[alert.config.confirmButtonI18nKey]}
      </button>
      {#if !alert.config.onlyConfirmButton}
        <button class="button-primary transparent highlight outline default-corners" onclick={() => { alert.config.onCancel(); alert.destroy(); }}>
          {i18n.t[alert.config.cancelButtonI18nKey]}
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .alert {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-width: 240px;
    max-width: 440px;
    background-color: var(--color-secondary1);
    padding: 1.5rem;
    outline: 1px solid var(--outline-color2);
    border: none;
    border-radius: 0.5rem;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
    transition: width 100ms linear;
    overflow: hidden;

    &::after {
      position: absolute;
      content: '';
      bottom: 0;
      left: 0;
      width: var(--progress-width, 0%);
      height: 3px;
      background-color: var(--color-white-primary1);
    }
  }

  .alert-message {
    text-align: left;
    margin: 0;
    word-wrap: break-word;
    hyphens: auto;

    span {
      position: relative;
      display: list-item;
      list-style-type: none;

      &:not(:first-child, :nth-child(2)):not(.list-ignore) {
        padding-left: 2rem;
      }

      &:not(:first-child, :nth-child(2)):not(.list-ignore)::before {
        content: '•';
        position: absolute;
        left: 1rem;
      }
    }
  }

  .alert-buttons {
    display: flex;
    flex-direction: row;
    gap: 1rem;
    padding-top: 0.75rem;
    border-top: 2px solid var(--outline-color2);

    button {
      height: 2rem;
    }
  }
</style>