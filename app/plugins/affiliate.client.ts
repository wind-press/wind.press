export default defineNuxtPlugin(() => {
    const hostname = 'rosua.org';
    const affiliateCookieKey = 'affiliate-wp-ref';
    let affiliateId: string | null = null;

    if (typeof window !== 'undefined') {
        const currentUrl = new URL(window.location.href);

        // Check for the affiliate id
        if (currentUrl.searchParams.has('ref')) {
            affiliateId = currentUrl.searchParams.get('ref');
            document.cookie = `${affiliateCookieKey}=${affiliateId}; max-age=2592000; path=/`;
        } else if (document.cookie.includes(affiliateCookieKey)) {
            affiliateId = document.cookie
                .split('; ')
                .find(row => row.startsWith(affiliateCookieKey))
                ?.split('=')[1] || null;
        }

        if (affiliateId) {
            const transferAff = (doc: Document) => {
                const links = doc.querySelectorAll('a');
                links.forEach(link => {
                    try {
                        const url = new URL(link.href);
                        if (url.hostname === hostname && !url.searchParams.has('ref')) {
                            url.searchParams.set('ref', affiliateId!);
                            link.href = url.href;
                        }
                    } catch {
                        // Ignore invalid URLs
                    }
                });
            };

            // Observe DOM changes
            const observer = new MutationObserver(mutationsList => {
                for (let mutation of mutationsList) {
                    if (mutation.type === 'childList') {
                        transferAff(document);
                    }
                }
            });
            observer.observe(document.body, { childList: true, subtree: true });

            // Initial run
            transferAff(document);

            // // Remove the affiliate id from the current url
            nextTick(() => {
                // const router = useRouter();
                // const route = useRoute();
                // if (route.query.ref) {
                //     const newQuery = { ...route.query };
                //     delete newQuery.ref;
                //     router.replace({ query: newQuery });
                // }
                currentUrl.searchParams.delete('ref');
                window.history.replaceState({}, document.title, currentUrl.href);
            });
        }
    }
});