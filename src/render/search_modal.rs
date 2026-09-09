//! VPLocalSearchBox modal shell (ported verbatim from
//! partials/search_modal.html). Result markup is built client-side and
//! styled through the `[&_...]` variants on the results list. The search
//! index URL is handed to the client via `window.genDocs` (stage 6 moves
//! the whole modal to Alpine).

use hypertext::prelude::*;

use super::icons::icon;

pub fn search_modal(index_url: &str) -> impl Renderable + '_ {
    let kbd_cls = "bg-[rgba(128,128,128,0.1)] rounded-[0.25rem] px-[0.375rem] py-[0.1875rem] min-w-6 inline-block text-center align-middle border border-[rgba(128,128,128,0.15)] shadow-[0_2px_2px_0_rgba(0,0,0,0.1)] font-[inherit]";
    let results_cls = concat!(
        "flex flex-col gap-[0.375rem] overflow-x-hidden overflow-y-auto overscroll-contain",
        " [&_.result]:flex [&_.result]:items-center [&_.result]:gap-2 [&_.result]:rounded-[0.25rem] [&_.result]:leading-none [&_.result]:border-2 [&_.result]:border-solid",
        " [&_.result]:border-(--vp-local-search-result-border) [&_.result]:outline-none [&_.result>div]:m-3 [&_.result>div]:w-full [&_.result>div]:overflow-hidden max-md:[&_.result>div]:m-2",
        " [&_.titles]:flex [&_.titles]:flex-wrap [&_.titles]:gap-1 [&_.titles]:relative [&_.titles]:z-[1001] [&_.titles]:py-[0.125rem]",
        " [&_.title]:flex [&_.title]:items-center [&_.title]:gap-1 [&_.title]:text-[0.875rem] [&_.title]:leading-[1.6] [&_.title.main]:font-medium",
        " [&_.title]:after:content-['/'] [&_.title]:after:opacity-50 [&_.title]:after:font-medium [&_.title:last-child]:after:content-none",
        " [&_.excerpt]:opacity-50 [&_.excerpt]:pointer-events-none [&_.excerpt]:max-h-[8.75rem] [&_.excerpt]:overflow-hidden [&_.excerpt]:relative [&_.excerpt]:mt-1",
        " [&_.result.selected]:border-(--vp-local-search-result-selected-border) [&_.result.selected_.excerpt]:opacity-100",
        " [&_.no-results]:py-4 [&_.no-results]:px-3 [&_.no-results]:text-[0.875rem] [&_.no-results]:text-text-2",
        " [&_mark]:bg-(--vp-local-search-highlight-bg) [&_mark]:text-(--vp-local-search-highlight-text) [&_mark]:rounded-[0.125rem] [&_mark]:px-[0.125rem]",
    );
    rsx! {
        <div class="fixed inset-0 z-[100] flex" id="VPLocalSearchBox" x-cloak x-show="$store.ui.search" x-data="searchModal" data-index-url=(index_url) role="dialog" aria-modal="true">
            <div class="absolute inset-0 bg-(--vp-backdrop-bg-color) transition-opacity duration-500" id="VPSearchBackdrop" @click="close()"></div>
            <div class="relative p-3 my-16 mx-auto flex flex-col gap-4 bg-(--vp-local-search-bg) w-[min(100vw-3.75rem,56.25rem)] h-min max-h-[min(100vh-8rem,56.25rem)] rounded-md max-md:my-0 max-md:w-screen max-md:h-screen max-md:max-h-none max-md:rounded-none">
                <form class="border border-divider rounded-[0.25rem] flex items-center px-3 cursor-text focus-within:border-brand-1 max-md:px-2" id="VPSearchBar" onsubmit="return false">
                    <label id="localsearch-label" for="localsearch-input" title="Search">
                        (icon("search", "block m-2 text-[1.125rem] max-md:hidden"))
                    </label>
                    <input
                        class="py-[0.375rem] px-3 w-full placeholder:text-text-3 [&::-webkit-search-cancel-button]:hidden"
                        id="localsearch-input"
                        x-ref="input"
                        x-model="q"
                        @input.debounce.200ms="search()"
                        @keydown.arrow-down.prevent="move(1)"
                        @keydown.arrow-up.prevent="move(-1)"
                        @keydown.enter.prevent="enter()"
                        aria-labelledby="localsearch-label"
                        autocomplete="off"
                        autocapitalize="off"
                        "autocorrect"="off"
                        spellcheck="false"
                        maxlength="64"
                        type="search"
                        placeholder="Search docs"
                    >
                    <div class="flex gap-1">
                        <button type="button" class="p-2 not-disabled:hover:text-brand-1 cursor-pointer" id="VPSearchClear" title="Clear" @click="clear()" :disabled=("q.trim() === ''")>
                            (icon("delete", ""))
                        </button>
                    </div>
                </form>

                <ul class=(results_cls) id="VPSearchResults" x-ref="results" x-html=(r#"resultsHtml"#) @click="pick($event)" :class=("(results.length) ? '' : 'flex-1'") role="listbox" aria-labelledby="localsearch-label"></ul>

                <div class="text-[0.8rem] opacity-75 flex flex-wrap gap-4 leading-[1.09375] max-md:hidden" id="VPSearchShortcuts">
                    <span class="flex items-center gap-1"><kbd class=(kbd_cls)>"←"</kbd><kbd class=(kbd_cls)>"→"</kbd>" to navigate"</span>
                    <span class="flex items-center gap-1"><kbd class=(kbd_cls)>"Enter"</kbd>" to select"</span>
                    <span class="flex items-center gap-1"><kbd class=(kbd_cls)>"Esc"</kbd>" to close"</span>
                </div>
            </div>
        </div>
    }
}
