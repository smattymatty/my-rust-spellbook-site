(function () {
    const region = document.querySelector('.quote-region');
    const panel  = document.querySelector('.quote-panel');
    if (!panel || !region) return;

    const dataEl   = panel.querySelector('.quote-data');
    const textEl   = panel.querySelector('.quote-text');
    const attrEl   = panel.querySelector('.quote-attr');
    const dateEl   = panel.querySelector('.quote-date');
    const nextBtn  = region.querySelector('.quote-next');
    const prevBtn  = region.querySelector('.quote-prev');
    const body     = panel.querySelector('.quote-body');

    let quotes = [];
    try { quotes = JSON.parse(dataEl.textContent); } catch (e) { quotes = []; }

    if (quotes.length === 0) { panel.remove(); return; }

    let i = 0;
    let history = [];   // stack of previously-shown indices (most recent on top)
    let timer = null;

    function show(idx) {
        const q = quotes[idx];
        body.classList.add('quote-fading');
        setTimeout(() => {
            textEl.textContent = '“' + q.text + '”';
            attrEl.textContent = 'From "' + q.post_title + '"';
            attrEl.href = q.post_url;
            if (dateEl) dateEl.textContent = q.post_date || '';
            body.classList.remove('quote-fading');
        }, 200);
    }

    function advance() {
        history.push(i);
        i = (i + 1) % quotes.length;
        show(i);
    }

    function goBack() {
        if (history.length > 0) {
            i = history.pop();
        } else {
            // Smart random: pick any quote that isn't the one currently displayed.
            // With only one quote this is a no-op; with two+ it always changes.
            if (quotes.length < 2) return;
            let r;
            do { r = Math.floor(Math.random() * quotes.length); } while (r === i);
            i = r;
        }
        show(i);
    }

    function startTimer() {
        if (quotes.length < 2) return;
        if (timer) clearInterval(timer);
        timer = setInterval(advance, 10000);
    }

    if (quotes.length < 2) {
        if (nextBtn) nextBtn.style.visibility = 'hidden';
        if (prevBtn) prevBtn.style.visibility = 'hidden';
    }

    if (nextBtn) {
        nextBtn.addEventListener('click', () => { advance(); startTimer(); });
    }
    if (prevBtn) {
        prevBtn.addEventListener('click', () => { goBack(); startTimer(); });
    }

    show(0);
    startTimer();
})();
