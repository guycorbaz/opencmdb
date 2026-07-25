# Epic 4 — rapport de fin d'epic

**Pour Guy, à discuter.** Écrit à la clôture du run autonome du 2026-07-25 (mandat : travailler
jusqu'au bout de l'epic, convoquer une party mode sur les problèmes et décider, rapporter à la
fin). La rétrospective se tiendra **avec toi, après cette lecture** — c'est ce que tu as choisi.

Ce rapport ne raconte pas ce qui a été tapé ; il raconte **ce qui a été décidé, par qui, contre
quoi, et ce que ça coûte**. Les six décisions ci-dessous sont celles qui méritent ton arbitrage
a posteriori — les trois premières sont les plus discutables.

---

## Ce qui a été livré

Sept PR mergées, CI verte à chaque fois, aucune poussée directe sur master :

| PR | Contenu | Squash |
|---|---|---|
| #29 | Story 4.14 — famille VRRP/HSRP (MAC virtuelle partagée) | `05b0d1f` |
| #30 | **PRD : réconciliation de FR36** (la plus vieille dette doc) | `0fec753` |
| #31 | Story 4.15 — collision de hostname | `a3ebe43` |
| #32 | Story 4.16 — veth Docker éphémères | `688265a` |
| #33 | Story 4.17 — hostname absent/vide (jamais null) | `cf7f9aa` |
| #35 | Story 4.18 — spec wire-format + 4.19a | `7b93c20` |
| #36 | Clôture d'epic (split 4.19, docs, tracking) | `199ba79` |

**État du corpus sur master :** 25 artefacts verrouillés, 24 pièges, **neuf familles** — chacune
en forme positive ET négative, chacune nommant la RÈGLE et non le résultat. Tests :
**119 (bin) + 86 (core) + 42 (xtask)**. `cargo xtask ci` : toutes gates vertes.

---

## Décision 1 — Amender la marche de vie privée pour la MAC VRRP authentique

*Party mode : Winston, Amelia, Murat — unanimes sur le fond, Amelia et Murat resserrant la
portée.*

**Le problème.** Une MAC virtuelle VRRP authentique (`00:00:5e:00:01:0a`) est **universellement
administrée** : la marche de vie privée du corpus la refuse. Or c'est précisément le préfixe
IANA que la famille existe pour tester (D16 crée le `virtual_device` sur ce fait structurel).

**L'alternative écartée.** Committer un substitut localement administré (`02:00:5e:00:01:0a`).
Winston : *« une spec dont les octets porteurs ne portent pas le fait testé n'est pas une spec
affaiblie — c'est une spec fausse »*. Le jour où l'ingestion d'Epic 5 lit les préfixes IANA,
la fixture ne déclencherait rien et le piège passerait pour de mauvaises raisons.

**La décision.** La marche apprend l'invariant qu'elle a toujours voulu dire : *aucun octet ne
peut identifier un réseau réel*. Une MAC définie par un protocole identifie un **protocole** —
les mêmes octets sur chaque déploiement VRRP de la planète — c'est l'analogue MAC d'une adresse
RFC 5737, et l'architecture elle-même l'imprime en prose. **Portée resserrée par
Amelia/Murat** : seul le bloc VRRP IPv4 sur **5 octets exacts** (`00:00:5e:00:01`), un seul
helper alimentant les deux sites de vérification, bornes prouvées rouges (y compris une mutation
enregistrée). HSRP reste dehors — et Murat a corrigé au passage une approximation de
l'architecture : `00:00:0c:07:ac` est un OUI **Cisco**, pas IANA.

**Ce que ça a coûté / rapporté.** Une exception sanctionnée dans un garde-fou de vie privée.
Le bénéfice inattendu : Amelia a exigé au passage un **garde flag-vs-octets** (le champ
`locally_administered` doit dire la vérité sur le bit U/L) — jusque-là inobservable puisque
tout le corpus était `true`. C'est une invariante gagnée.

