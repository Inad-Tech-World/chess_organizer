# Mensajes de la interfaz. Los títulos y columnas compartidos con las listas
# impresas están en lists.ftl.

## General

untitled = Torneo sin título
new-tournament = Nuevo torneo
action-open = Abrir…
action-import-trf = Importar TRF…
action-export-trf = Exportar TRF…
action-create = Crear
action-cancel = Cancelar
action-close = Cerrar
action-save = Guardar
action-edit = Editar
action-remove = Quitar
action-clear = Borrar
action-undo = Deshacer
action-redo = Rehacer
action-dismiss = Descartar
action-move-up = Subir
action-move-down = Bajar
undo-what = Deshacer { $what }
planned-badge = Fase { $phase }
no-tournament-hint = Abra o cree primero un torneo.
export-done = Guardado { $path }
import-warnings = Importado con observaciones:
import-new-window-hint = Cierre primero este torneo, o use Archivo › Abrir en ventana nueva.
start-tagline = Gestión gratuita de torneos suizos, round robin y por equipos — sin conexión, en su propio ordenador.
start-recent = Torneos recientes
status-rounds = Ronda { $paired } de { $total }
status-withdrawn = Retirado desde la ronda { $round }
status-late = Se incorpora en la ronda { $round }

## Navegación

nav-label = Secciones
nav-overview = Resumen
nav-setup = Configuración del torneo
nav-players = Jugadores
nav-rounds = Emparejamientos y resultados
nav-lists = Listas
nav-teams = Equipos
nav-prizes = Premios
nav-ratings = Elo y normas
nav-publish = Publicar
nav-player-db = Base de jugadores
nav-settings = Preferencias

## Resumen

overview-type = Tipo de torneo
overview-players = Jugadores
overview-active = { $count } activos
overview-rounds = Rondas emparejadas
overview-pending = Resultados pendientes
overview-next = Siguiente paso
overview-add-players = Añadir jugadores
overview-enter-results = Introducir los resultados de la ronda { $round }
overview-pair = Emparejar la ronda { $round }
overview-final = Clasificación final

## Tipos de torneo

kind-swiss-dutch = Suizo – sistema holandés FIDE
kind-swiss-dubov = Suizo – sistema Dubov FIDE
kind-swiss-burstein = Suizo – sistema Burstein FIDE
kind-swiss-lim = Suizo – sistema Lim FIDE
kind-swiss-monrad = Suizo – simple (Monrad)
kind-rr = Round robin
kind-rr-double = Round robin a doble vuelta
kind-team-swiss = Suizo por equipos
kind-team-rr = Round robin por equipos
kind-team-rr-double = Round robin por equipos a doble vuelta

## Configuración

info-name = Nombre del torneo
info-name-placeholder = p. ej. Abierto de Kampala 2026
setup-info = Información del torneo
setup-venue = Sede
setup-start-date = Fecha de inicio
setup-end-date = Fecha de fin
setup-deputies = Árbitros adjuntos (separados por comas)
setup-organizer = Organizador
setup-system-title = Sistema y rondas
setup-system = Tipo de torneo
setup-rounds = Número de rondas
setup-rating-type = Tipo de Elo
setup-initial-color = Color del primer cabeza de serie en la mesa 1, ronda 1
setup-acceleration = Aceleración
setup-acceleration-none = Ninguna
setup-acceleration-baku = Aceleración de Bakú
setup-acceleration-manual = Manual (desde TRF)
setup-fide-rated = Torneo válido para Elo FIDE
setup-kind-locked = El tipo de torneo no puede cambiar una vez emparejada la ronda 1.
setup-points = Puntos
points-win = Victoria
points-draw = Tablas
points-loss = Derrota
points-forfeit-win = Victoria por incomparecencia
points-forfeit-loss = Derrota por incomparecencia
points-pab = Descanso asignado
points-half-bye = Descanso de medio punto
points-full-bye = Descanso de un punto
points-zero-bye = Descanso de cero puntos / ausente
setup-tiebreaks = Desempates (FIDE C.07)
setup-tiebreak = Desempate
setup-tb-cut = Corte / mediano
setup-tb-forfeits = Incomparecencias como jugadas (/P)
setup-tb-add = Añadir desempate
setup-tb-help = Los códigos siguen los descriptores de orden del TRF-2026 (p. ej. BH/C1 = Buchholz con corte 1). Un «–» significa que el valor no se puede calcular para ese jugador, p. ej. un desempate de Elo sin rivales con Elo.
setup-forbidden = Emparejamientos prohibidos
setup-forbidden-help = Los jugadores de un mismo grupo nunca se emparejan entre sí: compañeros de club en las primeras rondas, familiares o cualquier pareja que prohíban las bases.
setup-group-name = Nombre del grupo
setup-group-ranks = Números iniciales, p. ej. 3, 17, 24
setup-group-add = Añadir grupo
setup-group-default = Grupo
setup-group-too-small = Un grupo necesita al menos dos jugadores (por número inicial).
setup-team = Configuración de equipos
setup-boards = Tableros por encuentro
setup-reserves = Suplentes
setup-match-win = Puntos de match por victoria
setup-match-draw = Puntos de match por empate
setup-match-loss = Puntos de match por derrota
setup-primary-score = Clasificar por
setup-color-preference = Preferencias de color
score-match-points = Puntos de match
score-game-points = Puntos de partida
color-pref-a = Tipo A
color-pref-b = Tipo B
color-pref-none = Ninguna
color-white = Blancas
color-black = Negras
setup-saved = Configuración guardada.

