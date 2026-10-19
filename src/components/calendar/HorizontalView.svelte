<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  interface Props {
    /** The parent container's client width. */
    parentWidth: number;
  }

  let {
    parentWidth,
  }: Props = $props();
</script>

<div in:fly={{ x: -parentWidth / 2, duration: 400, easing: cubicInOut }}>
  placeholder
</div>

<style>

</style>