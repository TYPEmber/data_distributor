<template>
  <div>
    <!-- <a-input size="large" v-model:value="this.set_mut.name" /> -->
    <a-row type="flex">
      <a-col flex="auto">
        <a-typography-title
          :level="3"
          v-model:content="this.set_mut.name"
          editable
        />
      </a-col>
      <a-space>
        <a-col>
          {{ this.speed_show }}
        </a-col>
        <a-col>
          <a-popconfirm
            title="Are you sure delete this set?"
            ok-text="Yes"
            cancel-text="No"
            @confirm="
              (event) => {
                //this.$root.group.vec.splice(this.set_key, 1);
                this.$root.remove_set(this.set_key);

                event.stopPropagation();
              }
            "
          >
            <a-button
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
        <a-col>
          <a-switch
            :checked="this.set_mut.enable"
            @click="
              (checked, event) => {
                this.set_mut.enable = checked;
              }
            "
        /></a-col>
      </a-space>
    </a-row>
    <a-collapse v-model:activeKey="activeKey">
      <a-collapse-panel
        v-for="(dis, dis_index) in this.set_mut.vec"
        :key="'set_' + dis_index"
        :header="dis.name"
      >
        <template #extra>
          <a-row :gutter="8">
            <a-col>
              <a-popconfirm
                title="Are you sure delete this distributor?"
                ok-text="Yes"
                cancel-text="No"
                @confirm="
                  (event) => {
                    this.set_mut.vec.splice(dis_index, 1);
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
            <a-col>
              <a-switch
                size="small"
                :checked="dis.enable"
                @click="
                  (checked, event) => {
                    dis.enable = checked;
                    event.stopPropagation();
                  }
                "
              />
            </a-col>
          </a-row>
        </template>
        <distributor-ui
          :set_key="this.set_key"
          :dis_key="dis_index"
        ></distributor-ui>
        <span :title="text">{{ this.text }}</span>
      </a-collapse-panel>
    </a-collapse>

    <br />
    <a-button
      @click="
        () => {
          this.set_mut.vec.push({
            name: 'dis_new',
            enable: false,
            note: 'no comment',
            local_addr: '',
            remote_addrs: [],
          });
        }
      "
    >
      <template #icon> <PlusOutlined /> </template>
      ADD NEW DISTRIBUTOR
    </a-button>
  </div>
</template>

<script>
import { PlusOutlined, MinusOutlined } from "@ant-design/icons-vue";
import { defineComponent, ref, watch } from "vue";
import DistributorUI from "./DistributorUI.vue";
export default defineComponent({
  props: ["set_key"],
  setup() {
    const activeKey = ref(["0"]);
    watch(activeKey, (val) => {
      console.log(val);
    });

    return {
      timer: null,
      activeKey,
    };
  },
  data: function () {
    return {
      text: "",
      //key_name_buf: this.set.name,
    };
  },
  computed: {
    set_mut: function () {
      console.log(this.set_key);
      let ref = this.$root.group.vec[this.set_key];
      // if (ref == undefined) {
      //   //this.$root.selectedKeys = ref([this.$root.group.vec.length - 1]);
      //   this.$root.reset_select();
      //   ref = this.$root.group.vec[this.$root.group.vec.length - 1];
      // }
      return ref;
    },
    speed_show: function () {
      let speed_in = 0;
      let pkg_speed_in = 0;
      let speed_out = 0;
      let pkg_speed_out = 0;
      for (let dis_key of this.set_mut.vec.keys()) {
        let dis = this.set_mut.vec[dis_key];
        if (dis.enable == false) {
          continue;
        }

        speed_in += new Number(dis.speed_in);
        pkg_speed_in += new Number(dis.pkg_speed_in);

        for (let remote_key of dis.remote_addrs.keys()) {
          let remote = dis.remote_addrs[remote_key];
          if (remote.enable == false) {
            continue;
          }

          speed_out += new Number(remote.speed_out);
          pkg_speed_out += new Number(remote.pkg_speed_out);
        }
      }

      if (!this.$root.speed_show_mode) {
        return "IN: " + speed_in + " OUT: " + speed_out;
      } else {
        return (
          "IN: " + pkg_speed_in + " pkg/s OUT: " + pkg_speed_out + " pkg/s"
        );
      }
    },
  },
  // mounted() {
  //   this.set_mut = this.$root.group.map.get(this.dis_key);
  //   console.log(this.dis_key);
  // },

  components: {
    PlusOutlined,
    MinusOutlined,
    "distributor-ui": DistributorUI,
  },
});
</script>
