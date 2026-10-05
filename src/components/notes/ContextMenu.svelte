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

  let timeout: ReturnType<typeof setTimeout> | null = null;
  let hoverTitle = $state<{ state: boolean, content: string }>({ state: false, content: '' });

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

  $effect(() => {
    return () => { if (timeout) clearTimeout(timeout); };
  });

  const handleMouseEnter = (content?: string) => {
    if (timeout) clearTimeout(timeout);
    hoverTitle.content = content ? content : '';

    timeout = setTimeout(() => {
      hoverTitle.state = true;
    }, 300);
  };

  const handleMouseLeave = () => {
    if (timeout) clearTimeout(timeout);
    timeout = null;

    hoverTitle.state = false;
  };

</script>

<div id="context-menu-container" class="modal-default flex column"
  use:handleClickOutside={{ onOutsideClick: () => setContextMenuVisibility(false) }}
>
  {#if isColorModal}
    <ModalWrapper options={{ transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
      <div id="context-menu-color-menu" class="flex row notes-color-menu"
        use:handleClickOutside={{ onOutsideClick: () => isColorModal = false, getAdditionalElements: () => [toggleColorOptions] }}
      >
        <p style="width: 100%; margin-top: 0;">{$lang === 'en' ? "Dark" : "Tummat"}</p>
        {#each availableColors as color, i (i)}
          <button
            class="button-primary transparent"
            aria-label={$lang === 'en' ? color.title[0] : color.title[1]}
            style="background-color: {color.value}; border-radius: 50%;"
            onclick={() => { handleContextMenuTabColor(color.value as string); isColorModal = false; }}
            onmouseenter={() => handleMouseEnter($lang === 'en' ? color.title[0] : color.title[1])}
            onmouseleave={handleMouseLeave}
          ></button>
          {#if i === 11}
            <p style="width: 100%;">{$lang === 'en' ? "Bright" : "Kirkkaat"}</p>
          {/if}
        {/each}
      </div>
    </ModalWrapper>
  {/if}

  {#if hoverTitle.state}
    <ModalWrapper options={{
      position: { centerElement: true, moveTop: -40 },
      borderRadius: 8,
      outline: { width: 1, color: 'var(--outline-color4)'}
      }}
    >
      <p id="notes-context-menu-hover-title-content">
        {hoverTitle.content}
      </p>
    </ModalWrapper>
  {/if}

  <div id="context-menu-topbar" class="flex row">
    <h2 style="margin: 0;">{$t["settings-banner.title"]}</h2>
    <button aria-label="Close menu" class="button-primary transparent highlight static" onclick={() => setContextMenuVisibility(false)}>
      <span style="mask-image: url('close-x.svg');" class="span-icon img-small"></span>
    </button>
  </div>
  <div id="context-menu-buttons" class="flex column">
    {#each contextMenuButtons as button, i (button.title)}
      <button class="button-primary" onclick={button.command} bind:this={contextMenuButtonsRefs[i]}>
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
    background-color: var(--color-secondary2);

    .notes-color-menu { background-color: var(--color-secondary3); }

    #notes-context-menu-hover-title-content {
      margin: 0;
      padding: 0.25rem 0.5rem;
      background-color: var(--color-secondary4);
    }
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