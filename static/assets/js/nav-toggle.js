// Mobile nav: the terminal [ ≡ MENU ] button toggles the collapsed nav panel.
// Desktop never reaches this state — the button is display:none above 820px, so
// the listeners are harmless no-ops there. State lives in aria-expanded; CSS
// reads it to swap the glyph/label and the .nav-open class drives the panel.

document.addEventListener('DOMContentLoaded', () => {
    const header = document.querySelector('.site-header');
    const btn = header && header.querySelector('.nav-toggle');
    const nav = document.getElementById('site-nav');
    if (!header || !btn || !nav) return;

    const setOpen = (open) => {
        header.classList.toggle('nav-open', open);
        btn.setAttribute('aria-expanded', String(open));
        btn.setAttribute('aria-label', open ? 'Close menu' : 'Open menu');
    };

    const isOpen = () => btn.getAttribute('aria-expanded') === 'true';

    btn.addEventListener('click', () => setOpen(!isOpen()));

    // Tapping a destination closes the panel before the navigation lands.
    nav.addEventListener('click', (e) => {
        if (e.target.closest('a')) setOpen(false);
    });

    // Escape closes and returns focus to the toggle.
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && isOpen()) {
            setOpen(false);
            btn.focus();
        }
    });

    // A tap outside the header dismisses an open panel.
    document.addEventListener('click', (e) => {
        if (isOpen() && !header.contains(e.target)) setOpen(false);
    });
});
