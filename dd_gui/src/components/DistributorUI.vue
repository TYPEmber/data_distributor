<template>
  <div>
    <a-typography-title
      :level="5"
      v-model:content="this.dis_mut.name"
      editable
    />
    <ip-port-ui
      :obj_with_addr="this.dis_mut"
      :name="'local_addr'"
      :flag="this.$root.speed_show_mode"
      :speed="this.speed_in"
      :pkg_speed="this.pkg_speed_in"
    ></ip-port-ui>
    <a-divider orientation="left" style="font-size: 14px">Send To</a-divider>
    <a-row
      type="flex"
      :gutter="8"
      :key="'base_row' + remote_key"
      v-for="(remote, remote_key) in this.dis_mut.remote_addrs"
    >
      <template v-if="remote_key % 2 == 0">
        <a-col flex="auto">
          <ip-port-comment-enable-ui
            :key="remote_key"
            :set_key="this.set_key"
            :dis_key="this.dis_key"
            :remote_key="remote_key"
          ></ip-port-comment-enable-ui>
        </a-col>

        <a-col
          flex="auto"
          v-if="remote_key + 1 < this.dis_mut.remote_addrs.length"
        >
          <ip-port-comment-enable-ui
            :key="remote_key + 1"
            :set_key="this.set_key"
            :dis_key="this.dis_key"
            :remote_key="remote_key + 1"
          ></ip-port-comment-enable-ui>
        </a-col>
        <!-- <a-col v-else :flex="12"> </a-col> -->
      </template>
    </a-row>
    <!-- <ip-port-comment-enable-ui
      v-for="(addr, remote_key) in this.dis_mut.remote_addrs"
      :key="remote_key"
      :set_key="this.set_key"
      :dis_key="this.dis_key"
      :remote_key="remote_key"
    ></ip-port-comment-enable-ui> -->
    <br />
    <a-button
      @click="
        () => {
          this.dis_mut.remote_addrs.push({
            enable: false,
            note: 'no comment',
            addr: '',
          });
        }
      "
    >
      <template #icon> <PlusOutlined /> </template>
      ADD NEW REMOTE ADDR
    </a-button>
  </div>
</template>

<script>
import { PlusOutlined } from "@ant-design/icons-vue";
import IPPortCommentEnableUI from "./IPPortCommentEnableUI";
import IPPortUI from "./IPPortUI.vue";
export default {
  components: {
    PlusOutlined,
    "ip-port-comment-enable-ui": IPPortCommentEnableUI,
    "ip-port-ui": IPPortUI,
  },
  props: ["set_key", "dis_key"],
  setup() {},
  data() {
    console.log(this.dis_mut);
    return {
      text: "",
    };
  },
  computed: {
    dis_mut: function () {
      console.log(this.$root.group.vec[this.set_key].vec[this.dis_key]);
      return this.$root.group.vec[this.set_key].vec[this.dis_key];
    },
    //enable: this.dis_mut.enable,
    local_addr: function () {
      return this.dis_mut.local_addr;
    },
    remote_addrs: function () {
      return this.dis_mut.remote_addrs;
    },
    name: function () {
      return this.dis_mut.name;
    },
    comment: function () {
      return this.dis_mut.comment;
    },
    speed_in: function () {
      return this.dis_mut.speed_in;
    },
    pkg_speed_in: function () {
      return this.dis_mut.pkg_speed_in;
    },
  },
};
</script>