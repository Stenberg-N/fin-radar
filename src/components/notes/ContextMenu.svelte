<script lang="ts">
  import { getContext } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/functions";
  import { setViewState } from "$lib/viewStore";

  let {
    handleContextMenuDelete,
    handleContextMenuTabColor,
    handleTabEditStart,
    cursorPosX,
    cursorPosY,
    availableColors,
  }: {
    handleContextMenuDelete: () => void;
    handleContextMenuTabColor: (color: string) => void;
    handleTabEditStart: (contextMenu?: boolean) => void;
    cursorPosX: number;
    cursorPosY: number;
    availableColors: Array<Record<string, string | string[]>>
  } = $props();

  let toggleColorOptions = $state<HTMLButtonElement | null>(null);
  let isColorModal = $state<boolean>(false);
  let contextMenuButtons = [
    { title: "delete.button", icon: "/trash-can.svg", command: () => handleContextMenuDelete() },
    { title: "notes.change-tab-color", icon: "/palette.svg", command: () => isColorModal = !isColorModal },
    { title: "edit.button", icon:"/edit-pen.svg", command: () => handleTabEditStart(true)}
  ];
  let contextMenuButtonsRefs = $state<HTMLButtonElement[]>([]);

  // Used to collect contextMenuButtons button references and bind the button for color options to toggleColorsOptions,
  // and pass that to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (contextMenuButtonsRefs[1]) toggleColorOptions = contextMenuButtonsRefs[1];
  });

  /***********************************************************************************************************************************\
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const handleOutsideClick = () => { setViewState("isContextMenu", false); };
  
  /***********************************************************************************************************************************/

</script>

<div id="context-menu-container" class="modal-default vertical-flex-container" style="left: {cursorPosX}px; top: {cursorPosY}px;" transition:fade={{ duration: 200, easing: cubicInOut }}
  use:handleClickOutside={{ getIgnoredElements, onOutsideClick: handleOutsideClick }}
>
  {#if isColorModal}
    <div class="horizontal-flex-container notes-color-menu" use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isColorModal = false, additionalElements: [toggleColorOptions] }} transition:slide={{ axis:"y", duration: 200, easing: cubicInOut }}>
      {#each availableColors as color (color.value)}
        <button class="transparent-button" title={$lang === 'en' ? color.title[0] : color.title[1]} style="background-color: {color.value}; border-radius: 50%;"
          onclick={() => { handleContextMenuTabColor(color.value as string); isColorModal = false; }}
        ></button>
      {/each}
    </div>
  {/if}

  <div id="context-menu-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState("isContextMenu", false)}><img src="close-x.svg" alt="Close" class="img-small" /></button>
  </div>
  <div id="context-menu-buttons" class="vertical-flex-container">
    {#each contextMenuButtons as button, i (button.title)}
      <button class="primary-button horizontal-flex-container" onclick={() => button.command()} bind:this={contextMenuButtonsRefs[i]}><img src={button.icon} alt="" class="img-small" />{$t[button.title]}</button>
    {/each}
  </div>
</div>

<style>
  #context-menu-container {
    z-index: 1000;
    min-width: 240px;
  }

  #context-menu-topbar {
    width: 100%;
    gap: 48px;
    padding-bottom: 12px;
    justify-content: space-between;
    border-bottom: 1px solid #333;
  }

  #context-menu-buttons {
    width: 100%;
    height: 100%;
    gap: 6px;
  }

  #context-menu-buttons button {
    justify-content: flex-start;
    width: 100%;
    gap: 8px;
    background-color: transparent;
    box-shadow: none;
  }
  #context-menu-buttons button:hover {
    background-color: #333;
  }

  .notes-color-menu {
    top: 75px;
    right: 5px;
  }
</style>