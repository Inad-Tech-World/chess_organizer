# Messages de l'interface. Les titres et colonnes partagés avec les listes
# imprimées sont dans lists.ftl.

## Général

untitled = Tournoi sans titre
new-tournament = Nouveau tournoi
action-open = Ouvrir…
action-import-trf = Importer un TRF…
action-export-trf = Exporter en TRF…
action-create = Créer
action-cancel = Annuler
action-close = Fermer
action-save = Enregistrer
action-edit = Modifier
action-remove = Supprimer
action-clear = Effacer
action-undo = Annuler
action-redo = Rétablir
action-dismiss = Fermer
action-move-up = Monter
action-move-down = Descendre
undo-what = Annuler { $what }
planned-badge = Phase { $phase }
no-tournament-hint = Ouvrez ou créez d'abord un tournoi.
export-done = { $path } enregistré
import-warnings = Importé avec des remarques :
import-new-window-hint = Fermez d'abord ce tournoi, ou utilisez Fichier › Ouvrir dans une nouvelle fenêtre.
start-tagline = Gestion gratuite des tournois suisses, toutes rondes et par équipes — hors ligne, sur votre ordinateur.
start-recent = Tournois récents
status-rounds = Ronde { $paired } sur { $total }
status-withdrawn = Retiré à partir de la ronde { $round }
status-late = Entre à la ronde { $round }

## Navigation

nav-label = Sections
nav-overview = Vue d'ensemble
nav-setup = Paramètres du tournoi
nav-players = Joueurs
nav-rounds = Appariements et résultats
nav-lists = Listes
nav-teams = Équipes
nav-prizes = Prix
nav-ratings = Elo et normes
nav-publish = Publier
nav-player-db = Base de joueurs
nav-settings = Préférences

## Vue d'ensemble

overview-type = Type de tournoi
overview-players = Joueurs
overview-active = { $count } actifs
overview-rounds = Rondes appariées
overview-pending = Résultats manquants
overview-next = Étape suivante
overview-add-players = Ajouter des joueurs
overview-enter-results = Saisir les résultats de la ronde { $round }
overview-pair = Apparier la ronde { $round }
overview-final = Classement final

## Types de tournoi

kind-swiss-dutch = Suisse – système hollandais FIDE
kind-swiss-dubov = Suisse – système Dubov FIDE
kind-swiss-burstein = Suisse – système Burstein FIDE
kind-swiss-lim = Suisse – système Lim FIDE
kind-swiss-monrad = Suisse – simple (Monrad)
kind-rr = Toutes rondes
kind-rr-double = Toutes rondes aller-retour
kind-team-swiss = Suisse par équipes
kind-team-rr = Toutes rondes par équipes
kind-team-rr-double = Toutes rondes par équipes aller-retour

## Paramètres

info-name = Nom du tournoi
info-name-placeholder = p. ex. Open de Kampala 2026
setup-info = Informations sur le tournoi
setup-venue = Lieu
setup-start-date = Date de début
setup-end-date = Date de fin
setup-deputies = Arbitres adjoints (séparés par des virgules)
setup-organizer = Organisateur
setup-system-title = Système et rondes
setup-system = Type de tournoi
setup-rounds = Nombre de rondes
setup-rating-type = Type de classement
setup-initial-color = Couleur de la tête de série à la table 1, ronde 1
setup-acceleration = Accélération
setup-acceleration-none = Aucune
setup-acceleration-baku = Accélération de Bakou
setup-acceleration-manual = Manuelle (depuis le TRF)
setup-fide-rated = Tournoi homologué FIDE
setup-kind-locked = Le type de tournoi ne peut plus changer une fois la ronde 1 appariée.
setup-points = Points
points-win = Gain
points-draw = Nulle
points-loss = Perte
points-forfeit-win = Gain par forfait
points-forfeit-loss = Perte par forfait
points-pab = Exempt attribué
points-half-bye = Bye d'un demi-point
points-full-bye = Bye d'un point
points-zero-bye = Bye à zéro / absent
setup-tiebreaks = Départages (FIDE C.07)
setup-tiebreak = Départage
setup-tb-cut = Coupe / médian
setup-tb-forfeits = Forfaits comptés comme joués (/P)
setup-tb-add = Ajouter un départage
setup-tb-help = Les codes suivent les descripteurs d'ordre TRF-2026 (p. ex. BH/C1 = Buchholz moins le plus faible). Un « – » signifie que la valeur ne peut pas être calculée pour ce joueur, p. ex. un départage Elo sans adversaire classé.
setup-forbidden = Appariements interdits
setup-forbidden-help = Les joueurs d'un même groupe ne sont jamais appariés entre eux : membres d'un même club dans les premières rondes, membres d'une famille, ou toute paire interdite par le règlement.
setup-group-name = Nom du groupe
setup-group-ranks = Numéros de départ, p. ex. 3, 17, 24
setup-group-add = Ajouter un groupe
setup-group-default = Groupe
setup-group-too-small = Un groupe doit compter au moins deux joueurs (par numéro de départ).
setup-team = Paramètres des équipes
setup-boards = Échiquiers par match
setup-reserves = Remplaçants
setup-match-win = Points de match pour une victoire
setup-match-draw = Points de match pour un nul
setup-match-loss = Points de match pour une défaite
setup-primary-score = Classement par
setup-color-preference = Préférences de couleur
score-match-points = Points de match
score-game-points = Points de partie
color-pref-a = Type A
color-pref-b = Type B
color-pref-none = Aucune
color-white = Blancs
color-black = Noirs
setup-saved = Paramètres enregistrés.

