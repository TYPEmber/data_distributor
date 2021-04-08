<template>
  <a-layout>
    <a-layout-header
      class="header"
      :style="{ position: 'fixed', zIndex: 1, width: '100%' }"
    >
      <!-- <div>DataDistributor-V1.0.0.1</div> -->
      <a-button type="primary" @click="stop_start(0)">STOP</a-button>
      <a-button type="primary" @click="stop_start(1)">START</a-button>
    </a-layout-header>
    <a-layout-content
      :style="{
        padding: '0 50px',
        marginTop: '64px',
        //height: '100vh',
        //background: 'rgba(255,255,255,0.2)',
      }"
    >
      <a-layout-sider
        :style="{
          overflow: 'auto',
          height: '100vh',
          position: 'fixed',
          left: 0,
        }"
      >
        <div class="logo" />
        <a-menu
          mode="inline"
          @click="add"
          @select="select_proc"
          v-model:selectedKeys="selectedKeys"
          v-model:openKeys="openKeys"
          style="height: 100%"
        >
          <a-menu-item v-for="item in items.values()" :key="'set_' + item.name">
            <ClusterOutlined />
            <span class="nav-text">{{ item.name }}</span>
          </a-menu-item>

          <a-menu-item key="add">
            <AppstoreAddOutlined />
            <span class="nav-text">ADD</span>
          </a-menu-item>
          <a-menu-item key="help">
            <QuestionCircleOutlined />
            <span class="nav-text">HELP</span>
          </a-menu-item>
        </a-menu>
      </a-layout-sider>
      <a-layout :style="{ marginLeft: '200px' }">
        <a-layout-content
          :style="{
            margin: '24px 0px 0',
            overflow: 'initial',
            //height: '100vh',
            //background: 'rgba(255,255,255,0.2)',
          }"
        >
          <template v-if="selectedKeys[0] == 'add'"> </template>
          <template v-else-if="selectedKeys[0] == 'help'"> </template>
          <set-ui v-else :set="items.get(selectedKeys[0])"></set-ui>
        </a-layout-content>
      </a-layout>
    </a-layout-content>
  </a-layout>
</template>
<script>
import {
  AppstoreAddOutlined,
  ClusterOutlined,
  QuestionCircleOutlined,
} from "@ant-design/icons-vue";
import { defineComponent, ref } from "vue";
import SetUI from "./components/SetUI.vue";
import axios from "axios";
// import { MenuItem } from "./MenuItem.vue";
// const MenuItem = {
//   data: function () {
//     return {
//       count: 0,
//     };
//   },
//   template:
//     '<button v-on:click="count++">You clicked me {{ count }} times.</button>',
// };

export default defineComponent({
  setup() {
    return {
      selectedKeys: ref(["help"]),
    };
  },
  mounted() {
    axios.post("/api/group/get", {}).then((response) => {
      console.log(response.data);
      this.group = JSON.parse(JSON.stringify(response.data));
      console.log(this.group);

      //this.items.push(this.group.map.get("default_set"));
      for (let set in this.group.map) {
        this.items.set("set_" + this.group.map[set].name, this.group.map[set]);
        console.log(this.group.map[set]);
        this.item_count++;
      }
    });
  },
  data() {
    // let json =
    //   '{ "sites" : [' +
    //   '{ "name":"Runoob" , "url":"www.runoob.com" },' +
    //   '{ "name":"Google" , "url":"www.google.com" },' +
    //   '{ "name":"Taobao" , "url":"www.taobao.com" } ]}';
    // console.log(JSON.parse(json));

    // function Group() {
    //   this.map = {
    //     key: {
    //       name: "",
    //       notes: "",
    //       enable: false,
    //       local_addr: "",
    //       remote_addrs: [{ addr: "", comment: "", enable: false }],
    //     },
    //   };]
    // }

    return {
      // group: {
      //   map: {
      //     key: {
      //       name: "",
      //       notes: "",
      //       enable: false,
      //       local_addr: "",
      //       remote_addrs: [{ addr: "", comment: "", enable: false }],
      //     },
      //   },
      // },
      group: new Map(),
      item_count: 1,
      items: new Map(),
    };
  },
  methods: {
    add(e) {
      if (this.item_count == e.item.index + 1) {
        // console.log(this.item_count);
        // console.log(e);
        // console.log(e.item.index);
        // console.log(MenuItem);
        this.items.set("set_" + e.item.index, { map: {}, name: e.item.index });
        this.item_count++;
      }
    },
    select_proc() {
      if (this.selectedKeys[0] == "add") {
        let buf = this.item_count - 2;
        this.selectedKeys = ref(["set_" + buf]);
      }
      console.log(this.selectedKeys);
    },

    stop_start(order) {
      let o = "";
      if (order == 0) {
        o = "stop";
      } else {
        o = "start";
      }
      axios.post("/api/ctrl/" + o, {}).then((response) => {
        console.log(response);
      });
    },
  },

  components: {
    ClusterOutlined,
    AppstoreAddOutlined,
    QuestionCircleOutlined,
    //"menu-item": MenuItem,
    "set-ui": SetUI,
  },
});
</script>
<style>
#components-layout-demo-fixed-sider .logo {
  height: 32px;
  background: rgba(255, 255, 255, 0.2);
  margin: 16px;
}
.site-layout .site-layout-background {
  background: #fff;
}
body {
  height: 100vh;
  background: #f0f2f5;
}

[data-theme="dark"] .site-layout .site-layout-background {
  background: #141414;
}
</style>