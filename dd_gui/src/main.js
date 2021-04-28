import {createApp} from 'vue'

import { Layout, Menu, Space, Row, Col, Divider, Input, Switch, Button, Collapse, Message, Typography } from "ant-design-vue";

import App from './App.vue'

createApp(App)
    .use(Layout)
    .use(Menu)
    .use(Space)
    .use(Row)
    .use(Col)
    .use(Divider)
    .use(Input)
    .use(Switch)
    .use(Button)
    .use(Collapse)
    .use(Message)
    .use(Typography)
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