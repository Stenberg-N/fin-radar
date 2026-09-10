<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { setViewState } from "$lib/viewStore";
  import { userPrefs, updateUserPrefs } from "$lib/prefsStore";
  import { moveGutter, isGutterMoving } from "$lib/actions";
  import { t, lang } from "$lib/i18n/i18n";

  import ModalWrapper from "../ModalWrapper.svelte";
  import Account from "./settings-pages/Account.svelte";

  type PageName = "account";

  let selectedPage = $state<PageName>("account");
  const settingsPages = {
    "account": Account,
  };
  const settingsSidebarButtons = [
    {
      id: "account",
      img: "/user.svg",
      get title() { return $t["settings.pages.account.title"]; },
    },
  ];

  const sideBarWidth = $derived($userPrefs.settingsOverlayPrefs.sideBarWidth);
  let isHovering = $state(false);
  let timer: ReturnType<typeof setTimeout>;

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
    }
  };
</script>

<div id="main-settings-overlay" class="horizontal-flex-container" transition:fade={{ duration: 200, easing: cubicInOut }}>
  {#if $isGutterMoving || isHovering}
    <ModalWrapper options={{ position: { isContinuousUpdate: true, centerElement: true }, transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
      <p style="background-color: #222; margin: 0; padding: 8px;">{`${sideBarWidth}px`}</p>
    </ModalWrapper>
  {/if}

  <div id="main-settings-overlay-sidebar" class="vertical-flex-container" style="width: {sideBarWidth}px;">
    <div id="main-settings-overlay-sidebar-topbar" class="horizontal-flex-container">
      <h2>{$t["main.layout.settings"]}</h2>
      <button class="primary-button" style="width: 36px; font-weight: 600;" onclick={() => lang.set($lang === 'en' ? 'fi' : 'en')}>
        {$lang === 'en' ? 'EN' : 'FI'}
      </button>
    </div>
    <div id="main-settings-overlay-sidebar-content" class="vertical-flex-container">
      {#each settingsSidebarButtons as button, i (i)}
        <button class="main-settings-overlay-sidebar-button transparent-button-highlight" class:selected-page={selectedPage === button.id} onclick={() => setSelectedPage(button.id as PageName)}>
          <span class="span-icon img-medium" style="mask-image: url('{button.img}');"></span>
          {button.title}
        </button>
      {/each}
    </div>
  </div>

  <div role="slider" aria-valuenow={sideBarWidth} tabindex="0" id="main-settings-overlay-gutter" class="resize-gutter-default horizontal-flex-container" class:highlight={isHovering}
    use:moveGutter={{ onResize: (newWidth) => { updateUserPrefs("settingsOverlayPrefs", "sideBarWidth", newWidth); },  min: 200, max: 500 }}
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
  ></div>

  <div id="main-settings-overlay-content" class="vertical-flex-container">
    <button aria-label="Close settings" class="transparent-button-highlight" onclick={() => setViewState({ viewState: "isSettingsOverlay", state: false })}>
      <span class="span-icon img-small" style="mask-image: url('close-x.svg');"></span>
    </button>
    {#key selectedPage}
      {@const PageComponent = settingsPages[selectedPage]}
      <PageComponent />
    {/key}
  </div>
</div>

<style>
  #main-settings-overlay {
    position: fixed;
    z-index: 1000;
    inset: 0;
    background-color: #0f0f0f;
    contain: layout style;
  }

  #main-settings-overlay-sidebar {
    justify-content: flex-start;
    height: 100%;
    overflow: hidden;
    will-change: width;

    #main-settings-overlay-sidebar-topbar {
      justify-content: space-between;
      width: 100%;
      padding: 16px;
      padding-right: 8px;
      gap: 8px;
      border-bottom: 2px solid #333;

      button { height: 32px; }

      h2 {
        margin: 0;
      }
    }

    #main-settings-overlay-sidebar-content {
      justify-content: flex-start;
      align-items: flex-start;
      width: 100%;
      padding: 16px 4px 16px 10px;
      gap: 4px;
      margin-right: 6px;
      overflow-y: auto;
      scrollbar-gutter: stable both-edges;
      mask-image: linear-gradient(to top, rgba(0, 0, 0, 0), rgb(0, 0, 0) 1%, rgb(0, 0, 0) 99%, rgba(0, 0, 0, 0));

      button.main-settings-overlay-sidebar-button {
        flex-shrink: 0;
        position: relative;
        justify-content: flex-start;
        width: 100%;
        height: 40px;
        gap: 16px;
        padding: 8px 16px;
        font-size: 1rem;
        border-radius: 4px;

        &.selected-page {
          background-color: rgba(200, 200, 200, 0.2);
        }
      }
    }
  }

  #main-settings-overlay-content {
    position: relative;
    justify-content: flex-start;
    flex: 1 1 auto;
    height: 100%;
    padding: 60px;
    will-change: width;

    > button {
      position: absolute;
      right: 14px;
      top: 14px;
      width: 32px;
      height: 32px;
    }
  }
</style>