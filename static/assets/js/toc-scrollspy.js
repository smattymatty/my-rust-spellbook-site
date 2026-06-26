// Scroll-spy for the on-this-page TOC: highlights the link for the section
// currently in view. Pure enhancement — without it the links are still plain
// jump anchors. The "active band" is the top quarter of the viewport; the
// topmost heading inside it wins, and when none is inside (mid-section or at
// the very bottom) the last active link stays lit.
//
// Clicking a link locks the highlight to that target and ignores the observer
// until the smooth-scroll settles, so the active marker jumps straight to the
// destination instead of flickering through every section it passes.

document.addEventListener('DOMContentLoaded', () => {
    const links = Array.from(document.querySelectorAll('.toc-link'));
    if (links.length === 0) return;

    const idOf = (link) => decodeURIComponent(link.getAttribute('href').slice(1));
    const linkFor = new Map();
    const headings = [];
    links.forEach((link) => {
        const el = document.getElementById(idOf(link));
        if (el) { linkFor.set(idOf(link), link); headings.push(el); }
    });
    if (headings.length === 0) return;

    let current = null;
    const setActive = (id) => {
        if (id === current) return;
        if (current) linkFor.get(current)?.classList.remove('active');
        current = id;
        if (id) linkFor.get(id)?.classList.add('active');
    };

    // While a click-scroll is in flight, lockId pins the highlight and the
    // observer stands down. The lock releases once scrolling has settled
    // (no scroll event for a short beat).
    let lockId = null;
    let settleTimer = null;
    function armSettle() {
        clearTimeout(settleTimer);
        settleTimer = setTimeout(() => {
            lockId = null;
            window.removeEventListener('scroll', armSettle);
        }, 150);
    }

    const visible = new Set();
    const observer = new IntersectionObserver((entries) => {
        for (const e of entries) {
            if (e.isIntersecting) visible.add(e.target.id);
            else visible.delete(e.target.id);
        }
        if (lockId !== null) return;  // pinned to a clicked target
        const first = headings.find((h) => visible.has(h.id));
        if (first) setActive(first.id);
    }, { rootMargin: '0px 0px -75% 0px', threshold: 0 });

    headings.forEach((h) => observer.observe(h));

    // Clicking a link lights it immediately and holds it through the scroll.
    links.forEach((link) => link.addEventListener('click', () => {
        setActive(idOf(link));
        lockId = idOf(link);
        window.removeEventListener('scroll', armSettle);
        window.addEventListener('scroll', armSettle, { passive: true });
        armSettle();  // also covers the no-scroll case (already at target)
    }));
});
