<script setup lang="ts">
import { onMounted } from "vue";
import { usePoco } from "./store";
import Dev from "./pages/dev/index.vue";

const poco = usePoco();

onMounted(async () => {
  await poco.setup("development");
});
</script>

<template>
  <RouterView></RouterView>
  <!-- <Suspense>
    <Dev></Dev>
  </Suspense> -->
</template>

<style scoped></style>
