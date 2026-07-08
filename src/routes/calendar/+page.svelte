<script lang="ts">

</script>

<div id="calendar-main-container" class="horizontal-flex-container">
  <div id="calendar-event-container" class="vertical-flex-container">
    <p>placeholder</p>
  </div>

  <div id="calendar-grid" class="vertical-flex-container">
    <p>placeholder</p>
  </div>
</div>

<style>
  #calendar-main-container {
    width: 100%;
    height: 100%;
    flex-shrink: 0;

    > div {
      width: 100%;
      height: 100%;
    }

    #calendar-event-container {
      width: 300px;
    }
  }
</style>