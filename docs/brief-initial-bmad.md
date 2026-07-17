# Brief initial — Application de gestion d'infrastructure réseau pour PME et particuliers

> **Document d'entrée pour BMAD v6 — Phase 1 (Analysis / Product Brief)**
> Ce document rassemble le contexte, la vision, les contraintes et les décisions préliminaires
> issues des sessions de cadrage. Il est destiné à l'agent Analyst de BMAD, qui doit le
> challenger, l'approfondir et produire le Product Brief formel. Rien ici n'est figé sauf
> mention explicite « décision ferme ».

---

## 1. Vision et problème

### 1.1 Problème constaté

Il existe un vide entre :

- **Les outils professionnels** (NetBox, Nautobot, phpIPAM) : puissants mais surdimensionnés,
  complexes à déployer et à maintenir pour une PME sans équipe IT dédiée ou un particulier
  avec un homelab.
- **L'absence d'outil** : fichier Excel, schéma draw.io jamais à jour, ou simplement la
  mémoire de « la personne qui sait ».

Conséquences typiques : adresses IP en conflit, appareils inconnus non détectés, impossibilité
de savoir quelle application dépend de quelle base de données lors d'une panne ou d'une
migration, documentation obsolète dès sa création.

### 1.2 Vision produit

Une application **auto-hébergée, légère, en un seul binaire**, qui combine trois vues
habituellement séparées :

1. **IPAM** (gestion des adresses IP) avec découverte automatique.
2. **CMDB applicative légère** : applications composées de plusieurs logiciels, avec leurs
   relations (dépendances, connexions).
3. **Topologie réseau** : quel équipement est connecté où (port switch, SSID WiFi, VLAN).

Le principe directeur : **comparer en permanence l'état observé (découvert automatiquement)
et l'état déclaré (documenté par l'utilisateur)**. La valeur du produit naît de cet écart :
appareil inconnu apparu, IP documentée jamais vue depuis 30 jours, conflit d'adresse.

### 1.3 Public cible

- **PME** (5–100 employés) avec un réseau « complexe pour sa taille » : plusieurs VLANs,
  serveurs auto-hébergés, NAS, équipement réseau managé — mais sans ingénieur réseau à
  plein temps.
- **Particuliers avancés / homelabs** : Docker/Proxmox, NAS Synology, équipement UniFi ou
  similaire, dizaines d'appareils.

Anti-cible explicite : les grandes entreprises (NetBox existe pour elles), les foyers avec
une box opérateur et cinq appareils.

---

## 2. Périmètre fonctionnel

### 2.1 MVP (périmètre v1)

**F1 — Inventaire des équipements**
- CRUD équipements : nom, type (serveur, NAS, switch, AP, imprimante, IoT, VM, conteneur…),
  fabricant, modèle, emplacement physique, notes.
- Interfaces réseau multiples par équipement (Ethernet, WiFi, virtuelle), chacune avec MAC.

**F2 — IPAM**
- Gestion des sous-réseaux (CIDR), VLANs, plages DHCP, réservations.
- Attribution d'adresses IP aux interfaces (statique documentée, DHCP observée).
- Vue d'occupation par sous-réseau (utilisées / libres / documentées / observées).

**F3 — Découverte automatique**
- **Connecteur UniFi (première classe, décision ferme)** : interrogation périodique de
  l'API locale du contrôleur (a UniFi gateway / UniFi OS, API key). Remonte : clients connectés
  (MAC, IP, hostname, fabricant), mode de connexion (port switch exact pour le filaire,
  AP + SSID pour le WiFi), VLANs, SSIDs configurés, baux DHCP.
- **Scanner générique (fallback)** : ARP scan sur segment local + ping sweep pour réseaux
  routés. Enrichissement hostname via mDNS/reverse DNS. Implémentation MVP : appel à
  `nmap -sn` accepté ; implémentation native Rust (`pnet`) en option ultérieure.
- Planification des scans (intervalle configurable) + scan manuel à la demande.

