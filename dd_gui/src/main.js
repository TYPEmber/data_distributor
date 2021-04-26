import {createApp} from 'vue'
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/antd.css';
import App from './App.vue'

createApp(App)
    .use(Antd) 
    .mount('#app')
    

    // parse_raw_group(raw) {
    //     for (let key in raw.map) {
    //       let buf_set = raw.map[key];
    //       let dis_map = new Map();
    //       for (let dis_key in buf_set.map) {
    //         dis_map.set(dis_key, buf_set.map[dis_key]);
    //       }
    //       buf_set.map = dis_map;
    //       this.ggg.map.set(key, buf_set);
    //     }
  
    //     console.log(this.ggg);
    //   },