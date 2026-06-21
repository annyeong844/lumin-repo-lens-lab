use crate::support::edge::{assert_edge, assert_reexport_pair};
use crate::support::scan::scan_ok;

pub fn assert_semicolonless_nuxt_barrel_reexports() {
    let source = [
        "import '../../dist/app/types/augments'",
        "",
        "export { createNuxtApp, useNuxtApp } from './nuxt'",
        "export type { NuxtApp, RuntimeNuxtHooks } from './nuxt'",
        "export { useAsyncData, useFetch } from './composables/index'",
        "export type { AsyncData, UseFetchOptions } from './composables/index'",
    ]
    .join("\n");
    let edges = scan_ok("packages/nuxt/src/app/index.ts", &source, 5);

    assert_edge(&edges, "../../dist/app/types/augments", false, false, false);
    assert_reexport_pair(&edges, "./nuxt");
    assert_reexport_pair(&edges, "./composables/index");
}