**F4 — Réconciliation observé / déclaré (cœur du produit)**
- Matching automatique par adresse MAC (clé primaire), avec heuristiques secondaires
  (hostname, historique IP/DHCP).
- **Inbox de tri** : tout appareil découvert non rapproché atterrit dans une file « à
  trier » où l'utilisateur peut : créer un nouvel équipement, rattacher à un équipement
  existant, ou ignorer (avec mémoire de l'ignore).
- Gestion des cas pénibles : randomisation MAC (téléphones), équipements multi-interfaces,
  VMs/conteneurs partageant du matériel.
- Horodatage systématique : première vue, dernière vue, historique des IP par MAC.

**F5 — Applications et logiciels**
- **Logiciel** : instance d'un programme sur un équipement (ex. PostgreSQL sur le NAS,
  Paperless dans un conteneur). Attributs : nom, version, port(s) d'écoute, équipement hôte.
- **Application** : groupe logique de logiciels (ex. « GED » = Paperless + PostgreSQL +
  Redis). Une application a un responsable, une criticité, des notes.
- **Relations typées entre logiciels** : `se_connecte_à` (Paperless → PostgreSQL),
  `dépend_de`, `héberge` (équipement → logiciel), `expose` (reverse proxy → service).
- Vue d'impact : « si cet équipement tombe, quelles applications sont affectées ? »

**F6 — Topologie réseau (saisie assistée)**
- Pour le parc UniFi : topologie **remontée automatiquement** par le connecteur
  (client ↔ port switch, client ↔ AP/SSID).
- Pour le reste : saisie manuelle des liens (équipement A port X ↔ équipement B port Y).
- Gestion des SSIDs, VLANs et de leur association.
- Représentation en **liste/table structurée** au MVP ; le schéma graphique interactif
  est explicitement reporté en v2 (voir §2.2).

**F7 — Alertes et écarts (version minimale)**
- Nouvel appareil inconnu détecté (avec VLAN/segment).
- IP documentée non vue depuis N jours.
- Conflit potentiel (même IP vue sur deux MAC).
- Notification : au MVP, affichage dans l'application + éventuel webhook générique.

### 2.2 Explicitement hors MVP (v2+)

- Schéma réseau **graphique interactif** (diagramme visuel type carte).
- Découverte de topologie via SNMP/LLDP pour matériel non-UniFi.
- Connecteurs additionnels : Omada, Mikrotik, OPNsense/pfSense, Proxmox, Docker.
- Multi-utilisateurs avec rôles fins (MVP : un seul rôle admin, éventuellement un rôle
  lecture seule si peu coûteux).
- Notifications riches (mail, Telegram, ntfy).
- Import depuis NetBox/phpIPAM/CSV (un import CSV simple peut être discuté pour le MVP).
- Scan de ports/services (nmap -sV) pour suggérer les logiciels installés.
- API publique documentée (l'API existera de fait, sa stabilisation contractuelle est v2).

### 2.3 Questions ouvertes pour l'Analyst

À challenger/trancher pendant la phase Analysis :

1. L'import CSV initial (amorçage de l'inventaire) est-il MVP ou v2 ?
2. Faut-il un mode « découverte seule » (lecture pure, aucune documentation) comme porte
   d'entrée à faible friction ?
3. IPv6 : support complet, support partiel (documentation sans scan), ou v2 ?
4. Le rôle lecture seule est-il nécessaire au MVP pour la cible PME ?
5. Internationalisation : FR + EN dès le MVP, ou EN d'abord ?
6. Historisation : quelle profondeur d'historique (IP par MAC, présence) conserver et
   avec quelle politique de rétention/purge ?

---

## 3. Contraintes et décisions techniques

### 3.1 Décisions fermes

