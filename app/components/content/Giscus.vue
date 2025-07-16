<script setup>
import { ref, watch, onMounted } from "vue";

const props = defineProps({
    repo: {
        type: String,
        required: true,
    },
    repoId: {
        type: String,
        required: true,
    },
    category: {
        type: String,
        default: "General",
    },
    categoryId: {
        type: String,
        required: true,
    },
    mapping: {
        type: String,
        default: "pathname",
    },
    strict: {
        type: String,
        default: "0",
    },
    reactionsEnabled: {
        type: String,
        default: "1",
    },
    emitMetadata: {
        type: String,
        default: "0",
    },
    inputPosition: {
        type: String,
        default: "bottom",
    },
    theme: {
        type: String,
        default: "preferred_color_scheme",
    },
    lang: {
        type: String,
        default: "en",
    },
    loading: {
        type: String,
        default: "lazy",
    }
});

const vueGiscus = ref(null);

onMounted(() => {
    const giscus = document.createElement("script");
    giscus.async = true;
    giscus.setAttribute("src", "https://giscus.app/client.js");
    giscus.setAttribute("data-repo", props.repo);
    giscus.setAttribute("data-repo-id", props.repoId);
    giscus.setAttribute("data-category", props.category);
    giscus.setAttribute("data-category-id", props.categoryId);
    giscus.setAttribute("data-mapping", props.mapping);
    giscus.setAttribute("data-strict", props.strict);
    giscus.setAttribute("data-reactions-enabled", props.reactionsEnabled);
    giscus.setAttribute("data-emit-metadata", props.emitMetadata);
    giscus.setAttribute("data-input-position", props.inputPosition);
    giscus.setAttribute("data-theme", props.theme);
    giscus.setAttribute("data-lang", props.lang);
    giscus.setAttribute("data-loading", props.loading);
    giscus.setAttribute("crossorigin", "anonymous");

    vueGiscus.value.appendChild(giscus);
});

watch(
    () => props.theme,
    (newTheme) => {
        if (vueGiscus.value) {
            const iframe = vueGiscus.value.querySelector("iframe");
            if (iframe && iframe.contentWindow) {
                iframe.contentWindow.postMessage(
                    {
                        giscus: {
                            setConfig: {
                                theme: newTheme,
                            },
                        },
                    },
                    "https://giscus.app"
                );
            }
        }
    }
);
</script>

<template>
    <div class="vue-giscus" ref="vueGiscus">
        <!-- giscus script here -->
    </div>
</template>

<style>
.giscus {
    max-width: 100%;
}
</style>
