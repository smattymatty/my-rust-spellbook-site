// Wrap every <pre><code> on the page with a positioned container and inject a
// COPY button. On click, write the code's textContent to the clipboard and
// flash a confirmation label.

document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('pre').forEach((pre) => {
        const code = pre.querySelector('code');
        if (!code) return;
        if (pre.parentElement && pre.parentElement.classList.contains('code-block')) return;

        const wrapper = document.createElement('div');
        wrapper.className = 'code-block';
        pre.parentNode.insertBefore(wrapper, pre);
        wrapper.appendChild(pre);

        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'copy-btn';
        btn.setAttribute('aria-label', 'copy code to clipboard');
        btn.textContent = 'COPY';
        wrapper.appendChild(btn);

        let resetTimer = null;

        btn.addEventListener('click', async () => {
            if (resetTimer) clearTimeout(resetTimer);
            try {
                await navigator.clipboard.writeText(code.textContent);
                btn.textContent = 'COPIED';
                btn.classList.remove('copy-btn--err');
                btn.classList.add('copy-btn--ok');
            } catch (err) {
                btn.textContent = 'FAILED';
                btn.classList.remove('copy-btn--ok');
                btn.classList.add('copy-btn--err');
            }
            resetTimer = setTimeout(() => {
                btn.textContent = 'COPY';
                btn.classList.remove('copy-btn--ok', 'copy-btn--err');
            }, 1500);
        });
    });
});
