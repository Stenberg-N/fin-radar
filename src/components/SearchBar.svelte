<script lang="ts">
  import { onMount } from "svelte";
  import { slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t } from "$lib/i18n/i18n";
  import { handleClickOutside } from "$lib/actions";

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
    };
  } = $props();

  //svelte-ignore state_referenced_locally
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
    <input type="text" class="primary-input" placeholder={$t["search.placeholder"] as string} bind:value={searchable} oninput={handleInput} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} 
      onkeydown={(e) => { switch (e.key) {
        case 'Enter': handleSearch(); break;
        case 'Escape': clearSearch(); break;
      }}}
    />
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
      
      #search-button { order: 3; }
      #clear-search-button { order: 2; }
      input { order: 1; }
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