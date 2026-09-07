<script lang="ts">
  import { fade } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  import { setViewState } from "$lib/viewStore";
  import { userPrefs, updateUserPrefs } from "$lib/prefsStore";
  import { moveGutter, isGutterMoving } from "$lib/actions";

  import ModalWrapper from "./ModalWrapper.svelte";

  let sideBarWidth = $derived($userPrefs.settingsOverlayPrefs.sideBarWidth);
  let isHovering = $state(false);
  let timer: ReturnType<typeof setTimeout>;

  const handleMouseEnter = () => {
    timer = setTimeout(() => { isHovering = true }, 300);
  };

  const handleMouseLeave = () => {
    clearTimeout(timer);
    isHovering = false;
  };
</script>

<div id="main-settings-overlay" class="horizontal-flex-container" transition:fade={{ duration: 300, easing: cubicInOut }}>
  {#if $isGutterMoving || isHovering}
    <ModalWrapper options={{ position: { isContinuousUpdate: true, centerElement: true }, transition: { type: "fade", duration: 200, easing: "cubic-in-out" } }}>
      <p style="background-color: #222; margin: 0; padding: 8px;">{`${sideBarWidth}px`}</p>
    </ModalWrapper>
  {/if}

  <div id="main-settings-overlay-sidebar" class="vertical-flex-container" style="width: {sideBarWidth}px;">
    <button onclick={() => setViewState({ viewState: "isSettingsOverlay", state: false })}>Close</button>
  </div>
  <div role="slider" aria-valuenow={sideBarWidth} tabindex="0" id="main-settings-overlay-gutter" class="horizontal-flex-container" class:highlight={isHovering}
    use:moveGutter={{ onResize: (newWidth) => { updateUserPrefs("settingsOverlayPrefs", "sideBarWidth", newWidth); },  min: 200, max: 500 }}
    onmouseenter={handleMouseEnter}
    onmouseleave={handleMouseLeave}
  ></div>
  <div id="main-settings-overlay-content" class="vertical-flex-container">

  </div>
</div>

<style>
  #main-settings-overlay-gutter {
    position: relative;
    height: 100%;
    width: 6px;

    &::before {
      position: absolute;
      content: '';
      width: 1px;
      height: calc(100% - 20px);
      background-color: #333;
      transition: background-color 300ms ease;
    }

    &.highlight::before {
      background-color: rgba(255, 70, 70, 0.8);
    }

    &.highlight::after {
      position: absolute;
      content: '';
      width: 3px;
      height: calc(100% - 20px);
      background-color: rgba(255, 70, 70, 0.2);
    }

    &:hover {
      cursor: e-resize;
    }
  }

  #main-settings-overlay {
    position: fixed;
    z-index: 1000;
    inset: 0;
    background-color: #0f0f0f;
    contain: layout style;
  }

  #main-settings-overlay-sidebar {
    height: 100%;
    will-change: width;
  }

  #main-settings-overlay-content {
    flex: 1 1 auto;
    height: 100%;
    will-change: width;
  }
</style>