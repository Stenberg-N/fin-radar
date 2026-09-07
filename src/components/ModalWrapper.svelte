<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import { fade, slide, type TransitionConfig } from "svelte/transition";
  import { cubicInOut, cubicIn, cubicOut } from "svelte/easing";

  import { viewport } from "$lib/viewport";

  type TransitionOptions = {
    type: "slide";
    axis: "y" | "x";
    duration: number;
    delay?: number;
    easing?: "cubic-in-out" | "cubic-in" | "cubic-out" | undefined;
  } | {
    type: "fade";
    duration: number;
    delay?: number;
    easing?: "cubic-in-out" | "cubic-in" | "cubic-out" | undefined;
  };

  type PositionOptions = {
    left?: number;
    top?: number;
    isContinuousUpdate?: boolean;
    centerElement?: boolean;
  } | {
    left: number;
    top: number;
    isPositionAbsolute?: boolean;
  };

  let {
    children,
    options,
  }: {
    children: Snippet<[]>;
    options?: {
      position?: PositionOptions,
      transition?: TransitionOptions,
      outline?: { width: number, color: string };
    },
  } = $props();

  let wrapperEl: HTMLDivElement;

  onMount(() => {
    if (!wrapperEl) return;

    if (options?.position && options.position.left && options.position.top) {
      wrapperEl.style.setProperty('--modal-wrapper-component-left', `${options.position.left}px`);
      wrapperEl.style.setProperty('--modal-wrapper-component-top', `${options.position.top}px`);
    } else {
      if (options?.position && "centerElement" in options.position) {
        wrapperEl.style.setProperty('--modal-wrapper-component-left', `${($viewport.width < $viewport.cursorX + wrapperEl.clientWidth ? $viewport.cursorX - wrapperEl.clientWidth : ($viewport.cursorX - wrapperEl.clientWidth / 2))}px`);
      } else {
        wrapperEl.style.setProperty('--modal-wrapper-component-left', `${($viewport.width < $viewport.cursorX + wrapperEl.clientWidth ? $viewport.cursorX - wrapperEl.clientWidth : $viewport.cursorX)}px`);
      }
      wrapperEl.style.setProperty('--modal-wrapper-component-top', `${$viewport.height < $viewport.cursorY + wrapperEl.clientHeight ? $viewport.cursorY - wrapperEl.clientHeight : $viewport.cursorY}px`);
    }

    if (options?.outline) {
      wrapperEl.style.outline = `${options.outline.width}px solid ${options.outline.color}`;
    }
  });

  $effect(() => {
    if (options?.position && "isContinuousUpdate" in options.position) {
      if (options?.position && "centerElement" in options.position) {
        wrapperEl.style.setProperty('--modal-wrapper-component-left', `${($viewport.width < $viewport.cursorX + wrapperEl.clientWidth ? $viewport.cursorX - wrapperEl.clientWidth : ($viewport.cursorX - wrapperEl.clientWidth / 2))}px`);
      } else {
        wrapperEl.style.setProperty('--modal-wrapper-component-left', `${($viewport.width < $viewport.cursorX + wrapperEl.clientWidth ? $viewport.cursorX - wrapperEl.clientWidth : $viewport.cursorX)}px`);
      }
      wrapperEl.style.setProperty('--modal-wrapper-component-top', `${$viewport.height < $viewport.cursorY + wrapperEl.clientHeight ? $viewport.cursorY - wrapperEl.clientHeight : $viewport.cursorY}px`);
    }
  });

  const getEasing = (type: "cubic-in-out" | "cubic-in" | "cubic-out" | undefined) => {
    switch (type) {
      case "cubic-in-out": return cubicInOut;
      case "cubic-in": return cubicIn;
      case "cubic-out": return cubicOut;
      default: return undefined;
    }
  };

  const getTransition = (type: "fade" | "slide") => {
    switch (type) {
      case "fade": return fade;
      case "slide": return slide;
      default: return null;
    }
  };

  const applyTransition = (node: HTMLElement): TransitionConfig => {
    if (!options?.transition) return {};

    const transition = getTransition(options.transition.type);
    if (!transition) return {};

    return transition(node, {
      duration: options.transition.duration,
      delay: options.transition.delay,
      easing: getEasing(options.transition.easing),
      ...(options.transition.type === "slide" && {axis: options.transition.axis}),
    });
  };
</script>

<div bind:this={wrapperEl} class="modal-wrapper-component" style="position: {(options?.position && "isPositionAbsolute" in options.position && options.position.isPositionAbsolute) ? "absolute" : "fixed"};" transition:applyTransition>
  {@render children()}
</div>

<style>
  .modal-wrapper-component {
    top: var(--modal-wrapper-component-top);
    left: var(--modal-wrapper-component-left);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-width: fit-content;
    max-height: calc(100vh - 182px);
    z-index: 500;
    border-radius: 8px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }
</style>