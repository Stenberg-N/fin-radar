<script lang="ts">
  import { onDestroy, onMount, type Snippet } from "svelte";
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
    isContinuousUpdate?: boolean;
    centerElement?: boolean;
  } | {
    left?: number;
    top?: number;
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
  let raf: number | null = null;
  let latestPosition: {
    cursorX: number;
    cursorY: number;
    viewportWidth: number;
    viewportHeight: number;
    isCentered: boolean;
  } | null = null;

  onMount(() => {
    if (!wrapperEl) return;

    const w = wrapperEl.clientWidth;
    const h = wrapperEl.clientHeight;
    const left = !!(options?.position && "centerElement" in options.position)
      ? ($viewport.width < $viewport.cursorX + w ? $viewport.cursorX - w : ($viewport.cursorX - w / 2))
      : ($viewport.width < $viewport.cursorX + w ? $viewport.cursorX - w : $viewport.cursorX);
    const top = $viewport.height < $viewport.cursorY + h ? $viewport.cursorY - h : $viewport.cursorY + 5;

    if (options?.position && "left" in options.position && "top" in options.position && options.position.left && options.position.top) {
      wrapperEl.style.setProperty('--modal-wrapper-component-left', `${options.position.left}px`);
      wrapperEl.style.setProperty('--modal-wrapper-component-top', `${options.position.top}px`);
    } else {
      wrapperEl.style.setProperty('--modal-wrapper-component-left', `${left}px`);
      wrapperEl.style.setProperty('--modal-wrapper-component-top', `${top}px`);
    }

    if (options?.outline) {
      wrapperEl.style.outline = `${options.outline.width}px solid ${options.outline.color}`;
    }
  });

  onDestroy(() => {
    if (raf !== null) cancelAnimationFrame(raf);
  });

  $effect(() => {
    if (!(options?.position && "isContinuousUpdate" in options.position)) {
      latestPosition = null;
      return;
    }

    latestPosition = {
      cursorX: $viewport.cursorX,
      cursorY: $viewport.cursorY,
      viewportWidth: $viewport.width,
      viewportHeight: $viewport.height,
      isCentered: !!(options.position && "centerElement" in options.position),
    };

    if (raf === null) {
      raf = requestAnimationFrame(applyPosition);
    }
  });

  const applyPosition = () => {
    if (!wrapperEl || !latestPosition) return;
    raf = null;

    const { cursorX, cursorY, viewportHeight, viewportWidth, isCentered } = latestPosition;
    const w = wrapperEl.clientWidth;
    const h = wrapperEl.clientHeight;

    const left = isCentered
      ? (viewportWidth < cursorX + w ? cursorX - w : cursorX - w / 2)
      : (viewportWidth < cursorX + w ? cursorX - w : cursorX);
    const top = viewportHeight < cursorY + h ? cursorY - h : cursorY + 5;

    wrapperEl.style.setProperty('--modal-wrapper-component-left', `${left}px`);
    wrapperEl.style.setProperty('--modal-wrapper-component-top', `${top}px`);
  };

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

<div bind:this={wrapperEl} class="modal-wrapper-component" transition:applyTransition
  style="
    position: {(options?.position && "isPositionAbsolute" in options.position && options.position.isPositionAbsolute) ? "absolute" : "fixed"};
    top: {(options?.position && "isContinuousUpdate" in options.position && options.position.isContinuousUpdate) ? '0' : 'var(--modal-wrapper-component-top)'};
    left: {(options?.position && "isContinuousUpdate" in options.position && options.position.isContinuousUpdate) ? '0' : 'var(--modal-wrapper-component-left)'};
    transform: {(options?.position && "isContinuousUpdate" in options.position && options.position.isContinuousUpdate) ? 'translate3d(var(--modal-wrapper-component-left), var(--modal-wrapper-component-top), 0)' : ''};
    will-change: {(options?.position && "isContinuousUpdate" in options.position && options.position.isContinuousUpdate) ? 'transform' : ''};
  ">
  {@render children()}
</div>

<style>
  .modal-wrapper-component {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-width: fit-content;
    max-height: calc(100vh - 198px);
    z-index: 500;
    border-radius: 1rem;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);
  }
</style>