## Nombres de los desempates

tb-DE = Encuentro directo
tb-BPG = Partidas jugadas con negras
tb-BWG = Partidas ganadas con negras
tb-REP = Rondas elegidas para jugar
tb-SB = Sonneborn-Berger
tb-ARO = Elo medio de los rivales
tb-TPR = Rendimiento
tb-PTP = Rendimiento perfecto
tb-APRO = Rendimiento medio de los rivales
tb-APPO = Rendimiento perfecto medio de los rivales
tb-WIN = Número de victorias
tb-WON = Partidas ganadas en el tablero
tb-PS = Puntuación progresiva
tb-BH = Buchholz
tb-AOB = Media del Buchholz de los rivales
tb-FB = Buchholz anticipado
tb-KS = Sistema Koya
tb-BC = Recuento por tableros
tb-TBR = Resultado del primer tablero
tb-BBE = Eliminación del último tablero
tb-MPvGP = Puntos de match frente a puntos de partida
tb-EMMSB = Sonneborn-Berger por equipos (PM × PM)
tb-EMGSB = Sonneborn-Berger por equipos (PM × PP)
tb-EGMSB = Sonneborn-Berger por equipos (PP × PM)
tb-EGGSB = Sonneborn-Berger por equipos (PP × PP)
tb-EDE = Encuentro directo ampliado
tb-EDEBT = EDA + tableros + primer tablero
tb-EDEBB = EDA + tableros + último tablero
tb-EDET = EDA + primer tablero
tb-EDEB = EDA + último tablero
tb-SSSC = Puntuación compensada por fuerza
tb-OTHER_SEC = Puntuación secundaria

## Jugadores

col-status = Estado
player-name = Nombre (Apellido, Nombre)
player-name-placeholder = Apellido, Nombre
player-national-id = ID nacional
player-k-factor = Factor K
sex-male = Hombre
sex-female = Mujer
rating-standard = Estándar
rating-rapid = Rápido
rating-blitz = Blitz
rating-national = Nacional
players-count = { $count ->
    [one] 1 jugador
   *[other] { $count } jugadores
}
players-add = Añadir jugador
players-add-another = Añadir y continuar
players-edit = Editar jugador
players-empty = Aún no hay jugadores. Añádalos uno a uno, desde la base de jugadores, o importe un archivo TRF.
players-hint = Intro edita · Insert añade · Supr quita a un jugador que aún no ha jugado.
players-remove-confirm = ¿Quitar a { $name } del torneo?
players-withdraw = Retirar
players-withdraw-from = Sin emparejar desde la ronda
players-reenter = Reincorporar
players-byes = Descansos y pausas…
players-byes-title = Descansos de { $name }
players-byes-help = Los descansos solicitados se respetan al emparejar la ronda. Un descanso de cero puntos también sirve como pausa.
players-sort = Ordenar el ranking inicial
players-late-entry = Inscripción tardía: se incorpora en la ronda { $round }. Las rondas anteriores cuentan como
bye-none = Juega
bye-half-points = Descanso de medio punto
bye-zero-points = Descanso de cero puntos (ausente)
bye-full-points = Descanso de un punto

