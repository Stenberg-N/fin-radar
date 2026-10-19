<script lang="ts">
  import { onDestroy, onMount, type Snippet } from "svelte";
  import { fade, slide, type TransitionConfig } from "svelte/transition";
  import { cubicInOut, cubicIn, cubicOut } from "svelte/easing";

  import { viewport } from "$lib/utils/viewport";
  import { dragElement, draggedElement, handleClickOutside, portalElement } from "$lib/utils/actions.svelte";

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
    usePortal?: never;
  } | {
    left?: number;
    top?: number;
    centerElement?: boolean;
    usePortal?: boolean;

    isContinuousUpdate?: never;
    isPositionAbsolute?: never;
    isDraggable?: never;
    moveLeft?: never;
    moveTop?: never;
  } | {
    left?: number;
    top?: number;
    isPositionAbsolute?: boolean;
    centerElement?: boolean;
    usePortal?: boolean;

    isContinuousUpdate?: never;
    isDraggable?: never;
    moveLeft?: never;
    moveTop?: never;
  } | {
    left?: number;
    top?: number;
    isDraggable?: boolean;
    usePortal?: boolean;

    isContinuousUpdate?: never;
    centerElement?: never;
    isPositionAbsolute?: never;
    moveLeft?: never;
    moveTop?: never;
  };

  interface Props {
    children: Snippet<[]>;
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
     * Autofocus on the wrapper when it mounts. Defaults to `true`.
     */
    focus?: boolean;
    dragHandleColor?: "default" | "lighter" | "darker";
    /**
     * Pass attributes to attach onto the wrapper component.
     * - If using as a HoverTitle element, attach the following as attributes: { "hover-title-owner": `hover.id` }. This ensures the HoverTitle will not linger.
     */
    attributes?: Record<string, string>;
    getSelf?: (self: HTMLDivElement | null) => void;
  }

  let {
    children,
    position = {},
    transition: rawTransition,
    outline = undefined,
    borderRadius = undefined,
    ignorableEls = [],
    onOutsideClick = (() => {}),
    focus = true,
    dragHandleColor = "default",
    attributes,
    getSelf,
  }: Props = $props();

  const {
    moveTop = 5,
    moveLeft = 0,
    isPositionAbsolute = false,
    centerElement = false,
    isContinuousUpdate = false,
    isDraggable = false,
    usePortal = false
  } = $derived(position);

  const portal = $derived(usePortal ? portalElement : (() => {}));

  const dragHandleColors = {
    darker: {
      bg: "var(--color-secondary1)",
      outline: "var(--outline-color1)",
    },
    lighter: {
      bg: "var(--color-secondary3)",
      outline: "var(--outline-color3)",
    },
    default: {
      bg: "var(--color-secondary2)",
      outline: "var(--outline-color2)",
    }
  };
  const dh = $derived(dragHandleColors[dragHandleColor] ?? dragHandleColors.default);

  let wrapperEl = $state<HTMLDivElement | null>(null);
  let raf: number | null = null;
  let latestPosition: {
    cursorX: number;
    cursorY: number;
    viewportWidth: number;
    viewportHeight: number;
    isCentered: boolean;
    moveTop: number;
    moveLeft: number;
  } | null = null;

  onMount(() => {
    if (!wrapperEl) return;

    const w = wrapperEl.clientWidth;
    const h = wrapperEl.clientHeight;
    const x = position.left !== undefined ? position.left : $viewport.cursorX + moveLeft;
    const y = position.top !== undefined ?position.top : $viewport.cursorY + moveTop;
    const xStart =  x - (centerElement ? w / 2 : 0);
    const overflowsRight = xStart + w > $viewport.width;
    const overflowsBottom = y + h > $viewport.height;

    const left = overflowsRight
      ? clamp(x, 0, $viewport.width - w - 8)
      : Math.max(xStart, 0);
    const top = overflowsBottom
      ? clamp(y - h, 0, $viewport.height - h)
      : Math.max(y, 0);

    wrapperEl.style.setProperty('--modal-wrapper-component-top', `${top}px`);
    wrapperEl.style.setProperty('--modal-wrapper-component-left', `${left}px`);

    if (outline) {
      wrapperEl.style.outline = `${outline.width}px solid ${outline.color}`;
    }
  });

  onDestroy(() => {
    if (raf !== null) cancelAnimationFrame(raf);
  });

  $effect(() => {
    if (!isContinuousUpdate) {
      latestPosition = null;
      return;
    }

    latestPosition = {
      cursorX: $viewport.cursorX,
      cursorY: $viewport.cursorY,
      viewportWidth: $viewport.width,
      viewportHeight: $viewport.height,
      isCentered: centerElement,
      moveTop,
      moveLeft,
    };

    if (raf === null) {
      raf = requestAnimationFrame(applyPosition);
    }

    return () => {
      if (raf) cancelAnimationFrame(raf);
      raf = null;
      latestPosition = null;
    }
  });

  $effect(() => {
    if (wrapperEl !== null) getSelf?.(wrapperEl);

    return () => {
      getSelf?.(null);
    };
  });

  const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

  const applyPosition = () => {
    if (!wrapperEl || !latestPosition) return;
    raf = null;

    const { cursorX, cursorY, viewportHeight, viewportWidth, isCentered, moveLeft, moveTop } = latestPosition;

    const w = wrapperEl.clientWidth;
    const h = wrapperEl.clientHeight;
    const x = cursorX + moveLeft;
    const y = cursorY + moveTop;
    const xStart = x - (isCentered ? w / 2 : 0);
    const overflowsRight = xStart + w > viewportWidth;
    const overflowsTop = 0 > y;

    const left = overflowsRight
      ? clamp(x, 0, viewportWidth - w)
      : Math.max(xStart, 0)
    const top = overflowsTop
      ? clamp(cursorY + 20, 0, viewportHeight - h)
      : Math.max(y, 0)

    wrapperEl.style.transform = `translate3d(${left}px, ${top}px, 0)`;
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
    if (!rawTransition) return {};

    const transition = getTransition(rawTransition.type);
    if (!transition) return {};

    return transition(node, {
      duration: rawTransition.duration,
      delay: rawTransition.delay,
      easing: getEasing(rawTransition.easing),
      ...(rawTransition.type === "slide" && {axis: rawTransition.axis}),
    });
  };
