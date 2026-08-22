import puppeteer from "puppeteer-core";
const B='http://127.0.0.1:8080';
const br = await puppeteer.launch({executablePath:'/usr/bin/google-chrome',args:['--no-sandbox','--disable-gpu']});
const P = async (r) => { const p = await br.newPage(); await p.authenticate({username:'op',password:'pw'});
  await p.goto(B+r,{waitUntil:'networkidle0'}); return p; };
let fail = 0;
const check = (ok, label, detail) => { console.log(`${ok?'✅':'🔴'} ${label}${detail?'  '+detail:''}`); if(!ok) fail++; };

// combien de lignes la file a-t-elle ?
let p = await P('/triage');
const n = await p.$$eval('.queue .queue-row > a', a=>a.length);
console.log(`file : ${n} ligne(s)\n`);

if (n >= 2) {
  // A. la flèche déplace le focus IMMÉDIATEMENT
  await p.evaluate(()=>document.body.focus());
  await p.keyboard.press('ArrowDown');
  const idx = await p.evaluate(()=>{const a=[...document.querySelectorAll('.queue .queue-row > a')];return a.indexOf(document.activeElement);});
  // Depuis la ligne DÉJÀ sélectionnée (aria-current), ↓ va à la suivante : index 1 sur une
  // file de deux. Une première version de cette sonde attendait 0 — l'assertion était fausse,
  // pas le code.
  check(idx===1, 'flèche bas : le focus part de la ligne sélectionnée et avance', `index=${idx}`);
  await p.keyboard.press('ArrowDown');
  const idx2 = await p.evaluate(()=>{const a=[...document.querySelectorAll('.queue .queue-row > a')];return a.indexOf(document.activeElement);});
  // Relatif, jamais absolu : une première version épinglait `1`, écrite quand la file avait
  // deux lignes, et elle a rougi dès qu'elle en a eu cinq. Ce qui compte est le PAS.
  check(idx2===idx+1, 'flèche bas : le focus avance d’une ligne', `${idx} → ${idx2}`);
  const hl = await p.evaluate(()=>{const a=[...document.querySelectorAll('.queue .queue-row')];return a.findIndex(r=>r.classList.contains('selected'));});
  check(hl===idx2, 'le surlignage suit le focix immédiatement', `surligné=${hl}`);
  // B. l'URL rattrape après le silence
  const before = p.url();
  await new Promise(r=>setTimeout(r,600));
  const after = p.url();
  check(after!==before, "l'URL rattrape après 250 ms de silence", after.replace(B,''));
  // C. le focus est VISIBLE
  await p.goto(B+'/triage',{waitUntil:'networkidle0'});
  const ring = await p.evaluate(()=>{const a=document.querySelector('.queue .queue-row > a');a.focus();
    const s=getComputedStyle(a);return s.outlineWidth+' '+s.outlineStyle+' '+s.outlineColor;});
  check(!ring.startsWith('1px') && ring.includes('solid'), 'le focus est visible par une règle du produit', ring);
}
// D. INERTE là où il n'y a pas de file — mesuré sur MON code, pas sur le défilement du
//    navigateur. Contrôle : la même page SANS app.js ne défile pas davantage sous CDP, donc
//    scrollY ne mesurerait rien ici. La propriété qui compte est que la couche n'INTERCEPTE
//    pas la touche.
for (const r of ['/diagnostic','/apps','/sources']) {
  const q = await P(r);
  const res = await q.evaluate(()=>{
    const ev=new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true});
    document.body.dispatchEvent(ev);
    return {prevented: ev.defaultPrevented, rows: document.querySelectorAll('.queue .queue-row > a').length};
  });
  check(res.prevented===false && res.rows===0, `${r} : la couche laisse la flèche au navigateur`,
        `interceptée=${res.prevented} lignes=${res.rows}`);
  await q.close();
}
// E. la flèche ne fait RIEN quand le focus est dans la navigation
const q = await P('/triage');
const url0 = q.url();
await q.evaluate(()=>document.querySelector('nav.nav a.nav-entry').focus());
await q.keyboard.press('ArrowDown');
await new Promise(r=>setTimeout(r,600));
check(q.url()===url0, "flèche avec le focus dans la NAVIGATION : l'URL ne bouge pas", q.url().replace(B,''));
// F. aucune LETTRE n'est liée — à un index du MILIEU
if (n>=3) {
  const mid = Math.floor(n/2);
  const res = await q.evaluate((mid)=>{
    const a=[...document.querySelectorAll('.queue .queue-row > a')]; a[mid].focus();
    const out={};
    for (const k of ['a','j','k','x','Enter','Backspace','ArrowUp','ArrowDown','Home','PageDown',' ']) {
      const ev=new KeyboardEvent('keydown',{key:k,bubbles:true,cancelable:true});
      a[mid].dispatchEvent(ev); out[k]=ev.defaultPrevented;
    } return out; }, mid);
  const letters = ['a','j','k','x','Enter','Backspace','Home','PageDown',' '];
  check(letters.every(k=>res[k]===false), 'aucune lettre ni ⏎ ni ⌫ n’est liée', JSON.stringify(res));
  check(res['ArrowUp']===true && res['ArrowDown']===true, 'les deux flèches SONT liées (contrôle positif, index du milieu)', `↑=${res['ArrowUp']} ↓=${res['ArrowDown']}`);
}
await br.close();
console.log(fail===0 ? '\nTOUT VERT' : `\n${fail} ÉCHEC(S)`);
process.exit(fail===0?0:1);