## Base de jugadores

player-db-help = Descargue la lista Elo FIDE mensual (o importe una guardada, en zip, XML o TXT) y añada listas nacionales en CSV. Las búsquedas rellenan después nombres, títulos, Elo e identificadores.
player-db-download-fide = Descargar la lista FIDE
player-db-connecting = Conectando con ratings.fide.com…
player-db-downloading = Descargando la lista FIDE… { $mb } MB
player-db-importing = Importando… { $count } jugadores
player-db-imported = { $count } jugadores importados.
player-db-mapping-title = Importar lista nacional
player-db-mapping-help = Elija qué columna contiene cada campo. Se necesita una columna de nombre completo o una de apellido.
player-db-source-name = Nombre de la lista
player-db-has-header = La primera fila contiene los nombres de columna
player-db-field-name = Nombre completo
player-db-field-surname = Apellido
player-db-field-first-name = Nombre de pila
player-db-field-national-id = ID nacional
player-db-field-rating = Elo nacional
player-db-import = Importar
player-db-import-fide = Importar la lista Elo FIDE…
player-db-import-national = Importar lista nacional (CSV)…
player-db-search-placeholder = Buscar por nombre o ID FIDE
player-db-no-results = No se encontraron jugadores.
player-db-type-to-search = Escriba al menos dos letras para buscar.

## Rondas

round-n = Ronda { $round }
round-short = R{ $round }
rounds-next-tab = R{ $round } – siguiente
rounds-results-entered = { $entered } de { $total } resultados introducidos
rounds-swap = Intercambiar colores
rounds-check = Comprobar emparejamientos
rounds-check-ok = Ronda { $round }: no se encontraron problemas.
rounds-check-problems = La ronda { $round } incumple estas reglas:
rounds-print = Imprimir emparejamientos
rounds-delete = Eliminar ronda
rounds-delete-confirm = ¿Eliminar la ronda { $round } y todos sus resultados? Puede deshacerlo.
rounds-keys-hint = Escriba 1, 0 o 5 (=) para 1-0, 0-1, ½-½ · + / - para incomparecencias · * doble incomparecencia · Supr borra. El cursor pasa al siguiente tablero.
rounds-pair-title = Emparejar la ronda { $round }
rounds-auto-help = Emparejamiento automático ({ $system }), o manual más abajo.
rounds-pair-auto = Emparejar automáticamente
rounds-manual = Emparejar a mano
rounds-manual-help = Haga clic en dos jugadores para ponerlos en un tablero (el primero lleva blancas). Los jugadores aparecen por puntuación.
rounds-give-pab = Dar el descanso asignado a { $name }
rounds-requested-byes = Descansos solicitados en esta ronda
rounds-boards = { $count ->
    [one] 1 tablero
   *[other] { $count } tableros
}
rounds-commit = Emparejar la ronda { $round }
rounds-unpaired-left = { $count } jugador(es) aún sin emparejar.
rounds-none = No hay ninguna ronda que mostrar.
rounds-matches = { $count } encuentros
rounds-lineups = Alineaciones…
rounds-lineups-title = Alineaciones
rounds-lineups-help = Los tableros siguen el orden elegido aquí. Los resultados ya introducidos se conservan en los tableros cuyos jugadores no cambian.
rounds-team-keys-hint = Seleccione un tablero y escriba 1, 0 o 5 (=) para 1-0, 0-1, ½-½ · + / - incomparecencias · Supr borra. Las filas de encuentro muestran el marcador.
rounds-violations-title = Estos emparejamientos incumplen las reglas
rounds-violations-help = Chess Organiser encontró estos problemas. Corríjalos, o empareje igualmente si el árbitro así lo decidió.
rounds-violations-fix = Volver y corregir
rounds-violations-force = Emparejar igualmente

## Listas

list-name-pairings = Emparejamientos
list-name-standings = Clasificación
list-name-crosstable = Tabla cruzada
list-name-team-pairings = Emparejamientos por equipos
list-name-team-standings = Clasificación por equipos
lists-language = Idioma
lists-print = Imprimir / PDF…
lists-format = Formato de exportación
lists-export = Exportar…
lists-round = Ronda
lists-no-rounds = Aún no se ha emparejado ninguna ronda.
lists-print-hint = Imprimir / PDF abre la lista en su navegador; use su diálogo de impresión para imprimir o guardar como PDF.

