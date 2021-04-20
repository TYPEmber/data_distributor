<template>
  <a-layout>
    <a-layout-header
      class="header"
      theme="light"
      :style="{
        position: 'fixed',
        zIndex: 1,
        width: '100%',
      }"
    >
      <a-row type="flex" justify="center">
        <a-col :flex="4">
          <p />
          <a-typography-title :level="3" :style="{ color: '#fff' }">
            DataDistributor-V1.0.0.1
          </a-typography-title>
        </a-col>
        <a-col :flex="2"
          ><a-button
            type="primary"
            @click="
              () => {
                this.speed_show_mode = !this.speed_show_mode;
              }
            "
            >SHOW_M</a-button
          ></a-col
        >
        <a-col :flex="2"
          ><a-button type="primary" @click="stop_start(0)"
            >STOP</a-button
          ></a-col
        >
        <a-col :flex="2"
          ><a-button type="primary" @click="start_save()"
            >START_SAVE</a-button
          ></a-col
        >
        <a-col :flex="2"
          ><a-button type="primary" @click="stop_start(1)"
            >START</a-button
          ></a-col
        >
      </a-row>
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
          background: '#fff',
          overflow: 'auto',
          height: '100vh',
          position: 'fixed',
          left: 0,
        }"
      >
        <div class="logo" />
        <a-menu
          theme="light"
          mode="inline"
          @click="add"
          @select="select_proc"
          v-model:selectedKeys="selectedKeys"
          v-model:openKeys="openKeys"
          style="height: 100vh"
        >
          <a-menu-item
            v-for="(set, set_index) in this.group.vec"
            :key="set_index"
          >
            <ClusterOutlined />
            <span class="nav-text">{{ set.name }}</span>
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
            overflow: 'auto',
            //height: '100vh',
            //background: 'rgba(255,255,255,0.2)',
          }"
        >
          <template v-if="this.selectedKeys[0] == 'add'"> </template>
          <template v-else-if="this.selectedKeys[0] == 'help'"> </template>
          <set-ui v-else :set_key="this.selectedKeys[0]"></set-ui>
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
      uuid: () => {
        var temp_url = URL.createObjectURL(new Blob());
        var uuid = temp_url.toString(); // blob:https://xxx.com/b250d159-e1b6-4a87-9002-885d90033be3
        URL.revokeObjectURL(temp_url);
        return uuid.substr(uuid.lastIndexOf("/") + 1);
      },
      selectedKeys: ref(["help"]),
    };
  },
  mounted() {
    axios.post("/api/group/get", {}).then((response) => {
      console.log(response.data);
      // parse json
      this.group = JSON.parse(JSON.stringify(response.data));
      this.host_group = JSON.parse(JSON.stringify(response.data));

      // set timer for speed
      this.timer = setInterval(() => {
        //console.log("get speed");
        let vec_req = [];

        for (let [set_key, set] of this.host_group.vec.entries()) {
          if (set.enable == false) {
            continue;
          }
          for (let [dis_key, dis] of set.vec.entries()) {
            if (dis.enable == false) {
              continue;
            }
            vec_req.push({
              set_key: set_key,
              dis_key: dis_key,
              addr: "IN_" + dis.local_addr,
            });
            for (let [remote_key, remote] of dis.remote_addrs.entries()) {
              if (remote.enable == false) {
                continue;
              }
              vec_req.push({
                set_key: set_key,
                dis_key: dis_key,
                remote_key: remote_key,
                addr: "OUT_" + remote.addr,
              });
            }
          }
        }

        let vector = [];
        for (let item of vec_req.values()) {
          vector.push(item.addr);
        }

        axios.post("/api/speed/get", { vec: vector }).then((response) => {
          //this.$forceUpdate();
          for (let [key, speeds] of response.data.entries()) {
            let buf = speeds.split(" ");
            let speed = buf[0];
            let pkg_speed = buf[1];
            let set = this.group.vec[vec_req[key].set_key];
            if(set == undefined){
              continue;
            }
            let dis = set.vec[vec_req[key].dis_key];
            if (dis == undefined) {
              continue;
            }
            // send speed
            if (vec_req[key].remote_key != undefined) {
              dis.remote_addrs[vec_req[key].remote_key].speed_out = speed;
              dis.remote_addrs[
                vec_req[key].remote_key
              ].pkg_speed_out = pkg_speed;
            }
            // receive speed
            else {
              dis.speed_in = speed;
              dis.pkg_speed_in = pkg_speed;
            }
          }

          // for (let set_key of this.host_group.vec.keys()) {
          //   let set = this.host_group.vec[set_key];
          //   if (set.enable == false) {
          //     continue;
          //   }
          //   let speed_in = 0;
          //   let pkg_speed_in = 0;
          //   let speed_out = 0;
          //   let pkg_speed_out = 0;
          //   for (let dis_key of set.vec.keys()) {
          //     let dis = set.vec[dis_key];
          //     if (dis.enable == false) {
          //       continue;
          //     }
          //     speed_in += set.speed_in;
          //     pkg_speed_in += set.pkg_speed_in;

          //     for (let remote_key of dis.remote_addrs.keys()) {
          //       let remote = dis.remote_addrs[remote_key];
          //       if (remote.enable == false) {
          //         continue;
          //       }
          //       speed_out += remote.speed_out;
          //       pkg_speed_out += remote.pkg_speed_out;
          //     }
          //   }
          //   set.speed_in = speed_in;
          //   set.speed_out = speed_out;
          //   set.pkg_speed_in = pkg_speed_in;
          //   set.pkg_speed_out = pkg_speed_out;
          // }

          //console.log(this.host_group);
        });
      }, 1000);

      console.log(this.group);
    });
  },

  beforeUnmount() {
    if (this.timer) {
      clearInterval(this.timer);
    }
  },
  data() {
    return {
      speed_show_mode: false,
      group: { vec: [] },
      host_group: { vec: [] },
    };
  },
  methods: {
    add(e) {
      console.log(e);
      if (e.key == "add") {
        this.group.vec.push({
          name: "default_set",
          note: "no comment",
          enable: true,
          vec: [],
        });
      }
    },
    remove_set(set_key) {
      let index =
        this.group.vec.length - 1 - 1 >= 0
          ? this.group.vec.length - 1 - 1
          : "help";

      // caution!
      this.selectedKeys = [index];
      this.group.vec.splice(set_key, 1);
    },
    select_proc(e) {
      console.log(e);
      if (e.selectedKeys[0] == "add") {
        this.selectedKeys = ref([this.group.vec.length - 1]);
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

    start_save() {
      console.log(JSON.stringify(this.group));
      axios
        .post("/api/ctrl/start_save/", { vec: this.group.vec })
        .then((response) => {
          console.log(response);
          this.host_group = JSON.parse(JSON.stringify(this.group));
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
.site-layout-header {
  background: #cecece;
}

/* [data-theme="dark"] .site-layout .site-layout-background {
  background: #141414;
} */
</style>