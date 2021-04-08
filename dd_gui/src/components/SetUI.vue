<template>
  <div>
    <a-collapse v-model:activeKey="activeKey">
      <a-collapse-panel
        v-for="distributor in set.map"
        :key="distributor.name"
        :header="distributor.name"
      >
        <template #extra
          ><a-switch
            size="small"
            :checked="distributor.enable"
            @click="
              (checked, event) => {
                event.stopPropagation();
              }
            "
        /></template>
        <distributor-ui :distributor="distributor"></distributor-ui>
        <span :title="text">{{ this.text }}</span>
      </a-collapse-panel>
      <a-collapse-panel
        key="2"
        header="This is panel header 2"
        :disabled="false"
      >
        <p>{{ text }}</p>
      </a-collapse-panel>
      <a-collapse-panel key="3" header="This is panel header 3" disabled>
        <p>{{ text }}</p>
      </a-collapse-panel>
    </a-collapse>
  </div>
</template>

<script>
import { defineComponent, ref, watch } from "vue";
import DistributorUI from "./DistributorUI.vue";
export default defineComponent({
  props: ["set"],
  setup() {
    const activeKey = ref(["1"]);
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
    };
  },

  components: {
    "distributor-ui": DistributorUI,
  },
});
</script>