## Noms des départages

tb-DE = Confrontation directe
tb-BPG = Parties jouées avec les Noirs
tb-BWG = Parties gagnées avec les Noirs
tb-REP = Rondes choisies pour jouer
tb-SB = Sonneborn-Berger
tb-ARO = Elo moyen des adversaires
tb-TPR = Performance
tb-PTP = Performance parfaite
tb-APRO = Performance moyenne des adversaires
tb-APPO = Performance parfaite moyenne des adversaires
tb-WIN = Nombre de victoires
tb-WON = Parties gagnées sur l'échiquier
tb-PS = Score progressif (cumulatif)
tb-BH = Buchholz
tb-AOB = Moyenne des Buchholz des adversaires
tb-FB = Buchholz anticipé
tb-KS = Système Koya
tb-BC = Décompte par échiquier
tb-TBR = Résultat au premier échiquier
tb-BBE = Élimination du dernier échiquier
tb-MPvGP = Points de match contre points de partie
tb-EMMSB = Sonneborn-Berger par équipes (PM × PM)
tb-EMGSB = Sonneborn-Berger par équipes (PM × PP)
tb-EGMSB = Sonneborn-Berger par équipes (PP × PM)
tb-EGGSB = Sonneborn-Berger par équipes (PP × PP)
tb-EDE = Confrontation directe étendue
tb-EDEBT = CDE + échiquiers + premier échiquier
tb-EDEBB = CDE + échiquiers + dernier échiquier
tb-EDET = CDE + premier échiquier
tb-EDEB = CDE + dernier échiquier
tb-SSSC = Score compensé par la force
tb-OTHER_SEC = Score secondaire

## Joueurs

col-status = Statut
player-name = Nom (Nom, Prénom)
player-name-placeholder = Nom, Prénom
player-national-id = ID national
player-k-factor = Coefficient K
sex-male = Homme
sex-female = Femme
rating-standard = Standard
rating-rapid = Rapide
rating-blitz = Blitz
rating-national = National
players-count = { $count ->
    [one] 1 joueur
   *[other] { $count } joueurs
}
players-add = Ajouter un joueur
players-add-another = Ajouter et continuer
players-edit = Modifier le joueur
players-empty = Aucun joueur pour l'instant. Ajoutez-les un par un, depuis la base de joueurs, ou importez un fichier TRF.
players-hint = Entrée modifie · Insert ajoute · Suppr retire un joueur qui n'a pas encore joué.
players-remove-confirm = Retirer { $name } du tournoi ?
players-withdraw = Retirer
players-withdraw-from = Non apparié à partir de la ronde
players-reenter = Réintégrer
players-byes = Byes et pauses…
players-byes-title = Byes de { $name }
players-byes-help = Les byes demandés sont respectés lors de l'appariement de la ronde. Un bye à zéro sert aussi de pause.
players-sort = Trier la liste de départ
players-late-entry = Inscription tardive : entre à la ronde { $round }. Les rondes précédentes comptent comme
bye-none = Joue
bye-half-points = Bye d'un demi-point
bye-zero-points = Bye à zéro (absent)
bye-full-points = Bye d'un point

## Base de joueurs

