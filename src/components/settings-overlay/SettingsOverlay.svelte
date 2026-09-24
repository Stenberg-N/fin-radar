<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { setViewState } from "$lib/viewStore";
  import { userPrefs, updateUserPrefs } from "$lib/prefsStore";
  import { moveGutter, isGutterMoving } from "$lib/actions";
  import { t, lang } from "$lib/i18n/i18n";

  import ModalWrapper from "../ModalWrapper.svelte";
  import Account from "./settings-pages/Account.svelte";
  import Notes from "./settings-pages/Notes.svelte";

  type PageName = "account" | "notes";

  let selectedPage = $state<PageName>("account");
  let settingsContent = $state<HTMLDivElement | null>(null);
  let settingsContentWidth = $state<number>(0);
  const sideBarWidth = $derived($userPrefs.settingsOverlayPrefs.sideBarWidth);
  let isHovering = $state(false);
  let timer: ReturnType<typeof setTimeout>;

  const settingsPages = {
    "account": Account,
    "notes": Notes,
  };
  const settingsSidebarButtons = [
    {
      id: "account",
      img: "/user.svg",
      get title() { return $t["settings.pages.account.title"]; },
    },
    {
      id: "notes",
      img: "/notes.svg",
      get title() { return $t["main.layout.view-title"][4]; },
    },
  ];

  const handleMouseEnter = () => {
    timer = setTimeout(() => { isHovering = true }, 300);
  };

  const handleMouseLeave = () => {
    clearTimeout(timer);
    isHovering = false;
  };

  const setSelectedPage = (pageId: PageName) => {
    switch (pageId) {
      case "account": selectedPage = "account"; break;
      case "notes": selectedPage = "notes"; break;
    }
  };

  $effect(() => {
    if (!settingsContent) return;
    settingsContent.style.alignItems = settingsContentWidth > 1360 ? 'center' : 'flex-start';
  });
</script>

<div id="main-settings-overlay" class="flex row" transition:fade={{ duration: 200, easing: cubicInOut }}>
  {#if $isGutterMoving || isHovering}
    <ModalWrapper options={{ position: { isContinuousUpdate: true, centerElement: true }, transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
      <p style="background-color: var(--color-secondary1); margin: 0; padding: 0.5rem;">{`${sideBarWidth}px`}</p>
    </ModalWrapper>
  {/if}

  <div id="main-settings-overlay-sidebar" class="flex column" style="width: {sideBarWidth}px;">
    <div id="main-settings-overlay-sidebar-topbar" class="flex row">
      <div class="flex row">
        <span class="span-icon img-small-medium" style="mask-image: url('/settings-cog.svg');"></span>
        <h3>{$t["main.layout.settings"]}</h3>
      </div>
      <button class="button-primary transparent highlight outline" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>
        {$lang === 'en' ? 'EN' : 'FI'}
      </button>
    </div>
    <div id="main-settings-overlay-sidebar-content" class="flex column">
      {#each settingsSidebarButtons as button, i (i)}
        <button class="main-settings-overlay-sidebar-button button-primary transparent highlight" class:selected-page={selectedPage === button.id} onclick={() => setSelectedPage(button.id as PageName)}>
          <span class="span-icon img-small-medium" style="mask-image: url('{button.img}');"></span>
          {button.title}
        </button>
      {/each}
    </div>
  </div>

  <div role="slider" aria-valuenow={sideBarWidth} tabindex="0" id="main-settings-overlay-gutter" class="resize-gutter-default flex row" class:highlight={isHovering}
    use:moveGutter={{ onResize: (newWidth) => { updateUserPrefs("settingsOverlayPrefs", "sideBarWidth", newWidth); },  min: 200, max: 800 }}
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
  ></div>

  <div id="main-settings-overlay-content">
    <div class="flex column" bind:clientWidth={settingsContentWidth} bind:this={settingsContent}>
      <button aria-label="Close settings" class="button-primary transparent highlight static" onclick={() => setViewState({ viewState: "isSettingsOverlay", state: false })}>
        <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
      </button>
      <div id="content-wrapper" class="flex">
        {#key selectedPage}
          {@const PageComponent = settingsPages[selectedPage]}
          <PageComponent />
        {/key}
      </div>
    </div>
  </div>
</div>

<style>
  #main-settings-overlay {
    position: fixed;
    z-index: 1000;
    inset: 0;
    justify-content: flex-start;
    padding: 0.5rem;
    background-color: var(--color-primary1);
    contain: layout style;
    overflow: hidden;

    #main-settings-overlay-gutter {
      margin: 0 4px;

      &::before {
        background-color: transparent;
      }

       &.highlight::before {
        background-color: var(--color-highlight1-dimmed);
      }
    }
  }

  #main-settings-overlay-sidebar {
    flex-shrink: 0;
    justify-content: flex-start;
    height: 100%;
    min-width: 200px;
    padding: 0.5rem;
    background-color: var(--color-primary2);
    border-radius: 0.5rem;
    overflow: hidden;
    will-change: width;

    #main-settings-overlay-sidebar-topbar {
      justify-content: space-between;
      width: 100%;
      padding: 0.5rem;
      gap: 0.5rem;
      border-bottom: 2px solid var(--color-secondary2);

      > div {
        gap: 0.5rem;
      }

      button {
        height: 2rem;
        border-radius: 0.25rem;
        font-weight: bold;
      }

      h3 {
        margin: 0;
      }
    }

    #main-settings-overlay-sidebar-content {
      justify-content: flex-start;
      align-items: flex-start;
      width: 100%;
      padding: 1rem 0;
      gap: 2px;
      overflow-y: auto;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));

      button.main-settings-overlay-sidebar-button {
        flex-shrink: 0;
        position: relative;
        justify-content: flex-start;
        width: 100%;
        height: 2rem;
        gap: 1rem;
        padding: 6px;
        border-radius: 0.25rem;

        &.selected-page {
          background-color: var(--color-highlight2);
        }
      }
    }
  }

  #main-settings-overlay-content {
    flex: 1 1 auto;
    height: 100%;
    border-radius: 0.5rem;
    background-color: var(--color-primary2);
    will-change: width;
    overflow: hidden;

    > div {
      position: relative;
      justify-content: flex-start;
      height: 100%;
      padding: 3.75rem;
      overflow-y: auto;
      overflow-x: auto;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));

      &::-webkit-scrollbar {
        height: 6px;
      }

      > button {
        position: absolute;
        right: 8px;
        top: 14px;
      }
    }

    #content-wrapper {
      justify-content: flex-start;
      max-width: 1360px;
      min-width: fit-content;
      width: 100%;
    }
  }
</style>