## Equipos y premios

teams-add = Añadir equipo
teams-edit = Editar equipo
teams-name = Nombre del equipo
teams-captain = Capitán
teams-members = Miembros
teams-board-order = Orden de tableros (en el orden marcado)
teams-empty = Aún no hay equipos.
teams-individual-note = Es un torneo individual; los equipos solo se usan para premios por equipos.
teams-remove-confirm = ¿Quitar el equipo { $name }?
teams-standings = Clasificación por equipos tras la ronda { $round }
prizes-add = Añadir categoría
prizes-category = Categoría
prizes-filter = Quién puede optar
prizes-prizes = Premios
prizes-prizes-help = Premios, uno por línea, en orden de puesto
prizes-empty = Aún no hay categorías de premios.
prizes-max-rating = Elo como máximo
prizes-min-rating = Elo como mínimo
prizes-born-from-label = Nacido en o después de (juveniles)
prizes-born-until-label = Nacido en o antes de (veteranos)
prizes-born-from = nac. ≥ { $year }
prizes-born-until = nac. ≤ { $year }
prizes-unrated = Solo sin Elo
prizes-women = Mujeres
prizes-winners = Premiados tras la ronda { $round }
prizes-one-each = Cada jugador recibe como máximo un premio, en el orden de las categorías.

## Elo y publicación

ratings-trf-title = Informe de Elo FIDE
ratings-trf-help = Exporte el torneo como archivo TRF-2026 para el responsable de Elo de la federación.
ratings-changes-title = Variaciones de Elo
ratings-changes-help = Puntuaciones esperadas, factores K y variación de Elo de cada jugador con Elo.
ratings-calculate = Calcular variaciones de Elo
ratings-norms-title = Normas de título
ratings-check-norms = Comprobar normas
ratings-norms-help = Comprueba los requisitos FIDE B.01 para una norma de GM, MI, WGM o WIM en las rondas jugadas: partidas, rivales con Elo y con título, federaciones y rendimiento.
ratings-norms-none = Ninguna norma se aplica a este jugador.
ratings-norm-achieved = norma conseguida
ratings-norm-missed = no conseguida
ratings-norm-stats = { $games } partidas · Elo medio de los rivales { $aro } · rendimiento { $perf }
ratings-first-rating = primer Elo { $rating }
publish-help = Suba emparejamientos y clasificaciones a un sitio de resultados para que los jugadores sigan el torneo en el móvil. Todo sigue funcionando sin conexión; las subidas se hacen cuando hay conexión.
publish-server = Servidor de resultados
publish-token = Token de subida
publish-upload = Subir ahora
publish-done = Subido. Los jugadores pueden seguir el torneo en línea.
publish-live-at = En línea en
publish-server-title = Mantener un servidor de resultados
publish-server-help = El programa co-publish sirve los torneos subidos. Ejecútelo en cualquier servidor detrás de HTTPS con un token de subida; la dirección y el token van en los campos de arriba.

## Preferencias y atajos

settings-language = Idioma
settings-theme = Apariencia
settings-translations-note = Las listas impresas pueden usar un idioma distinto del programa: elíjalo en la pantalla Listas. El texto aún no traducido aparece en inglés.
settings-updates = Actualizaciones
settings-check-updates = Buscar actualizaciones
settings-install-update = Instalar la versión { $version } y reiniciar
settings-updates-disabled = Esta compilación ({ $version }) no se actualiza sola; descargue las nuevas versiones del sitio del proyecto.
settings-up-to-date = Tiene la última versión ({ $version }).
theme-system = Igual que el sistema
theme-light = Claro
theme-dark = Oscuro
shortcuts-title = Atajos de teclado
shortcut-white-wins = Ganan blancas (1-0)
shortcut-black-wins = Ganan negras (0-1)
shortcut-draw = Tablas (½-½)
shortcut-white-forfeit = Ganan blancas por incomparecencia (+/-)
shortcut-black-forfeit = Ganan negras por incomparecencia (-/+)
shortcut-double-forfeit = Doble incomparecencia (-/-)
shortcut-clear-result = Borrar el resultado
shortcut-move = Moverse entre filas
shortcut-edit = Editar la fila seleccionada
shortcut-undo = Deshacer el último cambio
shortcut-redo = Rehacer
shortcut-pair = Emparejar la siguiente ronda
shortcut-sections = Ir a una sección

