<script lang="ts">
  import { t, lang } from "$lib/i18n/i18n";
  import { handleClickOutside } from "$lib/actions";
  import ModalWrapper from "../ModalWrapper.svelte";

  let {
    handleContextMenuDelete,
    handleContextMenuTabColor,
    handleTabEditStart,
    setContextMenuVisibility,
    availableColors,
  }: {
    handleContextMenuDelete: () => void;
    handleContextMenuTabColor: (color: string) => void;
    handleTabEditStart: (contextMenu?: boolean) => void;
    setContextMenuVisibility: (state: boolean) => void;
    availableColors: Array<Record<string, string | string[]>>;
  } = $props();

  let toggleColorOptions = $state<HTMLButtonElement | null>(null);
  let contextMenuButtonsRefs = $state<HTMLButtonElement[]>([]);
  let isColorModal = $state<boolean>(false);
  let contextMenuButtons = [
    { title: "delete.button", icon: "/trash-can.svg", command: () => handleContextMenuDelete() },
    { title: "notes.change-tab-color", icon: "/palette.svg", command: () => isColorModal = !isColorModal },
    { title: "edit.button", icon:"/edit-pen.svg", command: () => handleTabEditStart(true)}
  ];

  // Used to collect contextMenuButtons button references and bind the button for color options to toggleColorsOptions,
  // and pass that to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (contextMenuButtonsRefs[1]) toggleColorOptions = contextMenuButtonsRefs[1];
  });

</script>

<div id="context-menu-container" class="modal-default vertical-flex-container"
  use:handleClickOutside={{ onOutsideClick: () => setContextMenuVisibility(false) }}
>
  {#if isColorModal}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
      <div id="context-menu-color-menu" class="horizontal-flex-container notes-color-menu"
        use:handleClickOutside={{ onOutsideClick: () => isColorModal = false, additionalElements: [toggleColorOptions] }}
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
    </ModalWrapper>
  {/if}

  <div id="context-menu-topbar" class="horizontal-flex-container">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button aria-label="Close menu" class="transparent-button-highlight" style="width: 32px; height: 32px;" onclick={() => setContextMenuVisibility(false)}>
      <span style="mask-image: url('close-x.svg');" class="span-icon img-small"></span>
    </button>
  </div>
  <div id="context-menu-buttons" class="vertical-flex-container">
    {#each contextMenuButtons as button, i (button.title)}
      <button class="primary-button horizontal-flex-container" onclick={button.command} bind:this={contextMenuButtonsRefs[i]}>
        <span style="mask-image: url({button.icon});" class="span-icon img-small"></span>
        {$t[button.title]}
      </button>
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
    border-bottom: 2px solid #333;
  }

  #context-menu-buttons {
    width: 100%;
    height: 100%;
    gap: 6px;
  }
</style>