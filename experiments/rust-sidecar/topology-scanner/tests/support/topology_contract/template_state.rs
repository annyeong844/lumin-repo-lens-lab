use crate::support::scan::scan_ok;

pub fn assert_nested_template_interpolation_without_escaped_backticks_matches_js_oracle() {
    let source = [
        "import { joinURL } from 'ufo';",
        "const path = `${routes.map(route => `${route}/payload.json`).join(',')}`;",
    ]
    .join("\n");
    let edges = scan_ok("packages/nuxt/src/app/composables/router.ts", &source, 1);

    assert_eq!(edges[0].source, "ufo");
}

pub fn assert_single_conditional_escaped_backtick_template_does_not_leak_state() {
    let source = [
        "export const useRoute: typeof _useRoute = () => {",
        "  if (import.meta.dev && !getCurrentInstance() && isProcessingMiddleware()) {",
        "    const middleware = useNuxtApp()._processingMiddleware",
        "    const trace = getUserTrace().map(({ source, line, column }) => `at ${source}:${line}:${column}`).join('\\n')",
        "    console.warn(`[nuxt] \\`useRoute\\` was called within middleware${typeof middleware === 'string' ? ` (\\`${middleware}\\`)` : ''}. This may lead to misleading results. Instead, use the (to, from) arguments passed to the middleware to access the new and old routes. Learn more: https://nuxt.com/docs/4.x/directory-structure/app/middleware#accessing-route-in-middleware` + ('\\n' + trace))",
        "  }",
        "}",
    ]
    .join("\n");
    let edges = scan_ok("packages/nuxt/src/app/composables/router.ts", &source, 0);

    assert!(edges.is_empty());
}
