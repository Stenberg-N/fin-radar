<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";

  import List from "../List.svelte";
  import ColorList from "./ColorList.svelte";
  import ModalWrapper from "../ModalWrapper.svelte";

  interface Props {
    /** Used to pass a function to the delete button of the component. */
    handleContextMenuDelete: () => void;
    /** Returns the color that was clicked on. */
    handleContextMenuTabColor: (color: string) => void;
    /** Used to pass a function from the parent for handling entering edit mode for tabs. */
    handleTabEditStart: (contextMenu?: boolean) => void;
    /** Used to close the context menu on close button click. */
    setContextMenuVisibility: (state: boolean) => void;
    /**
     * An array of colors and their hover titles that is passed to ColorList.
     * 
     * @example
     * [
     *  { value: '#fff', title: ["White", "Valkoinen"] },
     *  { value: 'black', title: ["Black", "Musta"] },
     *  ...
     * ]
     */
    availableColors: { value: string; title: string[]; }[];
  }

  let {
    handleContextMenuDelete,
    handleContextMenuTabColor,
    handleTabEditStart,
    setContextMenuVisibility,
    availableColors,
  }: Props = $props();

  let toggleColorOptions = $state<HTMLButtonElement | null>(null);
  let contextMenuButtonsRefs = $state<HTMLButtonElement[]>([]);
  let isColorModal = $state<boolean>(false);

  const contextMenuButtons = [
    {
      get title() { return i18n.t["delete.button"] as string },
      icon: "/trash-can.svg",
      command: () => handleContextMenuDelete()
    },
    {
      get title() { return i18n.t["notes.change-tab-color"] as string; },
      icon: "/palette.svg",
      command: () => isColorModal = !isColorModal
    },
    {
      get title() { return i18n.t["edit.button"] as string; },
      icon:"/edit-pen.svg",
      command: () => handleTabEditStart(true)
    }
  ];

  // Used to collect contextMenuButtons button references and bind the button for color options to toggleColorsOptions,
  // and pass that to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (contextMenuButtonsRefs[1]) toggleColorOptions = contextMenuButtonsRefs[1];
  });

</script>

<ModalWrapper
  onOutsideClick={() => { setContextMenuVisibility(false); }}
  outline={{ width: 1, color: "var(--outline-color2)"}}
>
  <List
    padding={[8, 16, 16]}
  >
    {#if isColorModal}
      <ColorList
        theme="lighter1"
        {availableColors}
        onOutsideClick={(state) => { isColorModal = state; }}
        selectColor={(color) => { handleContextMenuTabColor(color); }}
        ignorableEls={[toggleColorOptions]}
      />
    {/if}

    <div id="context-menu-topbar" class="flex row">
      <h2 style="margin: 0;">{i18n.t["settings-banner.title"]}</h2>
      <button aria-label="Close menu" class="button-primary transparent highlight static" onclick={() => setContextMenuVisibility(false)}>
        <span style="mask-image: url('close-x.svg');" class="span-icon img-small"></span>
      </button>
    </div>
    <div id="context-menu-buttons" class="flex column">
      {#each contextMenuButtons as button, i (button.title)}
        <button class="button-primary transparent highlight default-corners" style="justify-content: flex-start; width: 100%;" onclick={button.command} bind:this={contextMenuButtonsRefs[i]}>
          <span style="mask-image: url({button.icon});" class="span-icon img-small"></span>
          {button.title}
        </button>
      {/each}
    </div>
  </List>
</ModalWrapper>

<style>
  #context-menu-topbar {
    width: 100%;
    gap: 3rem;
    padding-bottom: 0.75rem;
    justify-content: space-between;
    border-bottom: 2px solid var(--outline-color3);
  }
</style>