## Pantalla de bienvenida

welcome-sample-title = ¿Es nuevo? Explore con datos de ejemplo
welcome-sample-body = Abre un abierto rápido de 25 jugadores con cuatro rondas jugadas (la última a medio introducir), una inscripción tardía, un retiro, descansos, categorías de premios y una base de jugadores de ejemplo. Todo es ficticio; pruebe lo que quiera y deshágalo.
welcome-sample-load = Abrir el torneo de ejemplo
welcome-tour = Recorrido por el programa
welcome-features = Qué hace Chess Organiser
welcome-free = gratis para todas las federaciones y clubes
welcome-shot-rounds = Emparejamientos y resultados
welcome-shot-rounds-body = Introduzca los resultados con el teclado: 1, 0 o 5 en cada tablero, + y − para incomparecencias. El cursor pasa al siguiente tablero y cualquier cambio se puede deshacer.
welcome-shot-players = Jugadores
welcome-shot-players-body = Inscriba jugadores a mano o desde su lista de Elo, con títulos, federaciones y Elo. Inscripciones tardías, retiros, descansos solicitados y pausas se gestionan por ronda.
welcome-shot-lists = Listas
welcome-shot-lists-body = Ranking inicial, emparejamientos y clasificaciones listos para imprimir, guardar en PDF o exportar, en el idioma que elija.
welcome-shot-overview = Resumen
welcome-shot-overview-body = Vea de un vistazo cómo va el torneo y qué hacer a continuación.
feat-files = Un archivo por torneo
feat-files-body = Se guarda tras cada cambio y resiste un corte de luz. Funciona sin conexión; cópielo o envíelo como cualquier documento. Deshacer ilimitado con historial completo.
feat-players = Inscripción
feat-players-body = Jugadores de la lista FIDE o nacional, inscripciones tardías con descansos de cero o medio punto, retiros, pausas y ranking inicial FIDE.
feat-results = Introducción rápida de resultados
feat-results-body = Resultados desde el teclado, correcciones en rondas anteriores y emparejamiento manual con comprobación de repeticiones, compañeros de club y descansos.
feat-lists = Listas e impresión
feat-lists-body = Emparejamientos, clasificaciones, tablas cruzadas, fichas de jugador, fichas de mesa y listas de premios para imprimir, guardar en PDF o exportar a HTML, CSV y Excel.
feat-trf = Informes de Elo FIDE
feat-trf-body = Exportación e importación de archivos TRF-2026 para el responsable de Elo y para pasar torneos entre programas.
feat-pairing = Emparejamientos FIDE automáticos
feat-pairing-body = Sistemas holandés y Burstein comprobados contra el motor avalado por la FIDE, aceleración de Bakú, Dubov, Lim, Monrad y round robin con tablas de Berger.
feat-teams = Equipos y premios
feat-teams-body = Suizo y round robin por equipos con alineaciones y desempates por equipos; categorías de premios para mujeres, juveniles y tramos de Elo.
feat-ratings = Elo y normas
feat-ratings-body = Todos los desempates FIDE, variaciones de Elo según FIDE B.02 y comprobación de normas de GM, MI, WGM y WIM.
feat-languages = Idiomas y resultados en directo
feat-languages-body = Francés, árabe, suajili, portugués y español, y un sitio de resultados en directo para los móviles de los jugadores.
sample-replace-confirm = ¿Cerrar el torneo actual y abrir una copia nueva del torneo de ejemplo? Su torneo está guardado y sigue en Torneos recientes.

setup-note-unverified = Los emparejamientos siguen las reglas de este sistema con los mismos criterios absolutos que el motor holandés. A diferencia de los motores holandés y Burstein, no hay un programa de referencia avalado con el que comprobarlo, así que revise los emparejamientos antes de publicarlos.
setup-note-monrad = El Monrad empareja al primer jugador sin emparejar con el siguiente al que aún no se ha enfrentado. Está pensado para torneos de club, no para torneos válidos para Elo FIDE.
setup-note-team-swiss = Los equipos se emparejan con los criterios holandeses sobre puntos de match (o de partida); el color del equipo es el del primer tablero.
