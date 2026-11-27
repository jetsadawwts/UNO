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
function ages(){var win=+S('window','15');
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
 ['dash','manual'].forEach(id=>{var n=doc.getElementById(id);if(n)document.getElementById(id).innerHTML=n.innerHTML});
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
