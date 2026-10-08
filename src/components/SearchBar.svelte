<script lang="ts">
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { i18n } from "$lib/i18n/i18n.svelte";
  import { handleClickOutside } from "$lib/actions.svelte";

  let {
    options,
  }: {
    options: {
      sendRegexToParent: (regex: RegExp | null) => void;
      getClearSearch?: (func: {
        runClearSearch: () => void;
      } | null) => void;
      addFunctionsToClearSearch?: (() => void)[];
      mirrorSearchBar?: boolean;
      disabled?: boolean;
      searchModeIndicator?: boolean;
    };
  } = $props();

  let searchInput = $state<HTMLInputElement | null>(null);
  let searchable = $state<string | null>(null);
  let isSearchVisible = $state<boolean>(false);
  let timeout: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    if (options.getClearSearch) {
      options.getClearSearch({
        runClearSearch: clearSearch
      });
    }
  });

  $effect(() => {
    return () => { if (timeout) clearTimeout(timeout); };
  });

  const clearSearch = () => {
    searchable = null;
    options.sendRegexToParent(null);
    if (options.addFunctionsToClearSearch) options.addFunctionsToClearSearch.forEach((func) => func());
  };

  const handleSearch = () => {
    if (!isSearchVisible) isSearchVisible = true;
    if (!searchable || searchable.trim() === '') return;

    options.sendRegexToParent(new RegExp(searchable, 'i'));
  };

  const handleInput = () => {
    if (searchable?.trim() === '') {
      if (timeout) clearTimeout(timeout);
      timeout = null;
      clearSearch();
      return;
    }

    if (timeout) clearTimeout(timeout);
    timeout = null;

    timeout = setTimeout(() => {
      handleSearch();
    }, 300);
  };
</script>

<div
  id="search-container"
  class="flex row"
  class:mirrored={options.mirrorSearchBar}
  style="background-color: {isSearchVisible ? 'var(--color-secondary1)' : 'transparent'}; box-shadow: {isSearchVisible ? '0 4px 8px rgba(0, 0, 0, 0.8)' : 'none'};"
  use:handleClickOutside={{ onOutsideClick: () => searchable !== null ? {} : isSearchVisible = false }}
>
  <button aria-label="Search" id="search-button"
    class="button-primary transparent highlight static"
    style="border-radius: {isSearchVisible && options.mirrorSearchBar ? '0 0.25rem 0.25rem 0' : isSearchVisible ? '0.25rem 0 0 0.25rem' : '50%'};"
    onclick={() => handleSearch()}
    disabled={options?.disabled}
  >
    <span class="span-icon img-small" style="mask-image: url('search.svg');"></span>
  </button>
  {#if isSearchVisible}
    <button aria-label="Clear search" id="clear-search-button" class="button-primary transparent highlight" onclick={() => clearSearch()} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
      <span class="span-icon" style="mask-image: url('/close-x.svg');"></span>
    </button>
    <input
      bind:this={searchInput}
      bind:value={searchable}
      type="text"
      class="primary-input"
      placeholder={i18n.t["search.placeholder"] as string}
      oninput={handleInput}
      transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} 
      onkeydown={(e) => { switch (e.key) {
        case 'Enter': handleSearch(); break;
        case 'Escape': clearSearch(); break;
      }}}
    />
    {#each [searchInput], i (i)}
      {onMount(() => searchInput?.focus())}
    {/each}
  {/if}

  {#if searchable && options.searchModeIndicator}
    <span id="search-mode-indicator" transition:fade={{ duration: 600, delay: 300, easing: cubicInOut }}></span>
  {/if}
</div>

<style>
  #search-container {
    flex-shrink: 0;
    justify-content: flex-end;
    gap: 6px;
    border-radius: 0.25rem;
    max-width: 240px;
    height: 2rem;

    input {
      outline: none;
    }

    &.mirrored {
      justify-content: flex-start;
      
      #search-button { order: 4; }
      #clear-search-button { order: 3; }
      #search-mode-indicator { order: 2; }
      input { order: 1; }
    }

    #search-mode-indicator {
      position: relative;
      flex-shrink: 0;
      align-self: flex-start;
      width: 0.5rem;
      height: 0.5rem;
      margin: 0.25rem;
      border-radius: 50%;
      background-color: var(--color-highlight1);

      &::after {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 2px solid var(--color-highlight1);
        animation: pulse 1.5s ease-out infinite;
      }
    }
  }

  #clear-search-button {
    flex-shrink: 0;
    width: 20px;
    height: 20px;

    span {
      width: 10px;
      height: 10px;
    }
  }
</style>