| Sujet | Décision | Justification |
|---|---|---|
| Langage | **Rust** | Performance, binaire unique, fiabilité, préférence du porteur de projet |
| BDD petites installs | **SQLite** | Zéro dépendance, idéal tests et particuliers |
| BDD grandes installs | **MySQL/MariaDB** | Le NAS Synology fournit MariaDB nativement, **incluse dans les backups automatiques Synology** (contrairement à PostgreSQL, non fournie/non sauvegardée nativement) |
| PostgreSQL | **Non supporté** (au moins au MVP) | Réduire la matrice de test ; la cible de déploiement prioritaire est Synology |
| Déploiement | **Conteneur Docker** (Container Manager Synology) + binaire natif possible | Cible NAS Synology prioritaire, mais pas exclusive |
| Connecteur UniFi | **Première classe dès le MVP** | Le porteur du projet et une part importante de la cible sont équipés UniFi ; l'API locale fournit gratuitement clients, ports, SSIDs, VLANs, baux DHCP, en pur HTTP sortant sans privilège réseau |

### 3.2 Orientations techniques à valider en phase Architecture

- **Accès données** : SeaORM (construit sur SQLx) pressenti pour l'abstraction de dialecte
  SQLite/MariaDB et les migrations multi-backends. Alternative : SQLx pur avec SQL portable
  discipliné. À trancher par l'Architect (critères : ergonomie migrations, coût des
  requêtes complexes type graphe de dépendances).
- **Serveur web** : Axum pressenti (standard de facto de l'écosystème).
- **Frontend** : deux options à évaluer par l'Architect :
  - HTMX + templates (Askama/Maud) : full-Rust, binaire unique, simplicité de déploiement
    maximale — bien adapté à un outil d'administration.
  - SPA (Svelte/Vue) servi en statique : nécessaire si le schéma graphique v2 exige de
    toute façon du JS riche — anticiper pour éviter une réécriture.
- **Tâches de fond** : scheduler interne (tokio) pour scans et polling UniFi ; pas de
  dépendance externe type cron/redis.
- **Scan ARP en conteneur** : nécessite `network_mode: host` ou capability `NET_RAW` ;
  à documenter clairement. Le connecteur UniFi, lui, fonctionne sans privilège — c'est
  la voie recommandée quand disponible.
- **Authentification** : locale simple (login/mot de passe, session) au MVP. Pas de SSO/LDAP.
- **Configuration** : fichier + variables d'environnement (12-factor pour le conteneur).

### 3.3 Environnement de référence du porteur de projet

- a UniFi gateway + parc 100 % UniFi (dont NanoStations en pont point-à-point), UniFi OS récent.
- NAS Synology avec MariaDB et Container Manager.
- Réseau domestique/petite structure multi-bâtiments (maison + garage reliés en pont 5 GHz),
  VLANs, services auto-hébergés (dont stack IA locale, Paperless envisageable).
- Cet environnement sert de **banc de test réel** pour le MVP.

---

## 4. Esquisse du modèle de données

> À raffiner par l'Architect. L'esquisse ci-dessous fixe le vocabulaire.

**Entités principales :**

- `Site` (optionnel MVP) — regroupement physique (bâtiment).
- `Device` — équipement. Type énuméré extensible. Un device a 0..n `Interface`.
- `Interface` — physique (Ethernet, WiFi) ou virtuelle (bridge, VLAN, veth). Porte la MAC.
- `Subnet` — CIDR, VLAN associé, description, plage DHCP.
- `Vlan` — id, nom.
- `IpAddress` — adresse, subnet, type d'attribution (statique déclarée / DHCP observée /
  réservation), rattachée à une Interface (déclaré) et/ou observée sur une MAC.
- `Ssid` — nom, VLAN associé, remonté par UniFi ou déclaré.
- `Link` — lien physique/logique : (Device A, port) ↔ (Device B, port), ou
  (Interface WiFi) ↔ (AP, SSID). Source : `declared` | `observed(unifi)`.
- `Software` — logiciel installé : nom, version, ports d'écoute, hébergé par un `Device`
  (relation `héberge`).
- `Application` — groupe de `Software`, criticité, responsable.
- `Relation` — arête typée entre Software/Application : `connects_to`, `depends_on`,
  `exposes`. Attributs : protocole, port, description.
