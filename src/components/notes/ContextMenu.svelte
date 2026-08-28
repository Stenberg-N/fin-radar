<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { t, lang } from "$lib/i18n";
  import { handleClickOutside } from "$lib/actions";
  import { viewport } from "$lib/viewport";

  let {
    handleContextMenuDelete,
    handleContextMenuTabColor,
    handleTabEditStart,
    setContextMenuVisibility,
    cursorPosX,
    cursorPosY,
    availableColors,
    isContextMenu,
  }: {
    handleContextMenuDelete: () => void;
    handleContextMenuTabColor: (color: string) => void;
    handleTabEditStart: (contextMenu?: boolean) => void;
    setContextMenuVisibility: (state: boolean) => void;
    cursorPosX: number;
    cursorPosY: number;
    availableColors: Array<Record<string, string | string[]>>;
    isContextMenu: boolean;
  } = $props();

  let toggleColorOptions = $state<HTMLButtonElement | null>(null);
  let isColorModal = $state<boolean>(false);
  let contextMenuButtons = [
    { title: "delete.button", icon: "/trash-can.svg", command: () => handleContextMenuDelete() },
    { title: "notes.change-tab-color", icon: "/palette.svg", command: () => handleColorMenu() },
    { title: "edit.button", icon:"/edit-pen.svg", command: () => handleTabEditStart(true)}
  ];
  let contextMenuButtonsRefs = $state<HTMLButtonElement[]>([]);

  $effect(() => {
    if (!isContextMenu) return;

    const contextMenu = document.getElementById("context-menu-container");
    if (!contextMenu) return;

    contextMenu.style.setProperty('--context-menu-left', `${cursorPosX > 240 ? cursorPosX - 240 : cursorPosX}px`);
    contextMenu.style.setProperty('--context-menu-top', `${cursorPosY}px`);
  });

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
  const handleOutsideClick = () => { setContextMenuVisibility(false) };
  
  /***********************************************************************************************************************************/

  const handleColorMenu = () => {
    isColorModal = !isColorModal;
    queueMicrotask(() => {
      const colorMenu = document.getElementById("context-menu-color-menu");
      if (!colorMenu || !isColorModal) return;
      colorMenu.style.setProperty('--context-color-menu-left', `${$viewport.cursorX}px`);
    });
  };
</script>

<div id="context-menu-container" class="modal-default vertical-flex-container" transition:fade={{ duration: 200, easing: cubicInOut }}
  use:handleClickOutside={{ onOutsideClick: handleOutsideClick }}
>
  {#if isColorModal}
    <div id="context-menu-color-menu" class="horizontal-flex-container notes-color-menu"
      use:handleClickOutside={{ onOutsideClick: () => isColorModal = false, additionalElements: [toggleColorOptions] }}
      transition:fade={{ duration: 200, easing: cubicInOut }}
    >
      <p style="width: 100%; margin-top: 0;">{$lang === 'en' ? "Dark" : "Tummat"}</p>
      {#each availableColors as color, i (i)}
        <button class="transparent-button" title={$lang === 'en' ? color.title[0] : color.title[1]} style="background-color: {color.value}; border-radius: 50%;"
          onclick={() => { handleContextMenuTabColor(color.value as string); isColorModal = false; }}
        ></button>
        {#if i === 11}
          <p style="width: 100%;">{$lang === 'en' ? "Bright" : "Kirkkaat"}</p>
        {/if}
      {/each}
    </div>
  {/if}

  <div id="context-menu-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setContextMenuVisibility(false)}><img src="close-x.svg" alt="Close" class="img-small" /></button>
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
    left: var(--context-menu-left);
    top: var(--context-menu-top);
  }

  #context-menu-topbar {
    width: 100%;
    gap: 48px;
    padding-bottom: 12px;
    justify-content: space-between;
    border-bottom: 2px solid #333;
  }

  #context-menu-buttons {
    width: 100%;
    height: 100%;
    gap: 6px;
  }

  .notes-color-menu {
    left: var(--context-color-menu-left);
  }
</style>