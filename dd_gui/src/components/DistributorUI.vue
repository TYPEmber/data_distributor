<template>
  <a-typography-paragraph>
    <ip-port-ui
      :ip="this.distributor.local_addr.split(':')[0]"
      :port="this.distributor.local_addr.split(':')[1]"
    ></ip-port-ui>
    <a-divider orientation="left" style="font-size: 14px">Send To</a-divider>
    <ip-port-ui
      v-for="addr in this.distributor.remote_addrs"
      :key="addr[0]"
      :ip="addr[0].split(':')[0]"
      :port="addr[0].split(':')[1]"
    ></ip-port-ui>
    <span :title="text">{{ this.text }}</span>
  </a-typography-paragraph>
</template>

<script>
import axios from "axios";
import IPPortUI from "./IPPortUI.vue";
export default {
  components: { "ip-port-ui": IPPortUI },
  props: ["distributor"],
  setup() {},
  data() {
    console.log(this.distributor);
    return {
      text: "",
    };
  },
  mounted() {
    this.timer = setInterval(() => {
      //console.log("get speed");

      let vector = [this.distributor.local_addr];
      for (let key of this.distributor.remote_addrs) {
        vector.push(key[0]);
      }
      //console.log(vector);
      axios.post("/api/speed/get", { vec: vector }).then((response) => {
        if (this.text != response.data) {
          this.text = response.data;
        }

        //this.$forceUpdate();
      });
    }, 1000);
  },
  beforeUnmount() {
    if (this.timer) {
      console.log(this.$root.group.map);

      this.$root.group.map[0].map[0].name = "dsdf";
      clearInterval(this.timer);
    }
  },
};
</script>