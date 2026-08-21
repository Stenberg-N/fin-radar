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
  }

  let {
    children,
    options,
  }: {
    children: Snippet<[]>;
    options?: {
      /** Defaults to fixed positioning*/
      isPositionAbsolute?: boolean;
      position?: { left: number, top: number };
      transition?: TransitionOptions,
    },
  } = $props();

  let wrapperEl: HTMLDivElement;

  onMount(() => {
    if (!wrapperEl) return;

    if (options?.position) {
      wrapperEl.style.setProperty('--modal-wrapper-component-left', `${options.position.left}px`);
      wrapperEl.style.setProperty('--modal-wrapper-component-top', `${options.position.top}px`);
    } else {
      wrapperEl.style.setProperty('--modal-wrapper-component-left', `${$viewport.width < $viewport.cursorX + wrapperEl.clientWidth ? $viewport.cursorX - wrapperEl.clientWidth : $viewport.cursorX}px`);
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

<div bind:this={wrapperEl} class="modal-wrapper-component" style="position: {options?.isPositionAbsolute ? "absolute" : "fixed"};" transition:applyTransition>
  {@render children()}
</div>

<style>
  .modal-wrapper-component {
    top: var(--modal-wrapper-component-top);
    left: var(--modal-wrapper-component-left);
    max-width: fit-content;
    z-index: 500;
    border-radius: 8px;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }
</style>