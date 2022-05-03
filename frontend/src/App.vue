<template>
  <transition name="fade" appear>
    <div id="console" class="console">
      <div id="nav" class="nav">
        <span id="title" class="title">
          <span>
            r e l a y
          </span>
        </span>
      </div>
      <hr class="hr">
      <div id="messages" class="messages"></div>
      <div id="base" class="base"></div>
    </div>
  </transition>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  name: 'app',
  data () {
    return {}
  }
})
</script>

<style>
  body {
    background: whitesmoke;
  }
</style>