</script>

<!--
@component Component to wrap content with.

- If wrapping paragraph HTML elements, you can apply the class `nowrap` to the paragraph element to apply nowrap to the white-space property.
  This can helpful in situations where the wrapper is too close to the app window's edge and thus compresses the element and moves text to new rows.
- If using as a HoverTitle element, attach the following as attributes to the component: { "hover-title-owner": `hover.id` }. This ensures the HoverTitle will not linger.
-->
<div
  role="dialog"
  tabindex="0"
  bind:this={wrapperEl}
  class="modal-wrapper-component"
  class:dragged={draggedElement.isDragged}
  {...attributes}
  onkeydown={(e) => {
    switch (e.key) {
      case 'Escape': onOutsideClick(); break;
    }
  }}
  use:handleClickOutside={{ onOutsideClick: onOutsideClick, getAdditionalElements: () => ignorableEls }}
  use:portal
  transition:applyTransition
  style="
    position: {isPositionAbsolute ? "absolute" : "fixed"};
    border-radius: {borderRadius ? `${borderRadius}px` : '1rem'};
    {isContinuousUpdate
      ? 'top: 0; left: 0; transform: translate3d(var(--modal-wrapper-component-left), var(--modal-wrapper-component-top), 0); will-change: transform;'
      : 'top: var(--modal-wrapper-component-top); left: var(--modal-wrapper-component-left); transform: none; will-change: top left;'
    }
  "
>
  {#if isDraggable}
    <div id="drag-handle"
      role="button"
      tabindex="0"
      class="flex"
      style="
        background-color: {dh.bg};
        border-color: {dh.outline};
      "
      use:dragElement={{ elToMove: wrapperEl, onMove: (top, left) => dragApplyPosition(top, left)}}
    >
      <span class="span-icon img-small" style="mask-image: url('/grip-dots.svg');"></span>
    </div>
  {/if}
  {@render children()}
</div>

{#each [wrapperEl], i (i)}
  {onMount(() => { if (focus === true) wrapperEl?.focus(); })}
{/each}

<style>
  .modal-wrapper-component {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-width: fit-content;
    max-height: calc(100vh - 198px);
    height: fit-content;
    z-index: 10000;
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.8);

    :global(p.nowrap) {
      white-space: nowrap;
    }

    :global(div.modal-default.item-selector) {
      border-radius: 0;
      padding: 0;
      max-height: calc(32px * 16);
      overflow-y: auto;

      :global(button.button-primary) {
        border-radius: 0;
        min-width: 3rem;
      }

      :global(p.item-selector-title) {
        margin: 0;
        padding: 0.25rem 1rem;
        background-color: var(--color-secondary1-a);
        text-align: center;
      }
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