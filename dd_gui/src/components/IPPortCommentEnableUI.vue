<template>
  <a-row type="flex" :gutter="[8, 8]">
    <a-col flex="0 1 340px"
      ><ip-port-ui
        :obj_with_addr="this.remote_mut"
        :name="'addr'"
        :flag="this.$root.speed_show_mode"
        :speed="this.speed_out"
        :pkg_speed="this.pkg_speed_out"
    /></a-col>
    <a-col flex="auto">
      <a-typography-paragraph v-model:content="this.remote_mut.note" editable />
    </a-col>
    <a-col flex="0 0 50">
      <a-popconfirm
        title="Are you sure delete this remote addr?"
        ok-text="Yes"
        cancel-text="No"
        @confirm="
          (event) => {
            this.dis_mut.remote_addrs.splice(this.remote_key, 1);
            test();
            event.stopPropagation();
          }
        "
      >
        <a-button
          :size="'small'"
          @click="
            (event) => {
              event.stopPropagation();
            }
          "
        >
          <template #icon> <MinusOutlined /> </template>
          DELETE
        </a-button>
      </a-popconfirm>
    </a-col>

    <a-col flex="0 0 20">
      <a-switch
        size="small"
        :checked="this.remote_mut.enable"
        @click="
          (checked, event) => {
            this.remote_mut.enable = checked;
          }
        "
    /></a-col>
  </a-row>
</template>

<script>
import { MinusOutlined } from "@ant-design/icons-vue";
import IPPortUI from "./IPPortUI.vue";
export default {
  props: ["set_key", "dis_key", "remote_key"],
  setup() {},
  data() {
    return {
      ip_mut: this.ip,
    };
  },
  methods: {
    test() {
      console.log(this.remote_mut);
    },
  },
  computed: {
    dis_mut: function () {
      console.log(this.$root.group.vec[this.set_key].vec[this.dis_key]);
      return this.$root.group.vec[this.set_key].vec[this.dis_key];
    },
    remote_mut: function () {
      return this.dis_mut.remote_addrs[this.remote_key];
    },
    addr: function () {
      return this.remote_mut.addr;
    },
    ip: function () {
      return this.addr.split(":")[0];
    },
    port: function () {
      return this.addr.split(":")[1];
    },
    note: function () {
      return this.remote_mut.note;
    },
    enable: function () {
      return this.remote_mut.enable;
    },
    speed_out: function () {
      return this.remote_mut.speed_out;
    },
    pkg_speed_out: function () {
      return this.remote_mut.pkg_speed_out;
    },
  },
  components: {
    MinusOutlined,
    "ip-port-ui": IPPortUI,
  },
};
</script>