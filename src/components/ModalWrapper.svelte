<script lang="ts">
  import { onDestroy, onMount, type Snippet } from "svelte";
  import { fade, slide, type TransitionConfig } from "svelte/transition";
  import { cubicInOut, cubicIn, cubicOut } from "svelte/easing";

  import { viewport } from "$lib/viewport";
  import { dragElement, draggedElement, handleClickOutside } from "$lib/actions.svelte";

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
    moveLeft?: number;
    moveTop?: number;

    left?: never;
    top?: never;
    isPositionAbsolute?: never;
    isDraggable?: never;
  } | {
    left?: number;
    top?: number;
    centerElement?: boolean;

    isContinuousUpdate?: never;
    isPositionAbsolute?: never;
    isDraggable?: never;
  } | {
    left?: number;
    top?: number;
    isPositionAbsolute?: boolean;

    isContinuousUpdate?: never;
    centerElement?: never;
    isDraggable?: never;
  } | {
    left?: number;
    top?: number;
    isDraggable?: boolean;

    isContinuousUpdate?: never;
    centerElement?: never;
    isPositionAbsolute?: never;
  };

  let {
    children,
    options,
  }: {
    children: Snippet<[]>;
    options?: {
      /**
       * Position options for the Wrapper component.
       * 
       * - `left`: The left position value for the wrapper.
       * - `top`: The top position value for the wrapper.
       * - `isPositionAbsolute`: Sets the position to absolute if `true`. Defaults to `false` and position to fixed.
       * - `isDraggable`: Determines if the wrapper should be draggable. Creates a drag bar to the top of the wrapper if set to `true`. Defaults to `false`
       * - `isContinuousUpdate`: Updates the wrapper position continuously if `true`. Defaults to `false`
       * - `centerElement`: Centers the wrapper horizontally to the cursor if `true`. Defaults to `false`
       * - `moveLeft`: Move the wrapper's left position from the current position of the cursor by the given value.
       * - `moveTop`: Move the wrapper's top position from the current position of the cursor by the given value.
       */
      position?: PositionOptions,
      /**
       * - `type`: Transition animation.
       * - `duration`: The duration for the transition.
       * - `easing`: Transition easing.
       * - `delay`: Delay the transition.
       * - `axis`: Determines the axis for the slide transition type.
       */
      transition?: TransitionOptions,
      outline?: { width: number, color: string };
      /**
       * Set the border radius for the wrapper. Give the value in pixels. Defaults to `16px`.
       */
      borderRadius?: number;
      ignorableEls?: (HTMLElement | null)[];
      onOutsideClick?: () => void;
      /**
       * Autofocus on the wrapper when it mounts.
       */
      focus?: boolean;
      dragHandleColor?: "lighter";
    },
  } = $props();

  const onOutsideClick = $derived(options?.onOutsideClick ?? (() => {}));

  let wrapperEl = $state<HTMLDivElement | null>(null);
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

    const moveLeft = options?.position && "moveLeft" in options.position && options?.position.moveLeft ? options?.position.moveLeft : 0;
    const moveTop = options?.position && "moveTop" in options.position && options?.position.moveTop ? options?.position.moveTop : 5;
    const centered = !!(options?.position && options.position.centerElement);

    const w = wrapperEl.clientWidth;
    const h = wrapperEl.clientHeight;
    const x = $viewport.cursorX + moveLeft;
    const y = $viewport.cursorY + moveTop;
    const xStart = x - (centered ? w / 2 : 0);
    const overflowsRight = xStart + w > $viewport.width;
    const overflowsBottom = y + h > $viewport.height;

    const left = overflowsRight
      ? clamp(x, 0, $viewport.width - w)
      : Math.max(xStart, 0);
    const top = overflowsBottom
      ? clamp(y - h, 0, $viewport.height - h)
      : Math.max(y, 0)

    wrapperEl.style.setProperty('--modal-wrapper-component-top', `${options?.position && options.position.top ? options.position.top : top}px`);
    wrapperEl.style.setProperty('--modal-wrapper-component-left', `${options?.position && options.position.left ? options.position.left : left}px`);

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

  const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

  const applyPosition = () => {
    if (!wrapperEl || !latestPosition) return;
    raf = null;

    const { cursorX, cursorY, viewportHeight, viewportWidth, isCentered } = latestPosition;
    const moveTop = options?.position && "moveTop" in options.position && options?.position.moveTop ? options?.position.moveTop : 5;
    const moveLeft = options?.position && "moveLeft" in options.position && options?.position.moveLeft ? options?.position.moveLeft : 0;

    const w = wrapperEl.clientWidth;
    const h = wrapperEl.clientHeight;
    const x = cursorX + moveLeft;
    const y = cursorY + moveTop;
    const xStart = x - (isCentered ? w / 2 : 0);
    const overflowsRight = xStart + w > viewportWidth;
    const overflowsBottom = y + h > viewportHeight;

    const left = overflowsRight
      ? clamp(x, 0, viewportWidth - w)
      : Math.max(xStart, 0)
    const top = overflowsBottom
      ? clamp(y, 0, viewportHeight - h)
      : Math.max(y, 0)

    wrapperEl.style.setProperty('--modal-wrapper-component-left', `${left}px`);
    wrapperEl.style.setProperty('--modal-wrapper-component-top', `${top}px`);
  };

  const dragApplyPosition = (top: number, left: number) => {
    wrapperEl?.style.setProperty('--modal-wrapper-component-left', `${left}px`);
    wrapperEl?.style.setProperty('--modal-wrapper-component-top', `${top}px`);
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

<!--
@component Component to wrap content with.

- If wrapping paragraph HTML elements, you can apply the class `nowrap` to the paragraph element to apply nowrap to the white-space property.
  This can helpful in situations where the wrapper is too close to the app window's edge and thus compresses the element and moves text to new rows.
-->
<div
  role="dialog"
  tabindex="0"
  bind:this={wrapperEl}
  class="modal-wrapper-component"
  class:dragged={draggedElement.isDragged}
  onkeydown={(e) => {
    switch (e.key) {
      case 'Escape': onOutsideClick(); break;
    }
  }}
  use:handleClickOutside={{ onOutsideClick: onOutsideClick, getAdditionalElements: () => options?.ignorableEls ?? [] }}
  transition:applyTransition
  style="
    position: {(options?.position && "isPositionAbsolute" in options.position && options.position.isPositionAbsolute) ? "absolute" : "fixed"};
    border-radius: {options?.borderRadius ? `${options.borderRadius}px` : '1rem'};
    {(options?.position && "isContinuousUpdate" in options.position && options.position.isContinuousUpdate)
      ? 'top: 0; left: 0; transform: translate3d(var(--modal-wrapper-component-left), var(--modal-wrapper-component-top), 0); will-change: transform;'
      : 'top: var(--modal-wrapper-component-top); left: var(--modal-wrapper-component-left); transform: none; will-change: unset;'
    }
  "
>
  {#if options?.position && "isDraggable" in options.position && options.position.isDraggable}
    <div id="drag-handle"
      role="button"
      tabindex="0"
      class="flex"
      style="
        background-color: var(--color-secondary{options?.dragHandleColor === "lighter" ? '3' : '2'});
        border-color: var(--outline-color{options?.dragHandleColor === "lighter" ? '3' : '2'});
      "
      use:dragElement={{ elToMove: wrapperEl, onMove: (top, left) => dragApplyPosition(top, left)}}
    >
      <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg');"></span>
    </div>
  {/if}
  {@render children()}
</div>

{#each [wrapperEl], i (i)}
  {onMount(() => { if (options?.focus !== false) wrapperEl?.focus(); })}
{/each}

<style>
  .modal-wrapper-component {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-width: fit-content;
    max-height: calc(100vh - 198px);
    z-index: 10000;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);

    :global(p.nowrap) {
      white-space: nowrap;
    }

    &:focus {
      outline: none;
    }

    #drag-handle {
      padding: 0.25rem 0;
      border-bottom: 1px solid;
    }

    &.dragged {
      cursor: grabbing;
    }
  }
</style>