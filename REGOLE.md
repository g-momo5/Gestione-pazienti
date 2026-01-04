# REGOLE: STILE COMUNE E RIUSO COMPONENTI

## Obiettivo
Un solo linguaggio visivo per tutta l’app: colori, tipografia, spaziatura, icone e componenti devono provenire da fonti uniche e riutilizzabili. Niente duplicati locali.

## Flowchart decisionale (testuale)
- **Devo aggiungere un nuovo elemento UI?**
  - **Esiste già un componente simile?** (Card, Button, Input, Select, SectionHeader, IconBadge, Icon)
    - Sì → Riutilizza quel componente. Adatta con varianti/prop, non clonare.
    - No → Crea un nuovo componente **riutilizzabile** in `src/lib/components/ui/` con API chiara (prop, varianti, dimensioni).
  - **Il componente usa colori/tipografia/spaziatura custom?**
    - Sì → Prima aggiungi/usa token in `tailwind.config.js` (colori, radius, font size) o CSS globali (`src/app.css`). Vietato hardcodare valori inline.
    - No → Usa già le classi tailwind/utility esistenti.
  - **Serve un’icona?**
    - Sì → Usa `IconBadge`/`Icon` con nomi definiti; non usare emoji o SVG ad hoc.
  - **Serve uno stile hover/focus?**
    - Sì → Allinea a Button/Input esistenti (stessi ring, border, shadow).

- **Devo modificare uno stile esistente?**
  - Aggiorna la **fonte unica**:
    - Colori/radius/font: `tailwind.config.js`.
    - Spaziature/rounded comuni: `src/app.css` base layer.
    - Icone: mappa in `Icon.svelte`/`IconBadge`.
    - Componenti: il file del componente in `src/lib/components/ui/`.
  - Non applicare override locali se può stare in tema o componente.

- **Devo usare un elemento già usato altrove?**
  - Importa lo stesso componente e passa le prop; se manca una variante, aggiungila nel componente, non duplicarla.

## Regole operative
- **Colori/Tipografia**: modifica solo via tema (`tailwind.config.js`). Niente hex sparsi o font inline.
- **Spacing/Radius/Shadow**: usa i token esistenti; se serve un nuovo valore, aggiungilo al tema (non inline).
- **Icone**: esclusivamente Tabler tramite `Icon`/`IconBadge`. Aggiorna la mappa nomi in `Icon.svelte` se serve un’icona nuova.
- **Componenti**: crea/estendi in `src/lib/components/ui/`; evita duplicati in viste. Prop chiare per varianti/size/stato.
- **Qualsiasi nuovo pattern UI** (sezione, pannello, header con icona/freccia, card speciale, list item, badge): crealo/estrailo come componente o utility condivisa (`src/lib/components/ui/` o `src/app.css`) e riusalo; non duplicare markup/stili nelle viste.
- **Stili globali**: se un pattern tocca più elementi (scrollbar, focus, rounded, chip), mettilo in `src/app.css` (@layer base/utilities).
- **No hardcode**: niente stili inventati in una vista; sempre via tema/utility/componenti esistenti.

## Checklist prima di committare
- Ho riusato componenti esistenti invece di copiarli?
- Ho aggiunto/ritoccato i token nel tema (colori/font/radius/spacing) invece di hardcodare?
- Le icone passano attraverso `Icon`/`IconBadge` con un nome presente in `Icon.svelte`?
- Ho messo gli stili condivisibili in `src/app.css` o nel componente base, non nella vista?
- Ho mantenuto hover/focus/shadow coerenti con Button/Input/Card?
