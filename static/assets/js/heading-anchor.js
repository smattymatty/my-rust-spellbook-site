// Permalink headings: clicking a heading (or the page title) copies its URL to
// the clipboard with a brief check-mark flash. Section headings link to a
// fragment (#slug) and still jump natively; the page title links to the whole
// page URL, where a jump would just reload - so that one we copy without
// navigating. With JS off, section links still jump; only the copy is lost.

document.addEventListener('click', (e) => {
    const link = e.target.closest('.heading-link');
    if (!link) return;

    // The title's link points at the current page with no fragment - copying is
    // the whole point, so suppress the pointless reload. Section links (which
    // carry a #hash) keep their native jump.
    const samePageNoHash = link.pathname === location.pathname && !link.hash;
    if (samePageNoHash) e.preventDefault();

    if (!navigator.clipboard) return;
    navigator.clipboard.writeText(link.href).then(() => {
        link.classList.add('copied');
        setTimeout(() => link.classList.remove('copied'), 1200);
    }).catch(() => { /* clipboard blocked - any native jump still happened */ });
});
