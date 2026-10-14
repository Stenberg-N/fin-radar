<script lang="ts">
  import { fly } from "svelte/transition";
  import { cubicInOut } from "svelte/easing";

  let {
    options
  }: {
    options: {
      parentWidth: number;
    }
  } = $props();
</script>

<div in:fly={{ x: -options.parentWidth / 2, duration: 400, easing: cubicInOut }}>
  placeholder
</div>

<style>

</style>