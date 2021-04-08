<template>
  <a-input-group compact style="width: 100%">
    <a-typography-paragraph>
      <a-input
        style="width: 80%"
        v-model:value="ip_input"
        placeholder="127.0.0.1"
      />
      <a-input
        style="width: 20%"
        v-model:value="port_input"
        placeholder="5503"
      />
    </a-typography-paragraph>
  </a-input-group>
</template>

<script>
export default {
  props: ["ip", "port"],
  setup() {},
  data() {
    console.log(this.ip);
    return {
      ip_input: this.ip,
      port_input: this.port,
    };
  },
  methods: {
    get() {
      return this.ip_input + ":" + this.port_input;
    },
  },
};
</script>