player-db-help = Téléchargez la liste Elo FIDE mensuelle (ou importez une liste enregistrée, en zip, XML ou TXT) et ajoutez des listes nationales en CSV. Les recherches remplissent ensuite noms, titres, classements et identifiants.
player-db-download-fide = Télécharger la liste FIDE
player-db-connecting = Connexion à ratings.fide.com…
player-db-downloading = Téléchargement de la liste FIDE… { $mb } Mo
player-db-importing = Importation… { $count } joueurs
player-db-imported = { $count } joueurs importés.
player-db-mapping-title = Importer une liste nationale
player-db-mapping-help = Choisissez la colonne de chaque champ. Une colonne de nom complet ou une colonne de nom de famille est obligatoire.
player-db-source-name = Nom de la liste
player-db-has-header = La première ligne contient les noms de colonnes
player-db-field-name = Nom complet
player-db-field-surname = Nom de famille
player-db-field-first-name = Prénom
player-db-field-national-id = ID national
player-db-field-rating = Classement national
player-db-import = Importer
player-db-import-fide = Importer la liste Elo FIDE…
player-db-import-national = Importer une liste nationale (CSV)…
player-db-search-placeholder = Rechercher par nom ou ID FIDE
player-db-no-results = Aucun joueur trouvé.
player-db-type-to-search = Tapez au moins deux lettres pour rechercher.

## Rondes

round-n = Ronde { $round }
round-short = R{ $round }
rounds-next-tab = R{ $round } – suivante
rounds-results-entered = { $entered } résultats saisis sur { $total }
rounds-swap = Inverser les couleurs
rounds-check = Vérifier les appariements
rounds-check-ok = Ronde { $round } : aucun problème de règle.
rounds-check-problems = La ronde { $round } enfreint ces règles :
rounds-print = Imprimer les appariements
rounds-delete = Supprimer la ronde
rounds-delete-confirm = Supprimer la ronde { $round } et tous ses résultats ? Vous pourrez annuler.
rounds-keys-hint = Tapez 1, 0 ou 5 (=) pour 1-0, 0-1, ½-½ · + / - pour les forfaits · * double forfait · Suppr efface. Le curseur passe à l'échiquier suivant.
rounds-pair-title = Apparier la ronde { $round }
rounds-auto-help = Appariement automatique ({ $system }), ou manuel ci-dessous.
rounds-pair-auto = Apparier automatiquement
rounds-manual = Apparier manuellement
rounds-manual-help = Cliquez sur deux joueurs pour les placer à un échiquier (le premier a les Blancs). Les joueurs sont classés par score.
rounds-give-pab = Donner l'exempt à { $name }
rounds-requested-byes = Byes demandés pour cette ronde
rounds-boards = { $count ->
    [one] 1 échiquier
   *[other] { $count } échiquiers
}
rounds-commit = Apparier la ronde { $round }
rounds-unpaired-left = { $count } joueur(s) encore non apparié(s).
rounds-none = Aucune ronde à afficher.
rounds-matches = { $count } matchs
rounds-lineups = Compositions…
rounds-lineups-title = Compositions
rounds-lineups-help = Les échiquiers suivent l'ordre choisi ici. Les résultats déjà saisis sont conservés pour les échiquiers dont les joueurs n'ont pas changé.
rounds-team-keys-hint = Sélectionnez un échiquier et tapez 1, 0 ou 5 (=) pour 1-0, 0-1, ½-½ · + / - forfaits · Suppr efface. Les lignes de match montrent le score en cours.
rounds-violations-title = Ces appariements enfreignent les règles
rounds-violations-help = Chess Organiser a trouvé ces problèmes. Corrigez-les, ou appariez quand même si l'arbitre l'a décidé.
rounds-violations-fix = Revenir et corriger
rounds-violations-force = Apparier quand même

## Listes

list-name-pairings = Appariements
list-name-standings = Classement
list-name-crosstable = Grille américaine
list-name-team-pairings = Appariements par équipes
list-name-team-standings = Classement par équipes
lists-language = Langue
lists-print = Imprimer / PDF…
lists-format = Format d'export
lists-export = Exporter…
lists-round = Ronde
lists-no-rounds = Aucune ronde n'a encore été appariée.
lists-print-hint = Imprimer / PDF ouvre la liste dans votre navigateur ; utilisez sa boîte de dialogue d'impression pour imprimer ou enregistrer en PDF.