- `Observation` — enregistrement horodaté issu d'une source (`unifi`, `arp_scan`) :
  MAC, IP, hostname, port switch / AP / SSID, first_seen, last_seen.
- `ReconciliationDecision` — mémoire des décisions de tri (rattachement, ignore).
- `Alert` — écart détecté, état (nouveau / vu / résolu / ignoré).

**Principe structurant (décision ferme)** : séparation systématique
**déclaré** (saisi par l'utilisateur) / **observé** (issu des sources, horodaté, jamais
modifié à la main). La réconciliation crée des liens entre les deux, elle ne fusionne pas.

---

## 5. Parcours utilisateur clés (à détailler en PRD)

1. **Premier démarrage** : assistant de configuration → connexion contrôleur UniFi (URL +
   API key) OU déclaration d'un sous-réseau à scanner → premier scan → inbox de tri
   pré-remplie → l'utilisateur nomme ses 10 appareils les plus importants. Objectif :
   valeur visible en < 15 minutes.
2. **Routine hebdomadaire** : consulter l'inbox (nouveaux appareils), traiter les alertes.
3. **Documentation d'une application** : créer « Paperless », y rattacher les logiciels
   (Paperless-ngx, PostgreSQL, Redis), déclarer les relations, voir la vue d'impact.
4. **Diagnostic** : « quelle est cette IP ? » → recherche → fiche complète (équipement,
   historique, port switch, applications hébergées).
5. **Préparation migration** : vue d'impact d'un équipement avant de l'éteindre.

---

## 6. Risques identifiés

| Risque | Gravité | Mitigation envisagée |
|---|---|---|
| Effet « trois produits en un » → MVP obèse | Élevée | Périmètre §2.1 strict ; schéma graphique explicitement v2 |
| Réconciliation MAC fragile (randomisation, VMs) | Élevée | Inbox de tri comme UX centrale ; heuristiques secondaires ; mémoire des décisions |
| API UniFi : variations selon versions UniFi OS / rupture d'API | Moyenne | Couche connecteur isolée derrière un trait Rust ; ciblage API officielle par clé d'API ; tests contre versions courantes |
| Dialectes SQLite vs MariaDB (types, migrations) | Moyenne | ORM avec support multi-backend ; CI testant les deux backends systématiquement |
| Scan ARP en conteneur (privilèges) | Faible | Documentation claire ; UniFi comme voie sans privilège |
| Adoption : « encore un outil à maintenir » | Moyenne | Binaire unique, config minimale, valeur en 15 min, mode découverte seule (question ouverte Q2) |

---

## 7. Critères de succès du MVP

- Installation sur Synology (Container Manager) en < 30 minutes, documentation comprise.
- Sur un réseau UniFi : inventaire complet observé (clients, ports, SSIDs) sans aucune
  saisie manuelle, en un cycle de polling.
- Sur un réseau non-UniFi : découverte des appareils actifs du segment local par scan.
- Un utilisateur peut documenter une application 3-tiers et obtenir sa vue d'impact.
- L'écart observé/déclaré est visible et actionnable (inbox + alertes).
- Les deux backends (SQLite, MariaDB) passent la même suite de tests d'intégration.

---

## 8. Instructions pour l'agent Analyst (BMAD)

1. Challenger la segmentation cible (PME vs particulier) : les besoins divergent-ils au
   point de nécessiter des modes distincts ?
2. Trancher ou faire trancher les questions ouvertes du §2.3.
3. Vérifier le périmètre MVP §2.1 : proposer des coupes si le time-to-first-release
   dépasse un horizon raisonnable pour un développeur solo assisté par IA.
4. Produire le Product Brief formel BMAD à partir de ce document, puis enchaîner sur la
   phase Planning (PRD) en conservant les décisions fermes du §3.1 comme contraintes
   non négociables.
5. Toute proposition de scope supplémentaire doit être explicitement validée par le
   porteur de projet (pas de dé-scoping ni d'ajout silencieux).
