import { mount } from "svelte";
import App from "./App.svelte";

const target = document.getElementById("canvas-app");
if (target) {
  mount(App, { target });
}
