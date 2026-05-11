<script lang="ts">
  let {
    title,
    content
  }: {
    title: string;
    content: string;
  } = $props();
</script>

<div class="note-container vertical-flex-container">
  <div class="note-title-container">
    <p>{title}</p>
  </div>
  <div class="note-content-container">
    <textarea>
      {content}
    </textarea>
  </div>
</div>

<style>
  .note-container {
    justify-content: flex-start;
    padding: 24px;
    background-color: #222;
  }
</style>