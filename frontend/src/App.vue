<template>
  <transition name="fade" appear>
    <div id="console" class="console">
      <div id="nav" class="nav">
        <span id="title" class="title">
          <span>
            r e l a y
          </span>
          <span id="url" class="url">
            server
            {{ url }}
            status
            {{ ready_state }}
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
    return {
      url: '',
      ready_state: ''
    }
  }
})
</script>

<style>
  .fade-enter-from {
    opacity: 0;
  }
  .fade-enter-active {
    transition: opacity 2.5s ease;
  }
  body {
    background: whitesmoke;
  }
  .hr {
    color: steelblue;
    grid-row-start: 2;
    width: 100%;
  }
  .console {
    background: gainsboro;
    border-radius: 5px;
    width: 85vw;
    height: 85vh;
    padding: 20px;
    margin: auto;
    box-shadow: 10px 10px 10px 0 rgba(white, white, white, 0.75);
    position: relative;
    display: grid;
    grid-template-rows: 30px 10px auto 50px;
    gap: 10px;
  }
  .console .nav {
    display: grid;
    grid-template-columns: 50% 35% auto;
    align-items: center;
    justify-items: stretch;
  }
  .nav .title {
    color: goldenrod;
    grid-column-start: 1;
  }
</style>
