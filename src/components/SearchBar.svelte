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
    };
  } = $props();

  //svelte-ignore state_referenced_locally
  let searchable = $state<string | null>(null);
  let searchRegex: RegExp | null = null;
  let isSearchVisible = $state<boolean>(false);

  onMount(() => {
    if (options.getClearSearch) {
      options.getClearSearch({
        runClearSearch: clearSearch
      });
    }
  });

  const clearSearch = () => {
    searchable = null;
    options.sendRegexToParent(searchRegex = null);
    if (options.addFunctionsToClearSearch) options.addFunctionsToClearSearch.forEach((func) => func());
  };

  const handleSearch = () => {
    if (!isSearchVisible) isSearchVisible = true;
    if (!searchable || searchable.trim() === '') return;

    options.sendRegexToParent(searchRegex = new RegExp(searchable, 'gi'));
  };
</script>

<div
  id="search-container"
  class="horizontal-flex-container"
  class:mirrored={options.mirrorSearchBar}
  style="background-color: {isSearchVisible ? '#333' : 'transparent'}; box-shadow: {isSearchVisible ? '0 4px 8px rgba(0, 0, 0, 0.8)' : 'none'};"
  use:handleClickOutside={{ onOutsideClick: () => isSearchVisible = false }}
>
  {#if isSearchVisible}
    <input type="text" class="primary-input" placeholder={$t["search.placeholder"] as string} bind:value={searchable} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} 
      onkeydown={(e) => { switch (e.key) {
        case 'Enter': handleSearch(); break;
        case 'Escape': clearSearch(); break;
      }}}
    />
    <button aria-label="Clear search" id="clear-search-button" class="transparent-button-highlight" onclick={() => clearSearch()} transition:slide={{ axis: "x", duration: 250, easing: cubicInOut }} >
      <span class="span-icon" style="mask-image: url('/close-x.svg');"></span>
    </button>
  {/if}
  <button aria-label="Search" id="search-button" class="transparent-button-highlight" style="border-radius: {isSearchVisible && options.mirrorSearchBar ? '4px 0 0 4px' : isSearchVisible ? '0 4px 4px 0' : '50%'};" onclick={() => handleSearch()}>
    <span class="span-icon img-small" style="mask-image: url('search.svg');"></span>
  </button>
</div>

<style>
  #search-container {
    justify-content: flex-end;
    gap: 6px;
    border-radius: 4px;
    max-width: 240px;
    height: 32px;

    input {
      outline: none;
    }

    &.mirrored {
      justify-content: flex-start;
      
      #search-button { order: 1; }
      #clear-search-button { order: 2; }
      input { order: 3; }
    }
  }

  #search-button {
    flex-shrink: 0;
    height: 32px;
    width: 32px;
    
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