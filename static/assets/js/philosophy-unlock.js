// The footer "Show philosophy writing" switch. Philosophy ships in every build
// - in the page HTML, the sitemap, and the JSON-LD - so crawlers always index
// it. For a human visitor it stays hidden: CSS gates .post-card--philosophy and
// the .philosophy-gated nav link behind the .philosophy-unlocked class on <html>.
// An inline script in <head> restores that class before first paint (no flash);
// this file only wires the control and keeps it in sync with localStorage.

document.addEventListener('DOMContentLoaded', () => {
    const input = document.getElementById('philosophy-unlock');
    if (!input) return;

    const KEY = 'philosophy-unlocked';
    const root = document.documentElement;

    // Reflect the state the head script already restored from storage.
    input.checked = root.classList.contains('philosophy-unlocked');

    input.addEventListener('change', () => {
        const on = input.checked;
        root.classList.toggle('philosophy-unlocked', on);
        try {
            if (on) localStorage.setItem(KEY, '1');
            else localStorage.removeItem(KEY);
        } catch (e) { /* storage blocked: the toggle still works for this view */ }
    });
});
