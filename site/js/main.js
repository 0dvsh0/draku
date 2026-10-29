(function () {
  'use strict';
  var $ = function (s, c) { return (c || document).querySelector(s); };
  var $$ = function (s, c) { return Array.prototype.slice.call((c || document).querySelectorAll(s)); };

  /* ---------- toast ---------- */
  var toastEl = document.createElement('div');
  toastEl.className = 'toast'; toastEl.setAttribute('role', 'status');
  document.body.appendChild(toastEl);
  var toastT;
  function toast(msg) {
    toastEl.textContent = msg; toastEl.classList.add('is-on');
    clearTimeout(toastT); toastT = setTimeout(function () { toastEl.classList.remove('is-on'); }, 2400);
  }

  /* ---------- nav ---------- */
  var nav = $('.nav'), burger = $('.nav__burger');
  window.addEventListener('scroll', function () { nav.classList.toggle('is-scrolled', scrollY > 40); }, { passive: true });
  burger.addEventListener('click', function () {
    var o = nav.classList.toggle('is-open'); burger.setAttribute('aria-expanded', o);
  });
  $$('.nav__links a').forEach(function (a) { a.addEventListener('click', function () { nav.classList.remove('is-open'); burger.setAttribute('aria-expanded', false); }); });

  /* ---------- OS detect ---------- */
  var ua = navigator.userAgent, os = /Mac/.test(ua) ? 'macOS' : /Win/.test(ua) ? 'Windows' : /Linux/.test(ua) ? 'Linux' : 'your laptop';
  if (/iPhone|iPad|Android/.test(ua)) os = 'your laptop';
  $$('[data-os]').forEach(function (e) { e.textContent = os; });

  /* ---------- eyes: follow cursor, blink ---------- */
  var eyes = $$('[data-eye]');
  eyes.forEach(function (e) { e.classList.add('is-boot'); });
  setTimeout(function () { eyes.forEach(function (e) { e.classList.remove('is-boot'); }); }, 1800);
  function look(x, y) {
    eyes.forEach(function (eye) {
      var r = eye.getBoundingClientRect();
      var cx = r.left + r.width / 2, cy = r.top + r.height / 2;
      var dx = x - cx, dy = y - cy, d = Math.hypot(dx, dy) || 1;
      var k = Math.min(d / 300, 1) * 7;
      $('.eye__iris', eye).style.transform = 'translate(' + (dx / d * k).toFixed(2) + 'px,' + (dy / d * k).toFixed(2) + 'px)';
    });
  }
  window.addEventListener('pointermove', function (e) { look(e.clientX, e.clientY); }, { passive: true });
  function blink() {
    eyes.forEach(function (e) { e.classList.remove('is-blink'); void e.getBoundingClientRect(); e.classList.add('is-blink'); });
    setTimeout(blink, 2600 + Math.random() * 4200);
  }
  setTimeout(blink, 3200);

  /* ---------- reveal ---------- */
  var revealTargets = $$('.section .label, .section .h2, .section .lede, .ghost, .filename, .moment__quote, .anatomy, .flow, .app, .nos li, .player, .trio, .terminal, .engine, .reading, .road, .qa, .final__inner');
  var skipReveal = navigator.webdriver || !('IntersectionObserver' in window);
  if (!skipReveal) revealTargets.forEach(function (el) { el.classList.add('reveal'); });
  var io = new IntersectionObserver(function (en) {
    en.forEach(function (e) { if (e.isIntersecting) { e.target.classList.add('is-in'); io.unobserve(e.target); } });
  }, { threshold: .12, rootMargin: '0px 0px -40px 0px' });
  if (!skipReveal) revealTargets.forEach(function (el) { io.observe(el); });

  /* ---------- filename morph (scramble) ---------- */
  var morph = $('#morph'), fileBox = $('.filename');
  var glyphs = 'abcdefghijklmnopqrstuvwxyz0123456789-_';
  function scramble(el, to, done) {
    var from = el.textContent, len = Math.max(from.length, to.length), frame = 0, total = 38;
    (function step() {
      var out = '', p = frame / total;
      for (var i = 0; i < len; i++) {
        var reveal = i / len < p;
        if (reveal) out += to[i] || '';
        else if (i / len < p + .15) out += glyphs[(Math.random() * glyphs.length) | 0];
        else out += from[i] || '';
      }
      el.textContent = out;
      if (frame++ < total) requestAnimationFrame(step); else { el.textContent = to; done && done(); }
    })();
  }
  var morphState = 0;
  function cycleMorph() {
    var to = morphState ? morph.dataset.from : morph.dataset.to;
    fileBox.classList.toggle('is-known', !morphState);
    scramble(morph, to);
    morphState = 1 - morphState;
  }
  new IntersectionObserver(function (en, ob) {
    if (en[0].isIntersecting) {
      ob.disconnect();
      setTimeout(cycleMorph, 900);
      setInterval(cycleMorph, 5200);
    }
  }, { threshold: .6 }).observe(fileBox);

  /* ---------- four outcomes ---------- */
  var OUT = {
    find: ['IMG-20261011-WA0004.jpg', '2026-10-11 Dr Patel blood test prescription.jpg, found by typing “blood test”'],
    remember: ['A warranty receipt screenshot, one week from expiry', 'A calendar reminder, set with one tap'],
    protect: ['An Aadhaar photo with all 12 digits visible and a GPS tag inside', 'First 8 digits blurred, GPS tag removed'],
    letgo: ['Old OTPs, expired tickets, four copies of one photo', 'Set aside in a rest folder, restorable']
  };
  var ba = $('.ba'), baB = $('#baBefore'), baA = $('#baAfter');
  function setOutcome(k) {
    $$('.outcome').forEach(function (b) { var on = b.dataset.outcome === k; b.classList.toggle('is-active', on); b.setAttribute('aria-selected', on); });
    $$('.spot').forEach(function (b) { b.classList.toggle('is-active', b.dataset.outcome === k); });
    ba.classList.add('is-swap');
    setTimeout(function () { baB.textContent = OUT[k][0]; baA.textContent = OUT[k][1]; ba.classList.remove('is-swap'); }, 220);
  }
  $$('[data-outcome]').forEach(function (b) { b.addEventListener('click', function () { setOutcome(b.dataset.outcome); stopAuto(); }); });
  setOutcome('find');
  var order = ['find', 'remember', 'protect', 'letgo'], oi = 0, autoT = setInterval(function () { oi = (oi + 1) % 4; setOutcome(order[oi]); }, 4200);
  function stopAuto() { clearInterval(autoT); }

  /* ---------- app: Ask anything ---------- */
  var FILES = [
    { k: ['wifi', 'wi-fi', 'password', 'router', 'network'], was: 'Screenshot_20260918-221455.png', name: '2026-09-18 Home Wi-Fi password.png', crop: '<b>Home_5G</b>pass: tiger-lily-0429', meta: 'Screenshot · 6 weeks ago · cropped to the answer' },
    { k: ['blood', 'prescription', 'patel', 'pharmacy', 'medicine', 'doctor'], was: 'IMG-20261011-WA0004.jpg', name: '2026-10-11 Dr Patel blood test prescription.jpg', crop: '<b>Rx · Dr Patel</b>CBC, Lipid panel', meta: 'Photo · WhatsApp · 18 days ago' },
    { k: ['laptop', 'warranty', 'receipt', 'invoice'], was: 'scan_1(2).pdf', name: '2025-11-02 Laptop warranty receipt.pdf', crop: '<b>Warranty valid till</b>02 Nov 2026', meta: 'PDF · ends in 4 days · reminder ready' },
    { k: ['train', 'ticket', 'pnr', 'travel'], was: 'IMG_0042.png', name: '2026-08-14 Mumbai–Pune train ticket.png', crop: '<b>PNR 482-1937</b>Coach C2 · 41', meta: 'Screenshot · trip has passed · ready to let go' },
    { k: ['exam', 'form', 'admit'], was: 'Download (3).pdf', name: '2026-10-30 Exam application form.pdf', crop: '<b>Last date</b>30 Oct 2026', meta: 'PDF · closes tomorrow' }
  ];
  var ask = $('#ask'), results = $('#results');
  function search(q) {
    q = q.trim().toLowerCase();
    if (!q) { results.innerHTML = ''; return; }
    var hit = null, best = 0;
    FILES.forEach(function (f) {
      var s = 0; f.k.forEach(function (k) { if (q.indexOf(k) > -1) s += 1; });
      if (s > best) { best = s; hit = f; }
    });
    if (!hit) { results.innerHTML = '<div class="result result--none">Nothing in the demo folder matches. Try “Wi-Fi password from last month”.</div>'; return; }
    results.innerHTML = '<div class="result"><div class="result__crop">' + hit.crop + '</div><div><div class="result__was">' + hit.was + '</div><div class="result__name">' + hit.name + '</div><div class="result__meta">' + hit.meta + '</div></div></div>';
  }
  var debounce;
  ask.addEventListener('input', function () { clearTimeout(debounce); debounce = setTimeout(function () { search(ask.value); }, 160); });
  $$('.ask__hints button').forEach(function (b) {
    b.addEventListener('click', function () {
      var q = b.dataset.q, i = 0; ask.value = ''; ask.focus();
      (function type() { ask.value = q.slice(0, ++i); if (i < q.length) setTimeout(type, 28); else search(q); })();
    });
  });
  document.addEventListener('keydown', function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); ask.focus(); ask.scrollIntoView({ block: 'center', behavior: 'smooth' }); }
  });

  /* ---------- app: card buttons ---------- */
  var btnCal = $('#btnCal');
  btnCal.addEventListener('click', function () {
    if (btnCal.classList.contains('is-done')) return;
    $$('.card--date li').forEach(function (li) { li.classList.add('is-done'); });
    btnCal.classList.add('is-done'); $('span', btnCal).textContent = 'Added · 2 reminders';
    toast('2 reminders added to your calendar. Locally.');
  });
  var btnSafe = $('#btnSafe'), shot = $('#shot');
  btnSafe.addEventListener('click', function () {
    var on = shot.classList.toggle('is-safe');
    btnSafe.classList.toggle('is-done', on);
    $('span', btnSafe).textContent = on ? 'Safe copy made · undo' : 'Make a safe copy';
    if (on) toast('Password blurred into the pixels. GPS tag stripped.');
  });
  var btnAside = $('#btnAside'), tiles = $('#tiles'), rest = $('#rest'), num = $('#letgoNum');
  btnAside.addEventListener('click', function () {
    tiles.classList.add('is-gone');
    setTimeout(function () { tiles.hidden = true; rest.hidden = false; num.textContent = '0'; btnAside.hidden = true; }, 520);
    toast('9 files resting in a folder you can restore from.');
  });
  $('#btnRestore').addEventListener('click', function () {
    rest.hidden = true; tiles.hidden = false; num.textContent = '9'; btnAside.hidden = false;
    requestAnimationFrame(function () { tiles.classList.remove('is-gone'); });
    toast('Restored. Nothing was ever deleted.');
  });

  /* ---------- demo player ---------- */
  var BEATS = [
    { t: 0, on: 'A messy folder of 25 files: IMG_0042.png, scan_1(2).pdf, old screenshots.', say: '“This folder is on every laptop. Airplane mode is on. Everything you are about to see happens right here.”' },
    { t: 10, on: 'Drop the folder. Cryptic names turn into clear ones as you watch. Type: laptop warranty.', say: '“Draku reads each file once. Now every file has a name, and you can just ask.”' },
    { t: 25, on: 'Laptop warranty ends in 4 days. → Add to calendar.', say: '“That date was trapped inside a picture. Now it is in your calendar.”' },
    { t: 35, on: 'A password in a browser tab and a hidden GPS pin light up. → Make a safe copy.', say: '“Before you share it, you see what you would give away. One tap and it is gone.”' },
    { t: 48, on: 'The Let go card shows 9 files. → Set aside.', say: '“Nine files nothing depends on. They rest in a folder you can restore from.”' },
    { t: 55, on: 'That was airplane mode the whole time.', say: '“One more thing.”', reveal: true }
  ];
  var player = $('#player'), stage = $('.stage'), on = $('#stageOn'), say = $('#stageSay'), clock = $('#clock'), fill = $('#trackFill');
  var beatEls = $$('#beats li'), cur = 0, playing = false, T = 0, last = 0, idx = -1;
  var SPEED = 4; // 60s demo plays in 15s
  function show(i) {
    if (i === idx) return; idx = i;
    beatEls.forEach(function (li, j) { li.classList.toggle('is-active', j === i); });
    stage.classList.add('is-swap');
    setTimeout(function () {
      on.textContent = BEATS[i].on; say.textContent = BEATS[i].say;
      stage.classList.toggle('is-reveal', !!BEATS[i].reveal);
      stage.classList.remove('is-swap');
    }, 240);
  }
  function render() {
    var s = Math.floor(T); clock.textContent = '0:' + (s < 10 ? '0' : '') + s;
    if (T >= 60) clock.textContent = '1:00';
    fill.style.width = (T / 60 * 100) + '%';
    var i = 0; BEATS.forEach(function (b, j) { if (T >= b.t) i = j; }); show(i);
  }
  function tick(now) {
    if (!playing) return;
    T += (now - last) / 1000 * SPEED; last = now;
    if (T >= 60) { T = 60; render(); setPlay(false); return; }
    render(); requestAnimationFrame(tick);
  }
  function setPlay(p) {
    playing = p; player.classList.toggle('is-playing', p);
    $('#play').setAttribute('aria-label', p ? 'Pause demo' : 'Play demo');
    if (p) { if (T >= 60) T = 0; last = performance.now(); requestAnimationFrame(tick); }
  }
  $('#play').addEventListener('click', function () { setPlay(!playing); });
  beatEls.forEach(function (li) { li.addEventListener('click', function () { T = +li.dataset.t; render(); }); });
  $('#track').addEventListener('click', function (e) {
    var r = this.getBoundingClientRect(); T = Math.max(0, Math.min(59.9, (e.clientX - r.left) / r.width * 60)); render();
  });
  show(0);

  /* ---------- copy command ---------- */
  $('#copyCmd').addEventListener('click', function () {
    var txt = 'git clone https://github.com/draku-app/draku && cd draku\n./draku ~/Screenshots ~/Downloads';
    (navigator.clipboard ? navigator.clipboard.writeText(txt) : Promise.reject()).then(function () { toast('Copied. Now go read the code.'); }, function () { toast('Select and copy the commands above.'); });
  });
})();
