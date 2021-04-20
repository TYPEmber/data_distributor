<template>
  <a-row type="flex" :gutter="8">
    <a-col flex="0 1 140px">
      <a-input v-model:value="ip_input" placeholder="127.0.0.1"
    /></a-col>
    <a-col flex="0 1 80px">
      <a-input v-model:value="port_input" placeholder="5503"
    /></a-col>
    <a-col flex="0 1 120px"> {{ this.speed_show }}</a-col>
  </a-row>
</template>

<script>
import { toRef } from "vue";
export default {
  props: {
    obj_with_addr: {},
    name: String,
    flag: Boolean,
    speed: Number,
    pkg_speed: Number,
  },
  setup(props) {
    //console.log([props]);
    return {
      speed_unit_list: [
        "B/s",
        "KB/s",
        "MB/s",
        "GB/s",
        "TB/s",
        "PB/s",
        "EB/s",
        "ZB/s",
        "YB/s",
        "BB/s",
        "NB/s",
        "DB/s",
      ],
      addr_ref: toRef(props.obj_with_addr, props.name),
    };
  },
  data() {
    return {
      addr_reactive: this.addr_ref,
      ip_input: this.addr_ref.split(":")[0],
      port_input: this.addr_ref.split(":")[1],
    };
  },
  methods: {
    get() {
      return this.ip_input + ":" + this.port_input;
    },
  },
  watch: {
    ip_input(val) {
      console.log(val);
      this.addr_ref = val + ":" + this.port_input;
    },
    port_input(val) {
      console.log(val);
      this.addr_ref = this.ip_input + ":" + val;
    },
    obj_with_addr(val) {
      console.log(val);
      this.addr_ref = toRef(val, this.name);
      this.ip_input = this.addr_ref.split(":")[0];
      this.port_input = this.addr_ref.split(":")[1];
    },
  },
  computed: {
    speed_show: function () {
      if (!this.flag) {
        let speed = this.speed / 8.0;
        let count = 0;
        while (speed / 1024.0 >= 1) {
          speed = speed / 1024.0;
          count++;
        }
        return speed.toPrecision(6) + this.speed_unit_list[count];
      } else {
        return this.pkg_speed + " pkg/s";
      }
    },
  },
};
</script>