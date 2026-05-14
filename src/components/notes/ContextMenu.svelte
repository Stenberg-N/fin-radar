<script lang="ts">
  import { getContext } from "svelte";

  import { t } from "$lib/i18n";
  import { handleClickOutside } from "$lib/functions";
  import { setViewState } from "$lib/viewStore";

  let {
    handleContextMenuDelete,
    handleContextMenuTabColor,
    cursorX,
    cursorY,
    availableColors,
  }: {
    handleContextMenuDelete: () => void;
    handleContextMenuTabColor: (color: string) => void;
    cursorX: number;
    cursorY: number;
    availableColors: Array<Record<string, string>>
  } = $props();

  let isColorModal = $state<boolean>(false);
  let contextMenuButtons = [
    { title: "delete.button", icon: "/trash-can.svg", command: () => handleContextMenuDelete() },
    { title: "notes.change-tab-color", icon: "/palette.svg", command: () => isColorModal = true },
  ];

  /***********************************************************************************************************************************
  |
  | Context, Helper & Wrapper functions
  |
  \***********************************************************************************************************************************/
  const getIgnoredElements = getContext<() => (HTMLButtonElement | HTMLDivElement | null)[]>('ignoredElements');
  const handleOutsideClick = () => { setViewState("isContextMenu", false); };
  
  /***********************************************************************************************************************************/

</script>

<div id="context-menu-container" class="vertical-flex-container" style="left: {cursorX - 390}px; top: {cursorY - 202}px;" use:handleClickOutside={{ getIgnoredElements, onOutsideClick: handleOutsideClick }}>
  {#if isColorModal}
    <div id="context-menu-colors-container" class="horizontal-flex-container" use:handleClickOutside={{ getIgnoredElements, onOutsideClick: () => isColorModal = false }}>
      {#each availableColors as color (color.value)}
        <button class="transparent-button" title={color.value} style="background-color: {color.value}; border-radius: 50%;"
          onclick={() => { handleContextMenuTabColor(color.value); isColorModal = false; }}
        ></button>
      {/each}
    </div>
  {/if}

  <div id="context-menu-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setViewState("isContextMenu", false)}><img src="close-x.svg" alt="Close" class="img-small" /></button>
  </div>
  <div id="context-menu-buttons" class="vertical-flex-container">
    {#each contextMenuButtons as button (button.title)}
      <button class="primary-button horizontal-flex-container" onclick={() => button.command()}><img src={button.icon} alt="" class="img-small" />{$t[button.title]}</button>
    {/each}
  </div>
</div>

<style>
  #context-menu-container {
    position: absolute;
    z-index: 1000;
    min-width: 240px;
    gap: 12px;
    padding: 12px;
    border-radius: 8px;
    outline: 1px solid #333;
    background-color: #181818;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
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

  #context-menu-colors-container {
    position: absolute;
    bottom: 28px;
    right: 5px;
    z-index: 1;
    flex-wrap: wrap;
    gap: 4px;
    padding: 8px 12px;
    border-radius: 6px;
    background-color: #222;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }

  #context-menu-colors-container button {
    width: 24px;
    height: 24px;
  }
</style>