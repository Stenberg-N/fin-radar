<script lang="ts">
  import { i18n } from "$lib/i18n/i18n.svelte";
  import { HoverTitle } from "$lib/classes/hoverTitle.svelte";

  import ModalWrapper from "../ModalWrapper.svelte";
  import List from "../List.svelte";

  interface Props {
    /** Used to pass a function to the delete button of the component. */
    handleContextMenuDelete: () => void;
    /** Returns the color that was clicked on. */
    handleContextMenuTabColor: (color: string) => void;
    /** Used to pass a function from the parent for handling entering edit mode for tabs. */
    handleTabEditStart: (contextMenu?: boolean) => void;
    /** Used to close the context menu on close button click. */
    setContextMenuVisibility: (state: boolean) => void;
    /** The possible colors to show in the color list of the component. */
    availableColors: Array<Record<string, string | string[]>>;
  }

  type HoverContent = { content: string };

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

  const hover = new HoverTitle<HoverContent>();

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

  const colorMenuSections = $derived([
    {
      get title() { return i18n.lang === 'en' ? "Dark" : "Tummat"; },
      colors: availableColors.slice(0, 12),
    },
    {
      get title() { return i18n.lang === 'en' ? "Bright" : "Kirkkaat"; },
      colors: availableColors.slice(13),
    }
  ]);

  // Used to collect contextMenuButtons button references and bind the button for color options to toggleColorsOptions,
  // and pass that to handleClickOutside to be ignored, since Svelte's bind:this doesn't allow conditional expressions.
  $effect(() => {
    if (contextMenuButtonsRefs[1]) toggleColorOptions = contextMenuButtonsRefs[1];
  });

  $effect(() => {
    return () => { hover.destroy(); };
  });

</script>

<List
  theme="lighter"
  padding={[8, 16, 16]}
>
  {#if isColorModal}
    <ModalWrapper
      transition={{ type: "fade", duration: 200, easing: "cubic-in-out" }}
      onOutsideClick={() => isColorModal = false}
      ignorableEls={[toggleColorOptions]}
    >
      <div id="context-menu-color-menu" class="flex column notes-color-menu">
        {#each colorMenuSections as { title, colors }, i (title)}
          <p style="{i === 0 ? 'margin-top: 0' : ''};">
            {title}
          </p>
          <div class="color-menu-grid">
            {#each colors as color (color)}
              <button
                class="button-primary transparent"
                aria-label={i18n.lang === 'en' ? color.title[0] : color.title[1]}
                style="background-color: {color.value}; border-radius: 50%;"
                onclick={() => { handleContextMenuTabColor(color.value as string); isColorModal = false; }}
                onmouseenter={() => hover.enter({ content: i18n.lang === 'en' ? color.title[0] : color.title[1]})}
                onmouseleave={(e) => hover.leave(e)}
              ></button>
            {/each}
          </div>
        {/each}
      </div>
    </ModalWrapper>
  {/if}

  {#if hover.isHovering}
    <ModalWrapper
      attributes={{ "hover-title-owner": hover.id }}
      position={{ centerElement: true, moveTop: -40 }}
      borderRadius={8}
      outline={{ width: 1, color: 'var(--outline-color4)'}}
    >
      <p id="notes-context-menu-hover-title-content">
        {hover.target?.content}
      </p>
    </ModalWrapper>
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

<style>

  .notes-color-menu { background-color: var(--color-secondary3); }

  #notes-context-menu-hover-title-content {
    margin: 0;
    padding: 0.25rem 0.5rem;
    background-color: var(--color-secondary4);
  }

  #context-menu-topbar {
    width: 100%;
    gap: 3rem;
    padding-bottom: 0.75rem;
    justify-content: space-between;
    border-bottom: 2px solid var(--outline-color2);
  }

  #context-menu-buttons {
    width: 100%;
    height: 100%;
    gap: 6px;
  }
</style>