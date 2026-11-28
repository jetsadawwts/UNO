//! Page styles and scripts, inlined into every page (no external files).

// ---------------------------------------------------------------- page

pub(crate) const CSS: &str = r#"
:root{--bg:#fbfaf8;--fg:#1f1e1c;--muted:#6b6862;--line:#e4e1da;--card:#fff;--c1:#2f6fbf;--c2:#2e8b6e;--c3:#c2593a;--c4:#8a5cc2;--c5:#b8860b;--c6:#3d8fa6}
:root[data-theme=dark]{--bg:#1b1a19;--fg:#ecebe8;--muted:#a19e97;--line:#34322f;--card:#242321;--c1:#6fa3e6;--c2:#5fc0a0;--c3:#e58a6c;--c4:#b393e0;--c5:#e0b44a;--c6:#6cc0d6;color-scheme:dark}
[hidden]{display:none!important}#liveflag .ld{width:8px;height:8px;border-radius:50%;background:var(--c2);box-shadow:0 0 0 3px color-mix(in srgb,var(--c2) 25%,transparent)}#liveflag.off .ld{background:var(--c3);box-shadow:none}#liveflag .l-off{display:none}#liveflag.off .l-on{display:none}#liveflag.off .l-off{display:inline}#liveflag.flash{border-color:var(--c2)}*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:15px/1.6 -apple-system,"Sukhumvit Set","Thonburi",system-ui,sans-serif}
header{position:sticky;top:0;background:var(--bg);border-bottom:1px solid var(--line);padding:12px 28px;display:flex;gap:18px;align-items:center;z-index:2;flex-wrap:wrap}
header h1{font-size:18px;margin:0;display:flex;align-items:center;gap:10px;letter-spacing:.02em}header .meta{color:var(--muted);font-size:13px}
.hright{margin-left:auto;display:flex;gap:10px;align-items:center}
nav button{background:none;border:0;border-bottom:2px solid transparent;color:var(--muted);font:inherit;padding:6px 2px;margin-right:14px;cursor:pointer}
nav button.on{color:var(--fg);border-color:var(--c1);font-weight:600}
main{max-width:1120px;margin:0 auto;padding:24px 28px 60px}.view{display:none}.view.on{display:block}
.ic{width:17px;height:17px;flex:none}
nav button,.pbar .back,#theme>span,.iconbtn,.pill-btn{display:inline-flex;align-items:center;gap:7px}
.pill-btn,#theme{background:var(--card);border:1px solid var(--line);border-radius:99px;padding:4px 12px;font:inherit;font-size:13px;color:var(--fg);cursor:pointer}
#theme>span{display:none}html[data-mode=auto] #theme .m-auto,html[data-mode=light] #theme .m-light,html[data-mode=dark] #theme .m-dark,html:not([data-mode]) #theme .m-auto{display:inline-flex}
#lang{display:inline-flex;border:1px solid var(--line);border-radius:99px;overflow:hidden}
#lang button{background:var(--card);border:0;padding:4px 11px;font:inherit;font-size:13px;color:var(--muted);cursor:pointer}
html[data-lang=en] #lang [data-l=en],html[data-lang=th] #lang [data-l=th]{background:var(--c1);color:#fff;font-weight:600}
html[data-lang=en] .l-th,html[data-lang=th] .l-en,html:not([data-lang]) .l-th{display:none}
.tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(160px,1fr));gap:12px;margin-bottom:8px}
.tile{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px}
.tile .top{display:flex;justify-content:space-between;align-items:flex-start}.tile .num{font-size:30px;font-weight:650;line-height:1.1}
.badge{display:inline-flex;width:34px;height:34px;border-radius:9px;align-items:center;justify-content:center;color:var(--c1);background:color-mix(in srgb,var(--c1) 12%,var(--card))}.badge .ic{width:19px;height:19px}
.tile .lbl{font-weight:600}.tile .note{color:var(--muted);font-size:12.5px}
figcaption,h2{display:flex;align-items:center;gap:8px;flex-wrap:wrap}figcaption .ic,h2 .ic{color:var(--c1)}
h2{font-size:17px;margin:30px 0 10px}table{width:100%;border-collapse:collapse;background:var(--card);border:1px solid var(--line);border-radius:10px;overflow:hidden}
th,td{text-align:left;padding:8px 12px;border-bottom:1px solid var(--line);vertical-align:top;font-size:14px}th{color:var(--muted);font-weight:600;font-size:13px}
tr:last-child td{border-bottom:0}td:first-child{white-space:nowrap}
.charts{display:grid;grid-template-columns:repeat(auto-fit,minmax(440px,1fr));gap:14px;margin-top:10px}
.chart{margin:0;background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px;min-width:0}.chart.wide{grid-column:1/-1}
figcaption{font-weight:600;margin-bottom:8px}svg{width:100%;height:auto}svg .lbl{font-size:12px;fill:var(--muted)}svg .val{font-size:12px;fill:var(--fg)}
svg .axis{stroke:var(--line)}svg text.l-en,svg text.l-th{}html[data-lang=en] svg text.l-th,html[data-lang=th] svg text.l-en{display:none}
.donutwrap{display:flex;gap:18px;align-items:center}.donut{width:180px;flex:none}
.legend{display:flex;flex-wrap:wrap;gap:6px 16px;font-size:13px;color:var(--muted);margin-top:8px}.donutwrap .legend{flex-direction:column;margin:0}
.legend i{display:inline-block;width:10px;height:10px;border-radius:3px;margin-right:6px;vertical-align:-1px}
table.heat td,table.heat th{text-align:center}table.heat td:first-child,table.heat th:first-child{text-align:left}
.pill{background:var(--line);border-radius:99px;padding:1px 9px;font-size:12.5px;white-space:nowrap}
.s-done,.s-merged{background:color-mix(in srgb,var(--c2) 18%,var(--card))}.s-in-progress,.s-in-review,.s-ready{background:color-mix(in srgb,var(--c1) 18%,var(--card))}.s-blocked{background:color-mix(in srgb,var(--c3) 20%,var(--card))}
.muted{color:var(--muted)}.small{font-size:12.5px;margin:6px 0 0}.hint{font-size:13px;font-weight:400}
.pbar{display:flex;gap:10px;align-items:center;margin-bottom:22px}.pbar select,.pbar .back{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:6px 12px;font:inherit;color:var(--fg)}.pbar select{min-width:300px}.pbar .back{cursor:pointer;display:none}.pbar.inp .back{display:inline-flex}
tr.go{cursor:pointer}tr.go:hover td{background:color-mix(in srgb,var(--c1) 7%,var(--card))}tr.go a{color:var(--c1);text-decoration:none}
.pview{display:none}.pview.on{display:block}.ptitle{font-size:20px;margin-top:0}details{margin-top:22px}summary{cursor:pointer;font-weight:600;margin-bottom:10px}
.live{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px}
.livecard{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:12px 14px}
.lc-top{display:flex;align-items:center;gap:8px}.lc-top b{font-weight:600}.lc-last{margin-top:6px;font-size:14px}.lc-meta{font-size:12.5px;color:var(--muted)}
.dot{width:10px;height:10px;border-radius:50%;background:var(--muted);flex:none}
.st{display:none;margin-left:auto;font-size:12px;border-radius:99px;padding:1px 9px;background:var(--line);white-space:nowrap}
.livecard.working .dot{background:var(--c2);box-shadow:0 0 0 4px color-mix(in srgb,var(--c2) 25%,transparent)}.livecard.working .st-working{display:inline-block;background:color-mix(in srgb,var(--c2) 20%,var(--card))}
.livecard.open .dot{background:var(--c5)}.livecard.open .st-open{display:inline-block;background:color-mix(in srgb,var(--c5) 20%,var(--card))}
.livecard.idle .st-idle{display:inline-block}
.lc-line{margin-top:6px;font-size:14px}.lc-line .hv{color:var(--muted);font-size:12px}.lc-line .hv.diff{color:var(--fg);background:color-mix(in srgb,#f59e0b 28%,transparent);border-radius:4px;padding:0 4px;margin-left:4px}header .ver{color:var(--muted);font-size:13px;font-weight:400;white-space:nowrap}.lc-line.stale{display:none}.livecard.has-lines .lc-last{display:none}
.feed{list-style:none;margin:0;padding:0;background:var(--card);border:1px solid var(--line);border-radius:10px}
.feed li{display:flex;gap:12px;padding:7px 14px;border-bottom:1px solid var(--line);font-size:14px}.feed li:last-child{border-bottom:0}
.feed .ago{color:var(--muted);width:110px;flex:none}.feed .fp{color:var(--c1);width:170px;flex:none;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.iconbtn{background:none;border:0;color:var(--fg);cursor:pointer;padding:4px;border-radius:8px}
dialog#settings{border:1px solid var(--line);border-radius:14px;background:var(--bg);color:var(--fg);padding:0;width:min(520px,92vw)}
dialog#settings::backdrop{background:rgba(0,0,0,.35)}dialog form{padding:18px 22px}
.dh{display:flex;align-items:center;justify-content:space-between}.dh h2{margin:0}
fieldset{border:0;border-top:1px solid var(--line);margin:14px 0 0;padding:12px 0 0}legend{font-weight:600;padding:0 0 6px;font-size:14px}
.opt{display:inline-flex;align-items:center;gap:6px;margin:4px 14px 4px 0;font-size:14px;cursor:pointer}
.hours{display:flex;gap:16px;margin-top:8px;font-size:13px;color:var(--muted)}.hours label{display:flex;align-items:center;gap:8px}
.hours input{width:64px;background:var(--card);color:var(--fg);border:1px solid var(--line);border-radius:6px;padding:4px 6px;font:inherit}
.df{display:flex;justify-content:flex-end;gap:10px;margin-top:18px}
.df button{font:inherit;border-radius:8px;padding:6px 14px;cursor:pointer;border:1px solid var(--line)}.primary{background:var(--c1);color:#fff;border-color:var(--c1)!important}.ghost{background:var(--card);color:var(--fg)}
#manual{max-width:860px}#manual table{margin:10px 0}#manual pre{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px 14px;overflow:auto}
#manual code{font-size:13px}#manual h1{font-size:24px}#manual h2{border-top:1px solid var(--line);padding-top:18px}
main:has(#office.on){max-width:1440px}
.ofc-head{display:flex;flex-wrap:wrap;gap:10px;align-items:center;justify-content:space-between}.ofc-head h2{margin:0}
.ofc-tools{display:flex;gap:8px}.ofc-tools .pill-btn[aria-pressed=true]{background:var(--c1);color:#fff;border-color:var(--c1)}
.ofc-stats{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:14px;margin:14px 0}
.ofc-stat{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:12px 16px;display:flex;flex-direction:column}
.ofc-stat b.n{font-size:28px;line-height:1.2}.ofc-stat span{color:var(--muted);font-size:13px}
.ofc{display:grid;grid-template-columns:220px minmax(0,1fr) 270px;gap:16px;align-items:start}
.ofc-card{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:14px}.ofc-card h3{margin:0 0 10px;font-size:15px}
.ofc-roster{display:flex;flex-direction:column;gap:6px}
.ofc-roster button{display:flex;justify-content:space-between;gap:6px;align-items:center;min-height:44px;padding:4px 10px;border-radius:10px;border:1px solid var(--line);background:transparent;color:var(--fg);font:inherit;font-size:13px;cursor:pointer;text-align:left}
.ofc-roster button.sel{border-color:var(--c1);box-shadow:0 0 0 1px var(--c1)}.ofc-roster small{display:block;color:var(--muted);font-size:11px}
.ofc-roster .s{font-size:12px;text-align:right}.ofc-s-w{color:var(--c2)}.ofc-s-a{color:#b7701a}.ofc-s-e{color:var(--c3)}.ofc-s-i{color:var(--muted)}
.ofc-mid{display:flex;flex-direction:column;gap:12px;min-width:0}
.ofc-scene{position:relative;aspect-ratio:1;border:1px solid var(--line);border-radius:14px;overflow:hidden;background:#cfcfd1}
.ofc-stage{position:absolute;left:0;top:0;width:1024px;height:1024px;transform-origin:0 0;transition:transform .8s ease}
.ofc-stage>img,.ofc-stage>div,.ofc-stage>svg{position:absolute}
:root[data-theme=dark] .ofc-stage{filter:brightness(.72) saturate(.9)}
.ofc-man{background-repeat:no-repeat;cursor:default}.ofc-man.away{opacity:0;transition:opacity .4s}
.ofc-tag{z-index:99999;transform:translate(-50%,-100%);display:flex;flex-direction:column;align-items:center;pointer-events:none}
.ofc-bub{background:#fff;color:#111;border-radius:7px;padding:1px 5px;font-size:11px;font-weight:700;white-space:nowrap;border:2px solid #1c1b19;display:flex;gap:3px;align-items:center;box-shadow:0 1px 3px rgba(0,0,0,.35)}
.ofc-bub svg{width:13px;height:13px;stroke:#1c1b19;fill:none;stroke-width:2.4;stroke-linecap:round;stroke-linejoin:round}
.ofc-tail{width:0;height:0;border:4px solid transparent;border-top-color:#1c1b19;margin-top:-1px}
.ofc-tag.alert .ofc-bub{background:#f0b66a}.ofc-tag.err .ofc-bub{background:#f3a59b}.ofc-tag.done .ofc-bub{background:#dff1e8}.ofc-tag.hand .ofc-bub{background:#2d5fb0;color:#fff}.ofc-tag.hand .ofc-bub svg{stroke:#fff}
.ofc-tag.idle .ofc-bub,.ofc-tag.idle .ofc-tail{display:none}
.ofc-nm{display:none;background:#151513;color:#f4f4f2;border-radius:999px;padding:0 6px;font-size:11px;line-height:1.5;margin-top:2px}
.ofc-stage.names .ofc-nm,.ofc-tag.hover .ofc-nm{display:block}
.ofc-fx{left:0;top:0;width:1024px;height:1024px;z-index:99998;pointer-events:none}
.ofc-line{fill:none;stroke:#2d5fb0;stroke-width:4;stroke-dasharray:10 8;stroke-linecap:round;opacity:0;transition:opacity .3s}.ofc-line.on{opacity:.9;animation:ofcdash 1s linear infinite}@keyframes ofcdash{to{stroke-dashoffset:-36}}
.ofc-env{fill:#fff;stroke:#2d5fb0;stroke-width:3;opacity:0}
.ofc-detail{display:flex;flex-wrap:wrap;gap:8px 28px}.ofc-detail div{display:flex;flex-direction:column}.ofc-detail small{color:var(--muted);font-size:12px}
.ofc-side{display:flex;flex-direction:column;gap:16px}
.ofc-tl{font-size:12px;margin:3px 0 2px;display:flex;flex-wrap:wrap;gap:2px 6px;align-items:center}.ofc-ti{width:12px;height:12px;stroke:currentColor;fill:none;stroke-width:2.2;stroke-linecap:round;stroke-linejoin:round;margin-right:2px;vertical-align:-1px}.ofc-ok{color:var(--c2)}.ofc-run{color:var(--c1)}
.ofc-must ul{max-height:420px;overflow:auto}.ofc-n-e b{color:var(--c3)}.ofc-n-a b{color:#b7701a}
.ofc-must{border-color:#d9a55c;background:color-mix(in srgb,#f0b66a 12%,var(--card))}.ofc-must ul{margin:0;padding-left:18px;font-size:13px}.ofc-must li{padding:3px 0}
.ofc-hand{list-style:none;margin:0;padding:0;font-size:13px}.ofc-hand li{padding:7px 0;border-bottom:1px solid var(--line)}.ofc-hand li:last-child{border-bottom:0}.ofc-hand li span{color:var(--muted);font-size:12px}
.ofc-legend{display:flex;flex-wrap:wrap;gap:6px 12px;font-size:12px;color:var(--muted);margin-top:12px}.ofc-legend span{display:flex;gap:4px;align-items:center}.ofc-legend svg{width:14px;height:14px;stroke:currentColor;fill:none;stroke-width:2.2;stroke-linecap:round;stroke-linejoin:round}
.ofc-stagebar{margin-top:16px}.ofc-stagebar ol{list-style:none;margin:0;padding:0;display:grid;grid-template-columns:repeat(5,minmax(0,1fr));gap:8px}
.ofc-stagebar li{border:1px solid var(--line);border-radius:10px;padding:8px 10px;font-size:13px;display:flex;flex-direction:column}.ofc-stagebar li small{color:var(--muted);font-size:11px}
.ofc-stagebar li.done{background:color-mix(in srgb,var(--c2) 14%,var(--card))}.ofc-stagebar li.now{border-color:var(--c1);background:color-mix(in srgb,var(--c1) 14%,var(--card))}
@media (max-width:1100px){.ofc{grid-template-columns:1fr}.ofc-stats{grid-template-columns:1fr}}
@media (prefers-reduced-motion:reduce){.ofc-stage,.ofc-line,.ofc-man{transition:none;animation:none}}
@media (max-width:700px){.charts{grid-template-columns:1fr}.feed .fp{width:110px}.donutwrap{flex-direction:column}}
"#;

/// Runs before paint: settings, theme, language, favicon.
pub(crate) const HEAD_JS: &str = r#"(function(){var d=document.documentElement;
function S(k,def){try{var v=localStorage.getItem('uno-'+k);return v===null?def:v}catch(e){return def}}
window.unoS=S;
function pick(){var m=S('theme','auto'),day=+S('day','6'),night=+S('night','18');
var h=new Date().getHours();var lightNow=day<night?(h>=day&&h<night):(h>=day||h<night);
var th=m==='auto'?(lightNow?'light':'dark'):m;d.dataset.theme=th;d.dataset.mode=m;
var bg=th==='dark'?'%23FFFFFF':'%23111111',fg=th==='dark'?'%23111111':'%23FFFFFF';
var svg="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='"+bg+"'/%3E%3Cpath d='M18.5 18V33.5A11 11 0 0 0 40.5 33.5V20M35 25.5L40.5 20L46 25.5' fill='none' stroke='"+fg+"' stroke-width='5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E";
var f=document.getElementById('fav');if(!f){f=document.createElement('link');f.id='fav';f.rel='icon';f.type='image/svg+xml';document.head.appendChild(f);}f.href=svg;}
window.unoTheme=pick;pick();setInterval(pick,60000);
var l=S('lang','en');d.dataset.lang=l;d.lang=l;})();"#;

pub(crate) const JS: &str = r#"
var D=document.documentElement,S=window.unoS;
function put(k,v){try{localStorage.setItem('uno-'+k,v)}catch(e){}}
function setLang(l){D.dataset.lang=l;D.lang=l;put('lang',l);
 document.querySelectorAll('option[data-en]').forEach(o=>o.textContent=o.dataset[l]);
 document.title=l==='th'?'UNO — แดชบอร์ดและคู่มือการใช้งาน':'UNO — Dashboard and Manual';ages();}
document.querySelectorAll('#lang button').forEach(b=>b.onclick=()=>{setLang(b.dataset.l);syncForm()});
document.querySelectorAll('nav button').forEach(b=>b.onclick=()=>{
 document.querySelectorAll('nav button,.view').forEach(e=>e.classList.remove('on'));
 b.classList.add('on');document.getElementById(b.dataset.v).classList.add('on');put('view',b.dataset.v)});
document.getElementById('theme').onclick=()=>{var o=['auto','light','dark'];var m=D.dataset.mode||'auto';
 put('theme',o[(o.indexOf(m)+1)%3]);window.unoTheme();syncForm()};
function showP(id,keep){if(!document.getElementById(id))id='all';
 document.querySelectorAll('.pview').forEach(e=>e.classList.remove('on'));document.getElementById(id).classList.add('on');
 document.getElementById('psel').value=id;document.querySelector('.pbar').classList.toggle('inp',id!=='all');
 if(!keep)window.scrollTo(0,0);put('proj',id)}
function bindDyn(){
 document.getElementById('psel').onchange=e=>showP(e.target.value);
 document.querySelector('.pbar .back').onclick=()=>showP('all');
 document.querySelectorAll('tr.go').forEach(r=>r.onclick=e=>{e.preventDefault();showP(r.dataset.go)});}
bindDyn();
// relative times + live status
function ago(iso){var t=Date.parse(iso);if(isNaN(t))return '';var m=Math.round((Date.now()-t)/60000),th=D.dataset.lang==='th';
 if(m<1)return th?'เมื่อสักครู่':'just now';if(m<60)return th?m+' นาทีที่แล้ว':m+' min ago';
 var h=Math.round(m/60);if(h<48)return th?h+' ชั่วโมงที่แล้ว':h+' h ago';var dd=Math.round(h/24);return th?dd+' วันที่แล้ว':dd+' days ago'}
function dur(iso){var t=Date.parse(iso);if(isNaN(t))return '';var s=Math.max(0,Math.floor((Date.now()-t)/1000)),th=D.dataset.lang==='th';
 if(s<60)return s+(th?' วินาที':' s');if(s<3600)return Math.floor(s/60)+(th?' นาที':' min');return Math.floor(s/3600)+(th?' ชั่วโมง':' h')}
// a session quiet for 30 minutes counts as closed, as in the session count
function tick1(){document.querySelectorAll('.since').forEach(e=>e.textContent=dur(e.dataset.since));
 document.querySelectorAll('.livecard').forEach(c=>{var n=0;c.querySelectorAll('.lc-line').forEach(l=>{
  var old=!(Date.now()-Date.parse(l.dataset.time)<=1800000);l.classList.toggle('stale',old);if(!old)n++});c.classList.toggle('has-lines',n>0)})}
setInterval(tick1,1000);
function ages(){tick1();var win=+S('window','15');
 document.querySelectorAll('.ago').forEach(e=>e.textContent=ago(e.dataset.time));
 document.querySelectorAll('.livecard').forEach(c=>{var t=Date.parse(c.dataset.last),s=c.dataset.sessions,
  recent=!isNaN(t)&&(Date.now()-t)<win*60000,open=s!=='?'&&+s>0;
  c.classList.remove('working','open','idle');c.classList.add(recent&&s!=='0'?'working':(open?'open':'idle'))})}
setInterval(ages,30000);
// settings
var dlg=document.getElementById('settings');
document.getElementById('gear').onclick=()=>{syncForm();dlg.showModal()};
function syncForm(){[['s-lang','lang','en'],['s-theme','theme','auto'],['s-reload','reload','0'],['s-window','window','15'],['s-start','start','last'],['s-live','live','3']].forEach(([n,k,def])=>{
 var v=S(k,def);document.querySelectorAll('input[name="'+n+'"]').forEach(i=>i.checked=i.value===v)});
 document.getElementById('s-day').value=S('day','6');document.getElementById('s-night').value=S('night','18')}
dlg.addEventListener('change',e=>{var i=e.target,map={'s-lang':'lang','s-theme':'theme','s-reload':'reload','s-window':'window','s-start':'start','s-live':'live'};
 if(map[i.name]){put(map[i.name],i.value)}else if(i.id==='s-day'){put('day',Math.max(0,Math.min(23,+i.value||0)))}else if(i.id==='s-night'){put('night',Math.max(0,Math.min(23,+i.value||0)))}
 setLang(S('lang','en'));window.unoTheme();ages();armReload();if(window.unoLiveArm)window.unoLiveArm()});
document.getElementById('s-reset').onclick=()=>{['lang','theme','reload','window','start','day','night','proj','view','live'].forEach(k=>{try{localStorage.removeItem('uno-'+k)}catch(e){}});
 setLang('en');window.unoTheme();syncForm();ages();armReload()};
var rt=null;function armReload(){if(rt)clearTimeout(rt);var m=+S('reload','0');var b=document.getElementById('reloadflag');
 b.hidden=!m;if(m){b.querySelectorAll('.n').forEach(x=>x.textContent=m);rt=setTimeout(()=>location.reload(),m*60000)}}
setLang(S('lang','en'));ages();armReload();
if(S('start','last')==='last'){try{var p=localStorage.getItem('uno-proj');if(p)showP(p)}catch(e){}
 try{var v=localStorage.getItem('uno-view');if(v)document.querySelector('nav button[data-v="'+v+'"]').click()}catch(e){}}
// Office tab: people in a pixel office, from #ofc-data (live pane). Desk/chair art and seats come
// from window.UNO_OFFICE; routes and lounge spots below must match assets/office/floor.webp.
(function(){var st=document.getElementById('ofc-st'),O=window.UNO_OFFICE;if(!st||!O)return;
var L=O.layout,PH=L.ph,PW=Math.round(56*PH/100),FRESH=1800000;
var ICON={bash:'<path d="M4 6l5 5-5 5M11 17h9"/>',edit:'<path d="M14 4l6 6L9 21H3v-6z"/>',read:'<circle cx="10" cy="10" r="6"/><path d="M15 15l5 5"/>',
 agent:'<path d="M4 12h14M13 6l6 6-6 6"/>',wait:'<path d="M9 9a3 3 0 1 1 4 2.8c-.7.3-1 .8-1 1.5V15"/><path d="M12 19h0"/>',err:'<path d="M12 5v9M12 19h0"/>',
 done:'<path d="M5 12l5 5 9-10"/>',think:'<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-4 10.5c.7.7 1 1.5 1 2.5h6c0-1 .3-1.8 1-2.5A6 6 0 0 0 12 3z"/>'};
var TXT={bash:['running a command','รันคำสั่ง'],edit:['editing files','แก้ไฟล์'],read:['reading code','อ่าน / ค้นโค้ด'],agent:['handing off work','ส่งงานต่อ'],
 think:['working','กำลังทำงาน'],wait:['waiting for you','รอคุณตอบ'],err:['stopped by an error','หยุดเพราะ error'],done:['turn finished','ตอบรอบนั้นเสร็จ']};
var PLACE={WC:['restroom','ห้องน้ำ'],SRV:['server room','หน้าห้อง Server'],BAR:['coffee bar','เคาน์เตอร์กาแฟ'],KIT:['kitchen','ห้องครัว'],DIN1:['dining table','โต๊ะกินข้าว'],
 DIN2:['dining table','โต๊ะกินข้าว'],ARC:['arcade','ตู้เกม'],BEAN:['bean bag','บีนแบ็ก'],SOFA:['sofa','โซฟา'],MAS:['massage chair','เก้าอี้นวด'],FOOS:['foosball','โต๊ะฟุตบอล'],DOOR:["Uno's door",'หน้าห้อง Uno']};
function th(){return D.dataset.lang==='th'}
function tx(p){return th()?p[1]:p[0]}
function svg(k){return '<svg viewBox="0 0 24 24" aria-hidden="true">'+ICON[k]+'</svg>'}
function esc(s){return String(s||'').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;')}
function iconFor(m){var s=m.state,t=m.tool||'';if(s==='waiting')return 'wait';if(s==='error')return 'err';if(s==='done')return 'done';
 if(s==='running'){if(/^(Bash|BashOutput|KillShell)$/.test(t))return 'bash';if(/^(Edit|Write|MultiEdit|NotebookEdit)$/.test(t))return 'edit';
  if(/^(Read|Grep|Glob|LS|WebFetch|WebSearch)$/.test(t))return 'read';if(/^(Agent|Task)$/.test(t))return 'agent'}return 'think'}
function el(tag,cls,z){var e=document.createElement(tag);if(cls)e.setAttribute('class',cls);if(z!=null)e.style.zIndex=z;st.appendChild(e);return e}
function img(src,x,y,z){var e=el('img',null,z);e.src=src;e.alt='';e.style.left=x+'px';e.style.top=y+'px';return e}
img(O.floor,0,0,0).style.width='1024px';
L.desks.forEach(function(d,i){img(O.chairs[i],d.x,d.y,d.z-2);img(O.desks[i],d.x,d.y,d.z)});
img(O.ceo,L.ceo.x,L.ceo.y,L.ceo.z);
var fx=document.createElementNS('http://www.w3.org/2000/svg','svg');fx.setAttribute('class','ofc-fx');fx.setAttribute('viewBox','0 0 1024 1024');
fx.innerHTML='<path class="ofc-line" d=""/><circle class="ofc-env" r="7"/>';st.appendChild(fx);var line=fx.firstChild,env=fx.lastChild;
var ROLES={};document.querySelectorAll('.ofc-roles i').forEach(function(r){ROLES[r.dataset.who]=[r.dataset.en,r.dataset.th]});
var N=Object.keys(ROLES);
/* walk graph, screen px on the 1024 floor */
var NODE={L1:[230,470],L2:[330,440],S1:[470,398],M:[300,572],F1:[390,650],F2:[520,690],K1:[340,716],G:[545,795],LG:[620,755],DOORW:[700,440]};
var EDGE=[['L1','L2'],['L2','S1'],['L1','M'],['M','F1'],['F1','F2'],['F1','K1'],['F2','G'],['G','LG']];
/* Uno's room: chair, behind the desk, along its left side, out through the glass door */
NODE.U0=L.uno;NODE.U1=[806,352];NODE.U2=[780,365];NODE.U3=[756,385];NODE.U4=[738,410];EDGE.push(['U0','U1'],['U1','U2'],['U2','U3'],['U3','U4'],['U4','DOOR']);
[[1,2,3,4],[5,6,7,8],[9,10,11,12]].forEach(function(r,ri){var prev=null;r.forEach(function(n,j){var s=L.desks[n-1].seat,k='b'+n;NODE[k]=[s[0]+16,s[1]-8];NODE['s'+n]=s;EDGE.push(['s'+n,k]);
 if(j===0){NODE['e'+ri]=[NODE[k][0]-45,NODE[k][1]-23];EDGE.push(['e'+ri,k]);EDGE.push(['e'+ri,ri===0?'S1':'L2'])}if(prev)EDGE.push([prev,k]);prev=k})});
EDGE.push(['b3','DOORW']);EDGE.push(['b4','DOORW']);
/* aisle along the right ends of the rows, by the lounge wall: Uno's short way to the front rows */
NODE.R1=[770,488];NODE.R2=[690,548];NODE.R3=[600,600];EDGE.push(['b4','R1'],['R1','R2'],['R2','b8'],['R2','R3'],['R3','b12']);
var SPOT=[['WC',[300,398],'L2','door'],['SRV',[415,418],'L2','back'],['BAR',[515,362],'e0','back'],['KIT',[258,702],'K1','back'],['DIN1',[392,708],'F1','front'],
 ['DIN2',[500,748],'F2','front'],['ARC',[690,724],'LG','back'],['BEAN',[758,716],'LG','front'],['SOFA',[772,655],'LG','front'],['MAS',[598,772],'G','front'],
 ['FOOS',[640,806],'LG','back'],['DOOR',[722,430],'DOORW','back']];
SPOT.forEach(function(s){NODE[s[0]]=s[1];EDGE.push([s[0],s[2]])});EDGE.push(['ARC','BEAN']);EDGE.push(['BEAN','SOFA']);
var ADJ={};EDGE.forEach(function(e){(ADJ[e[0]]=ADJ[e[0]]||[]).push(e[1]);(ADJ[e[1]]=ADJ[e[1]]||[]).push(e[0])});
function dd(a,b){return Math.hypot(NODE[a][0]-NODE[b][0],NODE[a][1]-NODE[b][1])}
function route(a,b){var dist={},prev={},done={};dist[a]=0;for(;;){var u=null;for(var k in dist)if(!done[k]&&(u===null||dist[k]<dist[u]))u=k;if(u===null||u===b)break;done[u]=1;
  (ADJ[u]||[]).forEach(function(v){var nd=dist[u]+dd(u,v);if(dist[v]==null||nd<dist[v]){dist[v]=nd;prev[v]=u}})}
 var out=[];for(var x=b;x!=null;x=prev[x])out.unshift(x);return out[0]===a?out:[a,b]}
/* people */
var C=N.map(function(n,i){var m=el('div','ofc-man');m.style.width=PW+'px';m.style.height=PH+'px';m.style.backgroundImage='url('+O.people+')';m.style.backgroundSize=(PW*7)+'px '+(PH*N.length)+'px';
 var tag=el('div','ofc-tag idle');tag.innerHTML='<span class="ofc-bub"></span><i class="ofc-tail"></i><b class="ofc-nm"></b>';tag.lastChild.textContent=n;
 var c={i:i,n:n,m:m,tag:tag,bub:tag.firstChild,x:0,y:0,path:[],at:null,goal:null,until:0,t:0,info:null,seen:false};
 if(i===0){c.uno=true;c.seat='U0';c.dz=L.ceo.z}else{c.seat='s'+i;c.dz=L.desks[i-1].z}
 m.onmouseenter=function(){tag.classList.add('hover')};m.onmouseleave=function(){tag.classList.remove('hover')};m.onclick=function(){SEL=n;panels()};return c});
function taken(k){return C.some(function(c){return c.goal===k||c.at===k})}
function pick(){var f=SPOT.map(function(s){return s[0]}).filter(function(k){return k!=='DOOR'&&!taken(k)});return f.length?f[Math.floor(Math.random()*f.length)]:'BAR'}
function spotOf(k){for(var i=0;i<SPOT.length;i++)if(SPOT[i][0]===k)return SPOT[i];return null}
function go(c,g){c.m.classList.remove('away');var p=route(c.at||c.last||c.seat,g);c.path=p.slice(1).map(function(k){return NODE[k]});c.at=null;c.goal=g;c.last=g;
 if(matchMedia('(prefers-reduced-motion: reduce)').matches){var e=c.path[c.path.length-1];c.x=e[0];c.y=e[1];c.path=[];arrive(c)}}
function jump(c,g){c.x=NODE[g][0];c.y=NODE[g][1];c.path=[];c.goal=g;c.last=g;arrive(c)}
function arrive(c){c.at=c.goal;c.goal=null;c.t=0;if(c.at===c.seat){if(c.uno)c.errand=null;return}
 if(c.uno&&c.errand&&c.at===c.errand.node){c.face='front';c.until=Date.now()+4000;deliver(c.errand);return}
 var sp=spotOf(c.at);c.face=sp?sp[3]:'front';
 if(c.at==='DOOR'){c.until=Date.now()+3500;return}
 /* offline: short stops so people keep walking around */c.until=Date.now()+(sp&&sp[3]==='door'?4000+Math.random()*4000:3000+Math.random()*5000);if(sp&&sp[3]==='door')c.m.classList.add('away')}
/* data */
var SEL='Uno',handSeen=null,QUEUE=[],BUSY=0,REPLAY=600000,LAST={};
setInterval(function(){if(QUEUE.length&&Date.now()>BUSY&&!C[0].errand){BUSY=Date.now()+1500;handOff(QUEUE.shift())}},500);
function data(){var out={m:{},h:[],t:{},w:[],n:[],open:0};document.querySelectorAll('#ofc-data .s').forEach(function(e){out.open+=+e.dataset.open||0});document.querySelectorAll('#ofc-data .m').forEach(function(e){var d=e.dataset,f=d.time&&Date.now()-Date.parse(d.time)<=FRESH;out.m[d.who]=f?d:null});
 document.querySelectorAll('#ofc-data .h').forEach(function(e){out.h.push(e.dataset)});document.querySelectorAll('#ofc-data .w').forEach(function(e){out.w.push(e.dataset)});document.querySelectorAll('#ofc-data .n').forEach(function(e){out.n.push(e.dataset)});document.querySelectorAll('#ofc-data .t').forEach(function(e){out.t[e.dataset.project+'/'+e.dataset.id]=e.dataset});return out}
var DATA=data(),ONLINE=false;
/* online: a Claude Code session is open (or any fresh line); then everyone sits, else everyone goes to the lounge */
/* who sits: Uno while a session is open; a member only while working (a fresh line) or receiving a hand-off */
function wants(c){return c.uno?ONLINE:!!(c.info||c.handoff)}
function online(){if(DATA.open>0)return true;for(var k in DATA.m)if(DATA.m[k])return true;return false}
function read(){DATA=data();ONLINE=online();DATA.h.slice().reverse().forEach(function(h){LAST[h.who]=h});
 C.forEach(function(c){var m=DATA.m[c.n];c.info=m;
  if(c.handoff){c.bub.innerHTML=svg('agent')+esc(tx(['new task','รับงาน']));c.tag.className='ofc-tag hand'}
  else if(m){var k=iconFor(m);c.bub.innerHTML=svg(k)+esc(m.task||'');c.tag.className='ofc-tag'+(k==='wait'?' alert':k==='err'?' err':k==='done'?' done':'')}
  else{c.bub.innerHTML='';c.tag.className='ofc-tag idle'}
  if(!c.seen){c.seen=true;jump(c,wants(c)?c.seat:pick());return}
  if(c.errand)return;
  if(wants(c)&&c.goal!==c.seat&&c.at!==c.seat)go(c,c.seat);
  if(!wants(c)&&(c.at===c.seat||c.goal===c.seat))go(c,pick())});
 var keys=DATA.h.map(function(h){return h.time+'|'+h.who});
 if(handSeen===null){handSeen=keys;DATA.h.filter(function(h){return Date.now()-Date.parse(h.time)<=REPLAY}).slice(0,3).reverse().forEach(function(h){QUEUE.push(h)})}
 else{keys.forEach(function(k,i){if(handSeen.indexOf(k)<0)QUEUE.push(DATA.h[i])});handSeen=keys}
 panels()}
function handOff(h){var b=C.find(function(c){return c.n===h.who});if(!b||b.uno)return;LAST[b.n]=h;var a=C[0];
 b.handoff=true;if(b.at!==b.seat&&b.goal!==b.seat)go(b,b.seat);
 a.errand={h:h,b:b,node:'b'+b.i};if(!ONLINE||!NODE[a.errand.node]){a.errand=null;deliver({h:h,b:b});return}go(a,a.errand.node)}
function deliver(e){var b=e.b,a=C[0];
 var A0=[a.x,a.y-PH*.8],B0=[b.x,b.y-PH];line.setAttribute('d','M'+A0[0]+' '+A0[1]+' Q'+((A0[0]+B0[0])/2)+' '+(Math.min(A0[1],B0[1])-80)+' '+B0[0]+' '+B0[1]);line.classList.add('on');
 var len=line.getTotalLength(),t0=null;env.style.opacity=1;
 (function fly(ts){t0=t0||ts;var u=Math.min(1,(ts-t0)/1800),p=line.getPointAtLength(u*len);env.setAttribute('cx',p.x);env.setAttribute('cy',p.y);
  if(u<1)requestAnimationFrame(fly);else{env.style.opacity=0;setTimeout(function(){line.classList.remove('on')},5000);b.handoff=true;read();setTimeout(function(){b.handoff=false;read()},6000)}})(performance.now())}
/* hand-off timeline */
function hm(x){var d=new Date(x);return isNaN(d)?'':d.toLocaleTimeString(th()?'th-TH':'en-GB',{hour:'2-digit',minute:'2-digit'})}
var KIND={read:/^(Read|Grep|Glob|LS|WebFetch|WebSearch)$/,bash:/^(Bash|BashOutput|KillShell)$/,edit:/^(Edit|Write|MultiEdit|NotebookEdit)$/};
function handCard(h){var parts=[],n={read:0,bash:0,edit:0,other:0};
 (h.tools||'').split(',').forEach(function(x){if(!x)return;var p=x.split(':'),c=+p[1]||0,k='other';for(var q in KIND)if(KIND[q].test(p[0]))k=q;n[k]+=c});
 var line;
 if(h.start){parts.push(esc(tx(['started','เริ่ม']))+' '+esc(hm(h.start)));
  [['read',['read','อ่าน/ค้น']],['bash',['commands','รันคำสั่ง']],['edit',['edits','แก้ไฟล์']],['other',['other tools','อื่นๆ']]].forEach(function(k){if(n[k[0]])parts.push(svgSmall(k[0])+esc(tx(k[1])+' '+n[k[0]]))});
  if(h.end){var m=Math.max(0,Math.round((Date.parse(h.end)-Date.parse(h.start))/60000));parts.push('<b class="ofc-ok">'+esc(tx(['done','เสร็จ']))+' '+esc(hm(h.end))+' ('+m+' '+esc(tx(['min','นาที']))+')</b>')}
  else if(!h.closed&&Date.now()-Date.parse(h.last||h.start)<=FRESH)parts.push('<b class="ofc-run">'+esc(tx(['working','กำลังทำ']))+'</b>');
  else parts.push('<span class="muted">'+esc(tx(['outcome unknown','ไม่ทราบผล']))+'</span>');
  line=parts.join(' · ')}
 else line='<span class="muted">'+esc(Date.now()-Date.parse(h.time)<=FRESH&&!h.closed?tx(['not started yet','ยังไม่เริ่ม']):tx(['no work record','ไม่มีบันทึกการทำงาน']))+'</span>';
 return '<li><b>Uno → '+esc(h.who)+'</b> '+esc(h.desc)+'<div class="ofc-tl">'+line+'</div><span class="muted">'+esc(ago(h.time))+' · '+esc(h.project)+'</span></li>'}
function svgSmall(k){return ICON[k==='other'?'think':k]?'<svg class="ofc-ti" viewBox="0 0 24 24" aria-hidden="true">'+ICON[k==='other'?'think':k]+'</svg>':''}
/* panels */
function stateOf(c){if(!c.info)return ['i',!ONLINE?tx(['offline','ออฟไลน์']):c.uno?tx(['free, in his office','ว่าง (อยู่ในห้อง)']):tx(['free, taking a break','ว่าง (พักอยู่)'])];var k=iconFor(c.info);return [k==='wait'?'a':k==='err'?'e':'w',(c.info.task?c.info.task+' · ':'')+tx(TXT[k])]}
function panels(){var w=0,b=0,a=0,ro=document.getElementById('ofc-roster'),err=[],wait=[];ro.innerHTML='';
 C.forEach(function(c){var s=stateOf(c);if(s[0]==='i')a++;else if(s[0]==='w')w++;else{(s[0]==='e'?err:wait).push('<li class="ofc-n-'+s[0]+'"><b>'+esc(c.n)+'</b> · '+esc(s[1])+(c.info&&c.info.project?' <span class="muted">· '+esc(c.info.project)+'</span>':'')+'</li>')}
  var bt=document.createElement('button');bt.type='button';if(c.n===SEL)bt.className='sel';
  bt.innerHTML='<span><b>'+esc(c.n)+'</b><small>'+esc(tx(ROLES[c.n]))+'</small></span><span class="s ofc-s-'+s[0]+'">'+esc(s[1])+'</span>';bt.onclick=function(){SEL=c.n;panels()};ro.appendChild(bt)});
 var turns=DATA.w.filter(function(x){return Date.now()-Date.parse(x.time)<=FRESH}).map(function(x){return '<li class="ofc-n-a"><b>Uno</b> · '+esc(tx(['finished a turn, waiting for your reply','ตอบเสร็จ รอคำตอบจากคุณ']))+' <span class="muted">· '+esc(x.project)+(x.task?' · '+esc(x.task):'')+' · '+esc(ago(x.time))+'</span></li>'});
 var TS={escalated:['needs your decision','ต้องให้คุณตัดสินใจ'],'in-review':['waiting for your review or merge','รอคุณรีวิว / merge'],proposed:['waiting for your approval','รอคุณอนุมัติ']};
 var tasks=['escalated','in-review','proposed'].map(function(st){return DATA.n.filter(function(x){return x.status===st}).map(function(x){return '<li class="ofc-n-t"><b>'+esc(x.id)+'</b> '+esc(x.title)+'<br><span class="muted">'+esc(tx(TS[st]))+' · '+esc(x.project)+'</span></li>'}).join('')}).join('');
 var must=err.concat(wait,turns);b=must.length+DATA.n.length;
 document.getElementById('ofc-must').innerHTML=(must.join('')+tasks)||'<li class="muted">'+esc(tx(['Nothing','ไม่มี']))+'</li>';
 document.getElementById('ofc-stats').innerHTML=[[w,['Working','กำลังทำงาน'],['Including Uno','รวม Uno']],[b,['Needs you','ต้องดู'],['Replies, approvals, reviews, errors','รอคุณตอบ อนุมัติ รีวิว หรือ error']],
  ONLINE?[a,['Free','ว่าง'],['No task now, taking a break','ยังไม่มีงาน กำลังพัก']]:[a,['Offline','ออฟไลน์'],['No Claude Code session open','ไม่มี session ของ Claude Code เปิดอยู่']]].map(function(x){return '<div class="ofc-stat"><b class="n">'+x[0]+'</b><b>'+esc(tx(x[1]))+'</b><span>'+esc(tx(x[2]))+'</span></div>'}).join('');
 var c=C.find(function(x){return x.n===SEL}),s=stateOf(c),where=c.at===c.seat?(c.uno?tx(["Uno's office",'ห้อง Uno']):tx(['at the desk','ที่โต๊ะ'])):c.errand&&c.at===c.errand.node?tx(["at "+c.errand.b.n+"'s desk",'ที่โต๊ะ '+c.errand.b.n]):c.at&&PLACE[c.at]?tx(PLACE[c.at]):tx(['walking','กำลังเดิน']);
 document.getElementById('ofc-detail').innerHTML='<div><b style="font-size:17px">'+esc(c.n)+'</b><small>'+esc(tx(ROLES[c.n]))+'</small></div><div><small>'+esc(tx(['Now','ตอนนี้']))+'</small>'+esc(s[1])+
  (c.info&&c.info.project?' <span class="muted">· '+esc(c.info.project)+'</span>':'')+'</div><div><small>'+esc(tx(['Where','อยู่ที่']))+'</small>'+esc(where)+'</div>'+(LAST[c.n]?'<div><small>'+esc(tx(['Last task from Uno','งานล่าสุดจาก Uno']))+'</small>'+esc(LAST[c.n].desc)+' <span class="muted">· '+esc(ago(LAST[c.n].time))+'</span></div>':'');
 document.getElementById('ofc-hand').innerHTML=DATA.h.slice(0,12).map(handCard).join('')||'<li class="muted">'+esc(tx(['No hand-offs yet','ยังไม่มีการส่งงาน']))+'</li>';
 var bar=document.getElementById('ofc-stagebar'),pick2=(c.info&&c.info.task)?c:C.find(function(x){return x.info&&x.info.task}),task=pick2&&DATA.t[pick2.info.project+'/'+pick2.info.task];
 if(!task){bar.hidden=true}else{bar.hidden=false;var S=['proposed','ready','in-progress','in-review','done'],SN=[['Proposed','เสนอ'],['Ready','พร้อมทำ'],['In progress','กำลังทำ'],['In review','รอรีวิว'],['Done','เสร็จ']],at=S.indexOf(task.status);
  bar.innerHTML='<h3>'+esc(task.id)+' · '+esc(task.title)+' <span class="muted" style="font-weight:400;font-size:12px">tasks.md · '+esc(task.status)+'</span></h3><ol>'+S.map(function(x,i){return '<li class="'+(at<0?'':i<at?'done':i===at?'now':'')+'"><small>'+(i+1)+'</small><b>'+esc(tx(SN[i]))+'</b></li>'}).join('')+'</ol>'}}
document.getElementById('ofc-legend').innerHTML=['bash','edit','read','agent','think','wait','err','done'].map(function(k){return '<span>'+svg(k)+esc(tx(TXT[k]))+'</span>'}).join('');
/* motion */
var last=performance.now();
function step(now){var dt=Math.min(.1,(now-last)/1000),ms=Date.now();last=now;
 C.forEach(function(c){var f=0,h=PH,top,z;c.t+=dt;
  if(c.path.length){var p=c.path[0],ex=p[0]-c.x,ey=p[1]-c.y,d=Math.hypot(ex,ey),mv=80*dt;
   if(d<=mv){c.x=p[0];c.y=p[1];c.path.shift();if(!c.path.length)arrive(c)}else{c.x+=ex/d*mv;c.y+=ey/d*mv}
   f=(ey>0.01?1:3)+Math.floor(c.t*6)%2;top=c.y-PH;z=Math.round(c.y)}
  else if(c.at===c.seat){h=Math.round(PH*(c.uno?.72:.75));top=c.y-h;z=c.uno?L.ceo.z-10:c.dz-1}
  else{f=c.face==='back'?3:0;top=c.y-PH;z=Math.round(c.y);
   if(c.uno&&c.errand&&c.at===c.errand.node){if(ms>c.until){var back=ONLINE?c.seat:pick();c.errand.node=null;go(c,back);if(back!==c.seat)c.errand=null}}
   else if(ms>c.until&&!wants(c))go(c,pick())}
  c.m.style.backgroundPosition=(-f*PW)+'px '+(-c.i*PH)+'px';c.m.style.height=h+'px';c.m.style.left=(c.x-PW/2)+'px';c.m.style.top=top+'px';c.m.style.zIndex=z;
  c.tag.style.left=c.x+'px';c.tag.style.top=(top-2)+'px'});
 requestAnimationFrame(step)}
var sc=document.getElementById('ofc-sc'),zoom=false;
function fit(){var w=sc.clientWidth;if(!w)return;var s=w/1024;if(!zoom){st.style.transform='scale('+s+')';return}var z=s*2.6;st.style.transform='translate('+(w/2-820*z)+'px,'+(w/2-400*z)+'px) scale('+z+')'}
if(window.ResizeObserver)new ResizeObserver(fit).observe(sc);fit();
document.getElementById('ofc-zoom').onclick=function(){zoom=!zoom;this.setAttribute('aria-pressed',zoom);fit()};
document.getElementById('ofc-names').onclick=function(){this.setAttribute('aria-pressed',st.classList.toggle('names'))};
document.querySelectorAll('#lang button').forEach(function(b){b.addEventListener('click',function(){read();document.getElementById('ofc-legend').innerHTML=['bash','edit','read','agent','think','wait','err','done'].map(function(k){return '<span>'+svg(k)+esc(tx(TXT[k]))+'</span>'}).join('')})});
read();setInterval(read,1000);requestAnimationFrame(step)})();
"#;


/// Live page only (`uno-report --serve`): pane updates over `/ws`; polling `/version` while the
/// socket is down, reconnecting after 1, 2, 4 … 30 s. Runs after `JS` and uses its functions.
pub(crate) const LIVE_JS: &str = r#"
(function(){var lf=document.getElementById('liveflag');lf.hidden=false;
var st=document.createElement('style');st.textContent='#liveflag .l-poll{display:none}#liveflag.poll .l-on{display:none}#liveflag.poll .l-poll{display:inline}#liveflag.poll .ld{background:var(--c4);box-shadow:none}';document.head.appendChild(st);
var lp=document.createElement('span');lp.className='l-poll';lp.innerHTML='<span class="l-en">Polling</span><span class="l-th">ตรวจเป็นระยะ</span>';lf.appendChild(lp);
function setState(s){lf.classList.toggle('poll',s==='poll');lf.classList.toggle('off',s==='off');lf.dataset.state=s}
function flash(){lf.classList.add('flash');setTimeout(()=>lf.classList.remove('flash'),900)}
function keep(){return {cur:document.getElementById('psel').value,opened:[...document.querySelectorAll('#dash details')].map(d=>d.open)}}
function restore(k){bindDyn();setLang(D.dataset.lang);showP(k.cur,true);
 document.querySelectorAll('#dash details').forEach((d,i)=>{if(k.opened[i])d.open=true});ages()}
async function refresh(){var k=keep();
 var h=await (await fetch('/',{cache:'no-store'})).text();var doc=new DOMParser().parseFromString(h,'text/html');
 ['dash','ofc-data','manual'].forEach(id=>{var n=doc.getElementById(id);if(n)document.getElementById(id).innerHTML=n.innerHTML});
 var m=doc.querySelector('header .meta');if(m)document.querySelector('header .meta').innerHTML=m.innerHTML;
 restore(k);flash()}
var first=true;
// A socket's first message carries every pane. If the page holds a different set (projects
// changed while the socket was down, or between loading the page and connecting), rebuild it.
function sameSet(ids){var want=ids.filter(id=>id==='all'||/^p\d+$/.test(id)).sort().join();
 return want===[...document.querySelectorAll('.pview')].map(x=>x.id).sort().join()}
var fresh=false;
function apply(m){var full=fresh;fresh=false;if(m.reload){refresh();return}
 var ids=Object.keys(m.panes||{});
 if(ids.some(id=>!document.getElementById(id))||(full&&!sameSet(ids))){refresh();return}
 if(!ids.length)return;var k=keep();
 ids.forEach(id=>{var el=document.getElementById(id);if(el)el.innerHTML=m.panes[id]});
 if(m.meta!=null)document.querySelector('header .meta').innerHTML=m.meta;
 restore(k);if(!first)flash();window.unoSwaps=(window.unoSwaps||0)+1}
var sig=null,ws=null,backoff=1,poll=null,busy=false;
async function tick(){if(busy)return;busy=true;
 try{var v=await (await fetch('/version',{cache:'no-store'})).text();setState(ws&&ws.readyState===1?'live':'poll');
  if(sig!==null&&v!==sig)await refresh();sig=v}catch(e){setState('off')}busy=false}
function startPoll(){if(poll)return;tick();poll=setInterval(tick,(+S('live','3'))*1000)}
function stopPoll(){if(poll){clearInterval(poll);poll=null}}
window.unoLiveArm=function(){if(poll){stopPoll();startPoll()}};
function retry(){setTimeout(connect,backoff*1000);backoff=Math.min(backoff*2,30)}
function connect(){try{ws=new WebSocket((location.protocol==='https:'?'wss://':'ws://')+location.host+'/ws')}catch(e){ws=null;startPoll();retry();return}
 ws.onopen=function(){backoff=1;fresh=true;stopPoll();setState('live')};
 ws.onmessage=function(e){try{var m=JSON.parse(e.data);apply(m);first=false;sig=m.sig}catch(x){}};
 ws.onclose=function(){ws=null;if(lf.dataset.state!=='off')setState('poll');startPoll();retry()}}
setState('poll');connect()})();
"#;
