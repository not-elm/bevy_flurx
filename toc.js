// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="overview.html"><strong aria-hidden="true">1.</strong> Overview</a></li><li class="chapter-item expanded "><a href="actions/index.html"><strong aria-hidden="true">2.</strong> Actions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="actions/once/index.html"><strong aria-hidden="true">2.1.</strong> once</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="actions/once/run.html"><strong aria-hidden="true">2.1.1.</strong> run</a></li><li class="chapter-item expanded "><a href="actions/once/event.html"><strong aria-hidden="true">2.1.2.</strong> event</a></li><li class="chapter-item expanded "><a href="actions/once/res.html"><strong aria-hidden="true">2.1.3.</strong> res</a></li><li class="chapter-item expanded "><a href="actions/once/non_send.html"><strong aria-hidden="true">2.1.4.</strong> non_send</a></li><li class="chapter-item expanded "><a href="actions/once/switch.html"><strong aria-hidden="true">2.1.5.</strong> switch</a></li><li class="chapter-item expanded "><a href="actions/once/state.html"><strong aria-hidden="true">2.1.6.</strong> state</a></li><li class="chapter-item expanded "><a href="actions/once/audio.html"><strong aria-hidden="true">2.1.7.</strong> audio</a></li></ol></li><li class="chapter-item expanded "><a href="actions/wait/index.html"><strong aria-hidden="true">2.2.</strong> wait</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="actions/wait/until.html"><strong aria-hidden="true">2.2.1.</strong> until</a></li><li class="chapter-item expanded "><a href="actions/wait/output.html"><strong aria-hidden="true">2.2.2.</strong> output</a></li><li class="chapter-item expanded "><a href="actions/wait/event.html"><strong aria-hidden="true">2.2.3.</strong> event</a></li><li class="chapter-item expanded "><a href="actions/wait/switch.html"><strong aria-hidden="true">2.2.4.</strong> switch</a></li><li class="chapter-item expanded "><a href="actions/wait/state.html"><strong aria-hidden="true">2.2.5.</strong> state</a></li><li class="chapter-item expanded "><a href="actions/wait/audio.html"><strong aria-hidden="true">2.2.6.</strong> audio</a></li><li class="chapter-item expanded "><a href="actions/wait/input.html"><strong aria-hidden="true">2.2.7.</strong> input</a></li><li class="chapter-item expanded "><a href="actions/wait/all.html"><strong aria-hidden="true">2.2.8.</strong> all</a></li><li class="chapter-item expanded "><a href="actions/wait/any.html"><strong aria-hidden="true">2.2.9.</strong> any</a></li><li class="chapter-item expanded "><a href="actions/wait/both.html"><strong aria-hidden="true">2.2.10.</strong> both</a></li><li class="chapter-item expanded "><a href="actions/wait/either.html"><strong aria-hidden="true">2.2.11.</strong> either</a></li></ol></li><li class="chapter-item expanded "><a href="actions/delay.html"><strong aria-hidden="true">2.3.</strong> delay</a></li><li class="chapter-item expanded "><a href="actions/sequence.html"><strong aria-hidden="true">2.4.</strong> sequence</a></li><li class="chapter-item expanded "><a href="actions/pipe.html"><strong aria-hidden="true">2.5.</strong> pipe</a></li><li class="chapter-item expanded "><a href="actions/through.html"><strong aria-hidden="true">2.6.</strong> through</a></li><li class="chapter-item expanded "><a href="actions/omit.html"><strong aria-hidden="true">2.7.</strong> omit</a></li><li class="chapter-item expanded "><a href="actions/map.html"><strong aria-hidden="true">2.8.</strong> map</a></li><li class="chapter-item expanded "><a href="actions/inspect.html"><strong aria-hidden="true">2.9.</strong> inspect</a></li><li class="chapter-item expanded "><a href="actions/remake.html"><strong aria-hidden="true">2.10.</strong> remake</a></li><li class="chapter-item expanded "><a href="actions/switch.html"><strong aria-hidden="true">2.11.</strong> switch</a></li><li class="chapter-item expanded "><a href="actions/record/index.html"><strong aria-hidden="true">2.12.</strong> record</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="actions/record/push.html"><strong aria-hidden="true">2.12.1.</strong> push</a></li><li class="chapter-item expanded "><a href="actions/record/undo.html"><strong aria-hidden="true">2.12.2.</strong> undo</a></li><li class="chapter-item expanded "><a href="actions/record/redo.html"><strong aria-hidden="true">2.12.3.</strong> redo</a></li><li class="chapter-item expanded "><a href="actions/record/all_clear.html"><strong aria-hidden="true">2.12.4.</strong> all_clear</a></li><li class="chapter-item expanded "><a href="actions/record/extension.html"><strong aria-hidden="true">2.12.5.</strong> extension</a></li></ol></li><li class="chapter-item expanded "><a href="actions/side_effect/index.html"><strong aria-hidden="true">2.13.</strong> side_effect</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="actions/side_effect/bevy_task.html"><strong aria-hidden="true">2.13.1.</strong> bevy_task</a></li><li class="chapter-item expanded "><a href="actions/side_effect/thread.html"><strong aria-hidden="true">2.13.2.</strong> thread</a></li><li class="chapter-item expanded "><a href="actions/side_effect/tokio.html"><strong aria-hidden="true">2.13.3.</strong> tokio</a></li></ol></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