## Équipes et prix

teams-add = Ajouter une équipe
teams-edit = Modifier l'équipe
teams-name = Nom de l'équipe
teams-captain = Capitaine
teams-members = Membres
teams-board-order = Ordre des échiquiers (dans l'ordre coché)
teams-empty = Aucune équipe pour l'instant.
teams-individual-note = Ceci est un tournoi individuel ; les équipes ne servent qu'aux prix par équipes.
teams-remove-confirm = Supprimer l'équipe { $name } ?
teams-standings = Classement par équipes après la ronde { $round }
prizes-add = Ajouter une catégorie
prizes-category = Catégorie
prizes-filter = Qui est éligible
prizes-prizes = Prix
prizes-prizes-help = Prix, un par ligne, dans l'ordre des places
prizes-empty = Aucune catégorie de prix pour l'instant.
prizes-max-rating = Elo maximum
prizes-min-rating = Elo minimum
prizes-born-from-label = Né(e) en ou après (jeunes)
prizes-born-until-label = Né(e) en ou avant (vétérans)
prizes-born-from = né(e) ≥ { $year }
prizes-born-until = né(e) ≤ { $year }
prizes-unrated = Non classés uniquement
prizes-women = Femmes
prizes-winners = Lauréats après la ronde { $round }
prizes-one-each = Chaque joueur reçoit au plus un prix, dans l'ordre des catégories.

## Elo et publication

ratings-trf-title = Rapport Elo FIDE
ratings-trf-help = Exportez le tournoi en fichier TRF-2026 pour le responsable Elo de la fédération.
ratings-changes-title = Variations Elo
ratings-changes-help = Scores attendus, coefficients K et variation Elo de chaque joueur classé.
ratings-calculate = Calculer les variations Elo
ratings-norms-title = Normes de titre
ratings-check-norms = Vérifier les normes
ratings-norms-help = Vérifie les exigences FIDE B.01 pour une norme de GM, MI, GMF ou MIF sur les rondes jouées : parties, adversaires classés et titrés, fédérations et performance.
ratings-norms-none = Aucune norme ne s'applique à ce joueur.
ratings-norm-achieved = norme réalisée
ratings-norm-missed = non réalisée
ratings-norm-stats = { $games } parties · Elo moyen des adversaires { $aro } · performance { $perf }
ratings-first-rating = premier classement { $rating }
publish-help = Envoyez appariements et classements vers un site de résultats pour que les joueurs suivent le tournoi sur leur téléphone. Tout continue de fonctionner hors ligne ; les envois se font quand une connexion est disponible.
publish-server = Serveur de résultats
publish-token = Jeton d'envoi
publish-upload = Envoyer maintenant
publish-done = Envoyé. Les joueurs peuvent suivre le tournoi en ligne.
publish-live-at = En ligne sur
publish-server-title = Héberger un serveur de résultats
publish-server-help = Le programme co-publish sert les tournois envoyés. Lancez-le sur n'importe quel serveur derrière HTTPS avec un jeton d'envoi ; l'adresse et le jeton vont dans les champs ci-dessus.

## Préférences et raccourcis

settings-language = Langue
settings-theme = Apparence
settings-translations-note = Les listes imprimées peuvent utiliser une autre langue que le programme : choisissez-la dans l'écran Listes. Tout texte non encore traduit apparaît en anglais.
settings-updates = Mises à jour
settings-check-updates = Rechercher des mises à jour
settings-install-update = Installer la version { $version } et redémarrer
settings-updates-disabled = Cette version ({ $version }) ne se met pas à jour seule ; téléchargez les nouvelles versions sur le site du projet.
settings-up-to-date = Vous avez la dernière version ({ $version }).
theme-system = Comme le système
theme-light = Clair
theme-dark = Sombre
shortcuts-title = Raccourcis clavier
shortcut-white-wins = Les Blancs gagnent (1-0)
shortcut-black-wins = Les Noirs gagnent (0-1)
shortcut-draw = Nulle (½-½)
shortcut-white-forfeit = Les Blancs gagnent par forfait (+/-)
shortcut-black-forfeit = Les Noirs gagnent par forfait (-/+)
shortcut-double-forfeit = Double forfait (-/-)
shortcut-clear-result = Effacer le résultat
shortcut-move = Passer d'une ligne à l'autre
shortcut-edit = Modifier la ligne sélectionnée
shortcut-undo = Annuler la dernière modification
shortcut-redo = Rétablir
shortcut-pair = Apparier la ronde suivante
shortcut-sections = Aller à une section

## Écran d'accueil

welcome-sample-title = Nouveau ? Découvrez avec des données d'exemple
welcome-sample-body = Ouvre un open rapide de 25 joueurs avec quatre rondes jouées (la dernière à moitié saisie), une inscription tardive, un retrait, des byes, des catégories de prix et une base de joueurs d'exemple. Tout est fictif ; essayez tout, annulez tout.
welcome-sample-load = Ouvrir le tournoi d'exemple
welcome-tour = Visite du programme
welcome-features = Ce que fait Chess Organiser
welcome-free = gratuit pour toutes les fédérations et tous les clubs
welcome-shot-rounds = Appariements et résultats
welcome-shot-rounds-body = Saisissez les résultats au clavier : 1, 0 ou 5 à chaque échiquier, + et − pour les forfaits. Le curseur passe à l'échiquier suivant et tout changement peut être annulé.
welcome-shot-players = Joueurs
welcome-shot-players-body = Inscrivez les joueurs à la main ou depuis votre liste Elo, avec titres, fédérations et classements. Inscriptions tardives, retraits, byes demandés et pauses sont gérés ronde par ronde.
welcome-shot-lists = Listes
welcome-shot-lists-body = Liste de départ, appariements et classements, prêts à imprimer, enregistrer en PDF ou exporter, dans la langue de votre choix.
welcome-shot-overview = Vue d'ensemble
welcome-shot-overview-body = Voyez d'un coup d'œil où en est le tournoi et ce qu'il reste à faire.
feat-files = Un fichier par tournoi
feat-files-body = Enregistré après chaque modification et sûr en cas de coupure de courant. Fonctionne hors ligne ; copiez-le ou envoyez-le comme n'importe quel document. Annulation illimitée avec historique complet.
feat-players = Inscriptions
feat-players-body = Joueurs depuis la liste FIDE ou nationale, inscriptions tardives avec byes à zéro ou à un demi-point, retraits, pauses et ordre de départ FIDE.
feat-results = Saisie rapide des résultats
feat-results-body = Saisie au clavier, corrections dans les rondes précédentes et appariement manuel avec vérification des répétitions, des membres d'un même club et des byes.
feat-lists = Listes et impression
feat-lists-body = Appariements, classements, grilles américaines, fiches individuelles, fiches de table et palmarès à imprimer, enregistrer en PDF ou exporter en HTML, CSV et Excel.
feat-trf = Rapports Elo FIDE
feat-trf-body = Export et import des fichiers TRF-2026 pour le responsable Elo et pour passer d'un programme à l'autre.
feat-pairing = Appariements FIDE automatiques
feat-pairing-body = Systèmes hollandais et Burstein vérifiés contre le moteur homologué par la FIDE, accélération de Bakou, Dubov, Lim, Monrad et toutes rondes avec tables de Berger.
feat-teams = Équipes et prix
feat-teams-body = Suisse et toutes rondes par équipes avec compositions et départages par équipes ; catégories de prix pour les femmes, les jeunes et les tranches Elo.
feat-ratings = Elo et normes
feat-ratings-body = Tous les départages FIDE, variations Elo selon FIDE B.02 et vérification des normes de GM, MI, GMF et MIF.
feat-languages = Langues et résultats en direct
feat-languages-body = Français, arabe, swahili, portugais et espagnol, et un site de résultats en direct pour les téléphones des joueurs.
sample-replace-confirm = Fermer le tournoi actuel et ouvrir une nouvelle copie du tournoi d'exemple ? Votre tournoi est enregistré et reste dans les tournois récents.

setup-note-unverified = Les appariements suivent les règles de ce système avec les mêmes critères absolus que le moteur hollandais. Contrairement aux moteurs hollandais et Burstein, il n'existe pas de programme de référence homologué pour le vérifier : relisez les appariements avant de les publier.
setup-note-monrad = Le Monrad apparie le premier joueur non apparié avec le suivant qu'il n'a pas encore rencontré. Il est destiné aux tournois de club, pas aux tournois homologués FIDE.
setup-note-team-swiss = Les équipes sont appariées selon les critères hollandais sur les points de match (ou de partie), la couleur de l'équipe étant celle du premier échiquier.
