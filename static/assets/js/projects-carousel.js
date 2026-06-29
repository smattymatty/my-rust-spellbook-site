// Project image carousel: rotates a project's screenshots on /projects/. Same
// shape as the home-page quote rotator - auto-advances, with prev/next and dot
// controls. A single-image carousel is left static (its controls aren't even
// rendered). Pauses while hovered or focused so a reader can actually look.

document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('[data-carousel]').forEach((root) => {
        const slides = Array.from(root.querySelectorAll('.carousel-slide'));
        if (slides.length === 0) return;

        // Mark ready: CSS then drives visibility off .is-active instead of the
        // pre-JS "first slide visible" fallback, so there's no flash.
        root.classList.add('is-ready');

        if (slides.length === 1) {
            slides[0].classList.add('is-active');
            return;
        }

        const dots = Array.from(root.querySelectorAll('.carousel-dot'));
        let index = 0;
        let timer = null;

        const show = (n) => {
            index = (n + slides.length) % slides.length;
            slides.forEach((s, k) => s.classList.toggle('is-active', k === index));
            dots.forEach((d, k) => {
                const on = k === index;
                d.classList.toggle('is-active', on);
                d.setAttribute('aria-current', on ? 'true' : 'false');
            });
        };

        const start = () => { stop(); timer = setInterval(() => show(index + 1), 6000); };
        const stop = () => { if (timer) { clearInterval(timer); timer = null; } };

        root.querySelector('.carousel-prev')?.addEventListener('click', () => { show(index - 1); start(); });
        root.querySelector('.carousel-next')?.addEventListener('click', () => { show(index + 1); start(); });
        dots.forEach((d, k) => d.addEventListener('click', () => { show(k); start(); }));

        root.addEventListener('mouseenter', stop);
        root.addEventListener('mouseleave', start);
        root.addEventListener('focusin', stop);
        root.addEventListener('focusout', start);

        show(0);
        start();
    });
});
