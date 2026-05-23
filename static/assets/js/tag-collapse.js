// On article pages with many tags in the FILED UNDER strip, hide everything
// past the first 3 and inject a "+N more" toggle that expands/collapses the
// rest. Scoped to .meta-tags so it never fires on the index per-post tag rows.

document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('.meta-tags .tags').forEach((ul) => {
        const items = Array.from(ul.querySelectorAll('li'));
        if (items.length <= 3) return;

        const hidden = items.slice(3);
        hidden.forEach((li) => li.classList.add('tag-hidden'));

        const toggle = document.createElement('li');
        toggle.className = 'tag-toggle';
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'tag-toggle-btn';
        btn.setAttribute('aria-expanded', 'false');
        btn.textContent = `+${hidden.length} more`;
        toggle.appendChild(btn);
        ul.appendChild(toggle);

        btn.addEventListener('click', () => {
            const expanded = btn.getAttribute('aria-expanded') === 'true';
            if (expanded) {
                hidden.forEach((li) => li.classList.add('tag-hidden'));
                btn.setAttribute('aria-expanded', 'false');
                btn.textContent = `+${hidden.length} more`;
            } else {
                hidden.forEach((li) => li.classList.remove('tag-hidden'));
                btn.setAttribute('aria-expanded', 'true');
                btn.textContent = 'show less';
            }
        });
    });
});