**À discuter :** es-tu à l'aise avec le principe « une allowlist fermée, une plage par fixture
qui l'exerce » ? C'est la règle que j'ai appliquée deux fois (voir décision 4).

---

## Décision 2 — Frapper un identifiant de règle : `l2-virtual-mac-prefix`

*Party mode, point disputé et **arbitré**.*

La doctrine du corpus interdit de frapper un id de règle qu'aucune expectation ne cite (4.13
avait refusé un id d'IP-continuité). Murat a montré qu'ici **aucune règle existante ne peut
s'opposer** à la fusion de la VIP dans son maître : le must-merge de multi-nic récompense
justement le groupement de MACs distinctes dont l'uplink concorde — *un moteur suivant le
vocabulaire committé à la lettre fusionnerait la VIP dans son maître*. Seule la lecture du
préfixe l'en empêche.

**J'ai arbitré ainsi :** on frappe l'id (une expectation le cite — la doctrine interdit l'id
orphelin, pas l'id nécessaire), et « no new rule, no score » de D16 tient parce que cet id nomme
une **lecture à l'ingestion**, pas une règle scorée. Mais j'ai tranché **contre** Murat sur le
second point : le must-merge cite `l1-exact-mac`, pas l'id frappé — L1 est déterministe sur
`(l2_domain, mac)`, y compris pour une MAC virtuelle.

**À discuter :** c'est le premier id **structurel** du corpus. Si Epic 5 implémente cette
lecture sous un autre nom, le piège rougira en `rule_mismatch` et il faudra un bump délibéré du
corpus. J'ai considéré cette friction comme *la spec qui fonctionne*, pas comme un défaut —
mais c'est un jugement, et il t'appartient.

---

## Décision 3 — Scinder la story 4.19 et fermer l'epic malgré tout

*Party mode : Winston, Murat, John. La décision la plus structurante du run.*

**Le problème.** 4.18 et 4.19 nommaient des livrables dont le consommateur est le parseur UniFi
d'**Epic 11**, pas le moteur d'identité d'Epic 5. Trois contraintes se sont percutées : le corps
capturé réel ne peut **jamais** être committé (dépôt public, ton réseau) ; le parseur n'existe
pas ; et D45 interdit d'écrire un piège par croyance — *« une porte sur une fausse vérité dont
le rouge n'arrivera jamais »*.

**La décision, en trois temps.**

- **4.18 livrée EN ENTIER, comme spec.** Tout son contenu est de la *mesure*, portée par un
  corps **synthétique** ; le « variant attendu » est exprimé en **Observations attendues** —
  le schéma gelé de D19 étant le contrat de sortie du parseur, l'artefact **contraint Epic 11**
  au lieu de laisser le parseur se certifier lui-même (Murat : *« le pire oracle est un test
  vert de naissance »*). Ligne rouge de Murat adoptée comme critère d'arrêt : toute expectation
  doit être **dérivable** ; sinon c'est un **trou nommé**, jamais une supposition.
- **4.19 scindée.** **4.19a livrée** : le relevé de surface de dérive (127 clés de payload
  contre 7 variantes de `Fact` — une mesure) et la **charte contraignante** pour Epic 11 (un
  champ renommé doit produire une erreur explicite, jamais une collection silencieusement vide ;
  `#[serde(default)]` interdit sur toute collection nourrissant la présence ; l'injection en
  couche A est du théâtre). Winston : *« une exigence sur un composant futur n'est pas de la
  spéculation — c'est de l'architecture »*. **4.19b reportée à Epic 11** : le générateur, les
  ~30 fixtures générées et leurs résultats de parsing attendus. Murat voulait le générateur
  maintenant ; **j'ai tranché pour Winston** — un générateur n'a aucun test qui rougit sans le
  parseur qu'il attaque, et la règle maison est « pas de garde sans rouge ».
- **L'epic ferme `done`.** John : *« un epic “in progress” qui ne progresse pas est un mensonge
  dans le burndown ; le jour où une ligne du fichier de statut est connue fausse, plus personne
  ne croit les autres »*. La promesse d'Epic 4 envers son vrai consommateur — Epic 5 — est
  intégralement tenue.

**La promesse vit en quatre endroits**, jamais dans une note de passage : **issue GitHub #34**
(les critères d'acceptation hérités par Epic 11), le record `epic-4-correct-course-2026-07-25.md`,
les notes datées dans `epics.md` (clôture d'Epic 4 + bloc « hérité » d'Epic 11), et
`sprint-status.yaml` + les marqueurs `CONSUMER PENDING: Epic 11 (issue #34)` sur les deux
entrées du MANIFEST — **le verrou du corpus déclare lui-même que ces artefacts n'ont pas encore
de lecteur.**

**La leçon, de John, pour le prochain découpage :** *une story appartient à l'epic de son
CONSOMMATEUR, pas à l'epic de son thème.* 4.18/4.19 étaient rangées sous Epic 4 parce que
« pièges » était le thème ; leur seul consommateur a toujours été Epic 11. Le planning le savait
à moitié (« authored here but only become executable in Epic 11 ») — la clôture finit l'aveu.

**À discuter — c'est le point où j'aimerais le plus ton avis :** fermer un epic dont deux
stories ne tournent pas est un jugement assumé. L'alternative (garder l'epic ouvert pendant
qu'Epics 5–10 avancent) me semblait pire, mais c'est ta comptabilité.

---

## Décision 4 — Deuxième amendement de vie privée : le hostname honnêtement vide (4.17)

Même rituel que la décision 1, même honnêteté : la mesure dit que la source produit hostname
**MISSING et EMPTY, jamais null**, donc le corpus doit pouvoir committer un nom vide — que la
règle `doit commencer par "doc-"` refusait. Un prédicat, un rouge naturel enregistré, un test de
borne (`"printer-salon"` rougit toujours), et les noms composés d'espaces restent refusés
délibérément : la mesure enregistre `""`, pas du remplissage.

Le point subtil de cette famille mérite ton œil : le piège `must-abstain` sur la paire
vide-vs-absent asserte que **le vide COMPTE COMME absent**. Mais — et je l'ai écrit dans
l'en-tête plutôt que de le taire — **la gate ne compare jamais les causes**, seulement les
règles : un moteur qui abstient pour la mauvaise raison passe la gate. La cause est de la
**vérité enregistrée**, pas de la mécanique de porte. C'est la phrase la plus faible et vraie.

---

## Décision 5 — La dette FR36, soldée

La décision Paquet B (party mode, toi arbitre) avait rétréci FR36 à un MVP partiel ; `epics.md`
le portait depuis toujours, **le PRD seul promettait encore le tableau de bord complet**. J'ai
réconcilié FR36, l'item 11 de la liste de périmètre MVP et le bullet Domain-Specific, avec une
entrée datée dans `editHistory`. **Non touché délibérément** : le récit du Journey 5, qui montre
un tableau de bord avec un chiffre de couverture — un journey raconte la vie du produit (Growth
compris), il n'est pas un engagement de périmètre ; le périmètre vit dans les FR et les listes,
qui concordent désormais.

---

## Décision 6 — Une revue à une seule couche, dite plutôt que maquillée

La story 4.18 n'a reçu **qu'une couche de revue** : l'Acceptance Auditor est allé au bout
(**PASS 7/7**, rouge naturel rejoué, deux sha256 recalculés), mais la couche Blind/Edge a été
**coupée en cours par une limite de dépense API** et n'a produit aucun constat. C'est
enregistré dans la story et dans la PR. Les six autres stories ont eu leurs trois couches.

**À discuter :** faut-il repasser une couche adversariale sur 4.18 avant qu'Epic 11 s'appuie
dessus ? Mon avis : oui, mais au moment où Epic 11 la consommera — l'artefact ne bougera pas
d'ici là et le parseur donnera un meilleur contexte de lecture.

---

## Ce que la validation a rattrapé (et pourquoi elle reste rentable)

Deux agents à contexte frais par story, **avant** le dev. Les prises significatives :

- **4.14** — *deux HIGH.* Le texte des fichiers de pièges **n'atteint aucun scanner de vie
  privée** (unique site d'appel = la marche des `Record::Failure`) : la story revendiquait une
  couverture inexistante. Et la géométrie de l'uplink était fausse : même switch/ports
  différents est la forme qui **CONCORDE** selon le must-merge de multi-nic — il a fallu faire
  traverser le failover vers le second switch pour que la contradiction soit réelle.
- **4.16** — *un HIGH.* Le must-not-merge est le premier du corpus dont **les deux membres
  appartiennent à un seul device L2** ; sans une phrase de portée explicite dans l'en-tête, la
  famille se lit comme auto-contradictoire le jour où Epic 5 groupera les deux veths. La revue
  a ensuite élargi le doc de `Expectation::MustNotMerge` (qui disait « different devices ») à la
  vérité : *le refus est porté par le NIVEAU de la règle nommée*.
- **4.17** — *un HIGH.* Deux MACs non épinglées auraient laissé la paire d'abstention
  s'effondrer silencieusement en paire exact-MAC.
- **4.18** — *deux HIGH.* L'enveloppe `meta`/`rc`/`data` et la clé `ip` étaient des **croyances
  présentées comme mesure** — la story violait sa propre ligne rouge sur ses octets les plus
  externes ; elles sont devenues des trous nommés. Et deux README parents seraient devenus faux.

**Les revues, elles, ont surtout produit des patchs de véracité** : des doc-comments qui
promettaient plus que le test ne prouvait, des valeurs citées par une raison mais épinglées par
personne. La leçon maison tient toujours : *nommer le test derrière chaque affirmation, ou
écrire la phrase plus faible et vraie.*

---

## Deux dettes enregistrées, et une tâche due

**Dans `deferred-work.md`** (quatre entrées ajoutées ce run) : le texte des fichiers de pièges
n'est scanné par rien ; les évasions nommées du scanner de texte ; le champ `raw` des
observations n'est inspecté par personne ; les byte-pins de 4.13/4.14 n'épinglent pas la liaison
`obs_id` ↔ ligne (4.15 l'a corrigée chez elle, les sœurs attendent).

**⚠️ La tâche due, et c'est la seule :** **régénérer `architecture-views.md`**. Son
`sourceSha256` ne correspond plus depuis le commit `da23f9f` — **antérieur à Epic 4** : la
péremption est héritée, pas causée par ce run. Mais la fin d'Epic 4 est le *milestone* que le
projet lui-même a désigné pour cette régénération. C'est une dérivation d'environ 880 lignes à
partir des 5123 lignes de `architecture.md` : elle mérite sa propre session à contexte frais, et
je ne l'ai pas tentée en fin de run.

**Une observation d'environnement, deux fois rencontrée :** sur cet arbre synchronisé par
Synology Drive, un `cargo test` local peut exécuter un **binaire périmé** ou voir un corpus
transitoirement remplacé par un état serveur ancien (une couche de revue a observé 8 échecs sur
5 exécutions, sha256 et `git status` identiques, puis 15+ exécutions vertes). La CI, sur
checkout propre, n'est pas concernée. Réflexe adopté : `touch` les fichiers édités avant de
croire un résultat impossible.

---

## Ce qui vient ensuite

1. **Cette discussion**, puis la **rétrospective d'Epic 4** ensemble.
2. La régénération d'`architecture-views.md` (session dédiée).
3. **Epic 5 — le moteur d'identité.** Rien ne le bloque : le corpus, le harnais de métriques et
   la table de vérité l'attendent, tous écrits **avant** lui. C'était toute la thèse de cet
   epic : *« une métrique écrite après le moteur est pliée pour épouser le moteur. »*
