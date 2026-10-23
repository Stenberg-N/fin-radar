<script lang="ts">
  import { type Snippet } from "svelte";
  import { cubicIn, cubicInOut, cubicOut } from "svelte/easing";
  import { type TransitionConfig, fade, slide, fly } from "svelte/transition";

  import { handleClickOutside } from "$lib/utils/actions.svelte";

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
  } | {
    type: "fly";
    x?: number;
    y?: number
    duration: number;
    delay?: number;
    easing?: "cubic-in-out" | "cubic-in" | "cubic-out" | undefined;
  };

  type PositionOptions = {
    pos?: "fixed" | "static";
    top?: number;
    bottom?: number;
    left?: number;
    right?: number;
  };

  interface Props {
    children: Snippet<[]>;
    /**
     * - `type`: Transition animation.
     * - `duration`: The duration for the transition.
     * - `easing`: Transition easing.
     * - `delay`: Delay the transition.
     * - `axis`: Determines the axis for the slide transition type.
     * - `x` & `y`: Determines the direction and length of the fly transition.
     */
    transition?: TransitionOptions;
    position?: PositionOptions;
    theme?: "default" | "lighter" | "lighter2";
    /** Used to set the border radius property on the component. Defaults to 8. */
    borderRadius?: number;
    /**
     * Used to set the padding property on the component. Defaults to 16.
     * 
     * Give an array of numbers for each side like you would apply them in CSS.
     * 
     * @example [8, 8, 16]
     * This would apply 8px to the top, 8px to the sides and 16px to the bottom.
     * 
     * @example [8]
     * This would apply 8px to all sides.
     * 
     * @example [8, 4]
     * This would apply 8px to top and bottom, and 4px to the sides.
     */
    padding?: [number?, number?, number?, number?];
    /** Used to set the gap property on the component. Defaults to 4. */
    gap?: number;
    /** Used to disable box shadow. Defaults to `true`. */
    isBoxShadow?: boolean;
    /** Used to pass a function for click outside handler. */
    onOutsideClick?: () => void;
    /** Used to pass a ignorable elements for click outside handler. */
    ignorableEls?: (HTMLElement | null)[];
    /** Sets width to 100% on all divs. Defaults to `true`. */
    allDivs100?: boolean;
  }

  let {
    children,
    theme = "default",
    borderRadius = 16,
    padding = [16],
    gap = 4,
    isBoxShadow = true,
    onOutsideClick = (() => {}),
    ignorableEls = [],
    transition: rawTransition,
    position = {},
    allDivs100 = true,
  }: Props = $props();

  const {
    pos = "static",
  } = $derived(position);

  const themes = {
    default: {
      bg: "var(--color-secondary1)",
      outline: "var(--outline-color1)"
    },
    lighter: {
      bg: "var(--color-secondary2)",
      outline: "var(--outline-color2)"
    },
    lighter2: {
      bg: "var(--color-secondary3)",
      outline: "var(--outline-color3)"
    }
  } as const;
  const t = $derived(themes[theme] ?? themes["default"]);

  const getEasing = (type: TransitionOptions["easing"]) => {
    switch (type) {
      case "cubic-in-out": return cubicInOut;
      case "cubic-in": return cubicIn;
      case "cubic-out": return cubicOut;
      default: return undefined;
    }
  };

  const getTransition = (type: TransitionOptions["type"]) => {
    switch (type) {
      case "fade": return fade;
      case "slide": return slide;
      case "fly": return fly;
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
      ...(rawTransition.type === "slide" && { axis: rawTransition.axis }),
      ...(rawTransition.type === "fly" && {
        x: rawTransition.x,
        y: rawTransition.y
      }),
    });
  };
</script>

<div
  class="list-component flex column"
  style:--bg={t.bg}
  style:--outline={t.outline}
  style:--border-radius={`${borderRadius}px`}
  style:--padding={padding.filter((num) => num !== undefined).map((num) => `${num}px`).join(" ")}
  style:--gap={`${gap}px`}
  style:--box-shadow={isBoxShadow ? '0 8px 16px rgba(0, 0, 0, 0.8)' : ''}
  style:--position={pos}
  style:--top={position.top ? `${position.top}px` : ''}
  style:--bottom={position.bottom ? `${position.bottom}px` : ''}
  style:--right={position.right ? `${position.right}px` : ''}
  style:--left={position.left ? `${position.left}px` : ''}
  style:--divs-100={allDivs100 ? '100%' : 'auto'}
  use:handleClickOutside={{ onOutsideClick: () => onOutsideClick(), getAdditionalElements: () => ignorableEls }}
  transition:applyTransition
>
  {@render children()}
</div>

<style>
  .list-component {
    position: var(--position);
    top: var(--top);
    bottom: var(--bottom);
    left: var(--left);
    right: var(--right);
    z-index: 1000;
    align-items: flex-start;
    border-radius: var(--border-radius);
    background-color: var(--bg);
    outline: 1px solid var(--outline);
    padding: var(--padding);
    gap: var(--gap);
    border: none;
    box-shadow: var(--box-shadow);

    :global(div) {
      width: var(--divs-100);
    }

    :global(.button-primary) {
      height: 2rem;
    }
  }
</style>