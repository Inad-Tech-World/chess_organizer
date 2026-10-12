# Mensagens da interface. Títulos e colunas partilhados com as listas
# impressas estão em lists.ftl.

## Geral

untitled = Torneio sem título
new-tournament = Novo torneio
action-open = Abrir…
action-import-trf = Importar TRF…
action-export-trf = Exportar TRF…
action-create = Criar
action-cancel = Cancelar
action-close = Fechar
action-save = Guardar
action-edit = Editar
action-remove = Remover
action-clear = Limpar
action-undo = Anular
action-redo = Refazer
action-dismiss = Dispensar
action-move-up = Subir
action-move-down = Descer
undo-what = Anular { $what }
planned-badge = Fase { $phase }
no-tournament-hint = Abra ou crie primeiro um torneio.
export-done = { $path } guardado
import-warnings = Importado com observações:
import-new-window-hint = Feche primeiro este torneio, ou use Ficheiro › Abrir numa nova janela.
start-tagline = Gestão gratuita de torneios suíços, todos-contra-todos e por equipas — offline, no seu computador.
start-recent = Torneios recentes
status-rounds = Ronda { $paired } de { $total }
status-withdrawn = Desistiu a partir da ronda { $round }
status-late = Entra na ronda { $round }

## Navegação

nav-label = Secções
nav-overview = Visão geral
nav-setup = Configuração do torneio
nav-players = Jogadores
nav-rounds = Emparceiramentos e resultados
nav-lists = Listas
nav-teams = Equipas
nav-prizes = Prémios
nav-ratings = Rating e normas
nav-publish = Publicar
nav-player-db = Base de jogadores
nav-settings = Preferências

## Visão geral

overview-type = Tipo de torneio
overview-players = Jogadores
overview-active = { $count } ativos
overview-rounds = Rondas emparceiradas
overview-pending = Resultados em falta
overview-next = Próximo passo
overview-add-players = Adicionar jogadores
overview-enter-results = Introduzir os resultados da ronda { $round }
overview-pair = Emparceirar a ronda { $round }
overview-final = Classificação final

## Tipos de torneio

kind-swiss-dutch = Suíço – sistema holandês FIDE
kind-swiss-dubov = Suíço – sistema Dubov FIDE
kind-swiss-burstein = Suíço – sistema Burstein FIDE
kind-swiss-lim = Suíço – sistema Lim FIDE
kind-swiss-monrad = Suíço – simples (Monrad)
kind-rr = Todos contra todos
kind-rr-double = Todos contra todos a duas voltas
kind-team-swiss = Suíço por equipas
kind-team-rr = Todos contra todos por equipas
kind-team-rr-double = Todos contra todos por equipas a duas voltas

## Configuração

info-name = Nome do torneio
info-name-placeholder = p. ex. Open de Kampala 2026
setup-info = Informação do torneio
setup-venue = Local
setup-start-date = Data de início
setup-end-date = Data de fim
setup-deputies = Árbitros adjuntos (separados por vírgulas)
setup-organizer = Organizador
setup-system-title = Sistema e rondas
setup-system = Tipo de torneio
setup-rounds = Número de rondas
setup-rating-type = Tipo de rating
setup-initial-color = Cor do cabeça de série na mesa 1, ronda 1
setup-acceleration = Aceleração
setup-acceleration-none = Nenhuma
setup-acceleration-baku = Aceleração de Baku
setup-acceleration-manual = Manual (do TRF)
setup-fide-rated = Torneio válido para rating FIDE
setup-kind-locked = O tipo de torneio não pode mudar depois de emparceirada a ronda 1.
setup-points = Pontos
points-win = Vitória
points-draw = Empate
points-loss = Derrota
points-forfeit-win = Vitória por falta de comparência
points-forfeit-loss = Derrota por falta de comparência
points-pab = Isento atribuído
points-half-bye = Bye de meio ponto
points-full-bye = Bye de um ponto
points-zero-bye = Bye de zero pontos / ausente
setup-tiebreaks = Desempates (FIDE C.07)
setup-tiebreak = Desempate
setup-tb-cut = Corte / mediano
setup-tb-forfeits = Faltas de comparência como jogadas (/P)
setup-tb-add = Adicionar desempate
setup-tb-help = Os códigos seguem os descritores de ordem do TRF-2026 (p. ex. BH/C1 = Buchholz com corte 1). Um «–» indica que o valor não pode ser calculado para esse jogador, p. ex. um desempate de rating sem adversários com rating.
setup-forbidden = Emparceiramentos proibidos
setup-forbidden-help = Os jogadores do mesmo grupo nunca jogam entre si: colegas de clube nas primeiras rondas, familiares ou qualquer par que o regulamento proíba.
setup-group-name = Nome do grupo
setup-group-ranks = Números iniciais, p. ex. 3, 17, 24
setup-group-add = Adicionar grupo
setup-group-default = Grupo
setup-group-too-small = Um grupo precisa de pelo menos dois jogadores (por número inicial).
setup-team = Configuração das equipas
setup-boards = Tabuleiros por encontro
setup-reserves = Suplentes
setup-match-win = Pontos de encontro por vitória
setup-match-draw = Pontos de encontro por empate
setup-match-loss = Pontos de encontro por derrota
setup-primary-score = Classificar por
setup-color-preference = Preferências de cor
score-match-points = Pontos de encontro
score-game-points = Pontos de partida
color-pref-a = Tipo A
color-pref-b = Tipo B
color-pref-none = Nenhuma
color-white = Brancas
color-black = Pretas
setup-saved = Configuração guardada.

## Nomes dos desempates

tb-DE = Confronto direto
tb-BPG = Partidas jogadas com as pretas
tb-BWG = Partidas ganhas com as pretas
tb-REP = Rondas escolhidas para jogar
tb-SB = Sonneborn-Berger
tb-ARO = Rating médio dos adversários
tb-TPR = Performance no torneio
tb-PTP = Performance perfeita
tb-APRO = Performance média dos adversários
tb-APPO = Performance perfeita média dos adversários
tb-WIN = Número de vitórias
tb-WON = Partidas ganhas no tabuleiro
tb-PS = Pontuação progressiva
tb-BH = Buchholz
tb-AOB = Média do Buchholz dos adversários
tb-FB = Buchholz antecipado
tb-KS = Sistema Koya
tb-BC = Contagem por tabuleiros
tb-TBR = Resultado do primeiro tabuleiro
tb-BBE = Eliminação do último tabuleiro
tb-MPvGP = Pontos de encontro contra pontos de partida
tb-EMMSB = Sonneborn-Berger por equipas (PE × PE)
tb-EMGSB = Sonneborn-Berger por equipas (PE × PP)
tb-EGMSB = Sonneborn-Berger por equipas (PP × PE)
tb-EGGSB = Sonneborn-Berger por equipas (PP × PP)
tb-EDE = Confronto direto alargado
tb-EDEBT = CDA + tabuleiros + primeiro tabuleiro
tb-EDEBB = CDA + tabuleiros + último tabuleiro
tb-EDET = CDA + primeiro tabuleiro
tb-EDEB = CDA + último tabuleiro
tb-SSSC = Pontuação compensada pela força
tb-OTHER_SEC = Pontuação secundária

## Jogadores

col-status = Estado
player-name = Nome (Apelido, Nome)
player-name-placeholder = Apelido, Nome
player-national-id = ID nacional
player-k-factor = Fator K
sex-male = Masculino
sex-female = Feminino
rating-standard = Clássico
rating-rapid = Rápido
rating-blitz = Blitz
rating-national = Nacional
players-count = { $count ->
    [one] 1 jogador
   *[other] { $count } jogadores
}
players-add = Adicionar jogador
players-add-another = Adicionar e continuar
players-edit = Editar jogador
players-empty = Ainda não há jogadores. Adicione-os um a um, a partir da base de jogadores, ou importe um ficheiro TRF.
players-hint = Enter edita · Insert adiciona · Delete remove um jogador que ainda não jogou.
players-remove-confirm = Remover { $name } do torneio?
players-withdraw = Desistência
players-withdraw-from = Sem emparceiramento a partir da ronda
players-reenter = Reintegrar
players-byes = Byes e pausas…
players-byes-title = Byes de { $name }
players-byes-help = Os byes pedidos são respeitados ao emparceirar a ronda. Um bye de zero pontos também serve de pausa.
players-sort = Ordenar o ranking inicial
players-late-entry = Inscrição tardia: entra na ronda { $round }. As rondas anteriores contam como
bye-none = Joga
bye-half-points = Bye de meio ponto
bye-zero-points = Bye de zero pontos (ausente)
bye-full-points = Bye de um ponto

## Base de jogadores

player-db-help = Descarregue a lista mensal de rating FIDE (ou importe uma guardada, em zip, XML ou TXT) e adicione listas nacionais em CSV. As pesquisas preenchem depois nomes, títulos, ratings e identificadores.
player-db-download-fide = Descarregar a lista FIDE
player-db-connecting = A ligar a ratings.fide.com…
player-db-downloading = A descarregar a lista FIDE… { $mb } MB
player-db-importing = A importar… { $count } jogadores
player-db-imported = { $count } jogadores importados.
player-db-mapping-title = Importar lista nacional
player-db-mapping-help = Escolha a coluna de cada campo. É necessária uma coluna de nome completo ou de apelido.
player-db-source-name = Nome da lista
player-db-has-header = A primeira linha contém os nomes das colunas
player-db-field-name = Nome completo
player-db-field-surname = Apelido
player-db-field-first-name = Nome próprio
player-db-field-national-id = ID nacional
player-db-field-rating = Rating nacional
player-db-import = Importar
player-db-import-fide = Importar a lista de rating FIDE…
player-db-import-national = Importar lista nacional (CSV)…
player-db-search-placeholder = Pesquisar por nome ou ID FIDE
player-db-no-results = Nenhum jogador encontrado.
player-db-type-to-search = Escreva pelo menos duas letras para pesquisar.

## Rondas

round-n = Ronda { $round }
round-short = R{ $round }
rounds-next-tab = R{ $round } – seguinte
rounds-results-entered = { $entered } de { $total } resultados introduzidos
rounds-swap = Trocar cores
rounds-check = Verificar emparceiramentos
rounds-check-ok = Ronda { $round }: nenhum problema encontrado.
rounds-check-problems = A ronda { $round } viola estas regras:
rounds-print = Imprimir emparceiramentos
rounds-delete = Eliminar ronda
rounds-delete-confirm = Eliminar a ronda { $round } e todos os resultados? Pode anular depois.
rounds-keys-hint = Escreva 1, 0 ou 5 (=) para 1-0, 0-1, ½-½ · + / - para faltas de comparência · * falta dupla · Delete apaga. O cursor passa ao tabuleiro seguinte.
rounds-pair-title = Emparceirar a ronda { $round }
rounds-auto-help = Emparceiramento automático ({ $system }), ou manual abaixo.
rounds-pair-auto = Emparceirar automaticamente
rounds-manual = Emparceirar manualmente
rounds-manual-help = Clique em dois jogadores para os pôr num tabuleiro (o primeiro joga com as brancas). Os jogadores aparecem por pontuação.
rounds-give-pab = Dar a isenção a { $name }
rounds-requested-byes = Byes pedidos nesta ronda
rounds-boards = { $count ->
    [one] 1 tabuleiro
   *[other] { $count } tabuleiros
}
rounds-commit = Emparceirar a ronda { $round }
rounds-unpaired-left = { $count } jogador(es) ainda por emparceirar.
rounds-none = Nenhuma ronda para mostrar.
rounds-matches = { $count } encontros
rounds-lineups = Constituições…
rounds-lineups-title = Constituições das equipas
rounds-lineups-help = Os tabuleiros seguem a ordem escolhida aqui. Os resultados já introduzidos mantêm-se nos tabuleiros cujos jogadores não mudaram.
rounds-team-keys-hint = Selecione um tabuleiro e escreva 1, 0 ou 5 (=) para 1-0, 0-1, ½-½ · + / - faltas de comparência · Delete apaga. As linhas de encontro mostram o resultado parcial.
rounds-violations-title = Estes emparceiramentos violam as regras
rounds-violations-help = O Chess Organiser encontrou estes problemas. Corrija-os, ou emparceire na mesma se o árbitro assim decidiu.
rounds-violations-fix = Voltar e corrigir
rounds-violations-force = Emparceirar na mesma

## Listas

list-name-pairings = Emparceiramentos
list-name-standings = Classificação
list-name-crosstable = Tabela cruzada
list-name-team-pairings = Emparceiramentos por equipas
list-name-team-standings = Classificação por equipas
lists-language = Língua
lists-print = Imprimir / PDF…
lists-format = Formato de exportação
lists-export = Exportar…
lists-round = Ronda
lists-no-rounds = Ainda não foi emparceirada nenhuma ronda.
lists-print-hint = Imprimir / PDF abre a lista no navegador; use o diálogo de impressão para imprimir ou guardar em PDF.

## Equipas e prémios

teams-add = Adicionar equipa
teams-edit = Editar equipa
teams-name = Nome da equipa
teams-captain = Capitão
teams-members = Membros
teams-board-order = Ordem dos tabuleiros (pela ordem marcada)
teams-empty = Ainda não há equipas.
teams-individual-note = Este é um torneio individual; as equipas servem apenas para prémios por equipas.
teams-remove-confirm = Remover a equipa { $name }?
teams-standings = Classificação por equipas após a ronda { $round }
prizes-add = Adicionar categoria
prizes-category = Categoria
prizes-filter = Quem pode concorrer
prizes-prizes = Prémios
prizes-prizes-help = Prémios, um por linha, pela ordem dos lugares
prizes-empty = Ainda não há categorias de prémios.
prizes-max-rating = Rating no máximo
prizes-min-rating = Rating no mínimo
prizes-born-from-label = Nascido em ou depois de (jovens)
prizes-born-until-label = Nascido em ou antes de (veteranos)
prizes-born-from = nasc. ≥ { $year }
prizes-born-until = nasc. ≤ { $year }
prizes-unrated = Só sem rating
prizes-women = Femininos
prizes-winners = Premiados após a ronda { $round }
prizes-one-each = Cada jogador recebe no máximo um prémio, pela ordem das categorias.

## Rating e publicação

ratings-trf-title = Relatório de rating FIDE
ratings-trf-help = Exporte o torneio como ficheiro TRF-2026 para o responsável de rating da federação.
ratings-changes-title = Variações de rating
ratings-changes-help = Pontuações esperadas, fatores K e variação de rating de cada jogador com rating.
ratings-calculate = Calcular variações de rating
ratings-norms-title = Normas de título
ratings-check-norms = Verificar normas
ratings-norms-help = Verifica os requisitos FIDE B.01 para uma norma de GM, MI, WGM ou WIM nas rondas jogadas: partidas, adversários com rating e com título, federações e performance.
ratings-norms-none = Nenhuma norma se aplica a este jogador.
ratings-norm-achieved = norma obtida
ratings-norm-missed = não obtida
ratings-norm-stats = { $games } partidas · rating médio dos adversários { $aro } · performance { $perf }
ratings-first-rating = primeiro rating { $rating }
publish-help = Envie emparceiramentos e classificações para um site de resultados para que os jogadores acompanhem o torneio no telemóvel. Tudo continua a funcionar offline; os envios acontecem quando há ligação.
publish-server = Servidor de resultados
publish-token = Token de envio
publish-upload = Enviar agora
publish-done = Enviado. Os jogadores podem acompanhar o torneio online.
publish-live-at = Online em
publish-server-title = Manter um servidor de resultados
publish-server-help = O programa co-publish serve os torneios enviados. Execute-o em qualquer servidor atrás de HTTPS com um token de envio; o endereço e o token vão nos campos acima.

## Preferências e atalhos

settings-language = Língua
settings-theme = Aparência
settings-translations-note = As listas impressas podem usar uma língua diferente da do programa: escolha-a no ecrã Listas. O texto ainda não traduzido aparece em inglês.
settings-updates = Atualizações
settings-check-updates = Procurar atualizações
settings-install-update = Instalar a versão { $version } e reiniciar
settings-updates-disabled = Esta versão ({ $version }) não se atualiza sozinha; descarregue as novas versões no site do projeto.
settings-up-to-date = Tem a versão mais recente ({ $version }).
theme-system = Igual ao sistema
theme-light = Claro
theme-dark = Escuro
shortcuts-title = Atalhos de teclado
shortcut-white-wins = Ganham as brancas (1-0)
shortcut-black-wins = Ganham as pretas (0-1)
shortcut-draw = Empate (½-½)
shortcut-white-forfeit = Brancas ganham por falta de comparência (+/-)
shortcut-black-forfeit = Pretas ganham por falta de comparência (-/+)
shortcut-double-forfeit = Falta de comparência dupla (-/-)
shortcut-clear-result = Apagar o resultado
shortcut-move = Mover entre linhas
shortcut-edit = Editar a linha selecionada
shortcut-undo = Anular a última alteração
shortcut-redo = Refazer
shortcut-pair = Emparceirar a ronda seguinte
shortcut-sections = Ir para uma secção

## Ecrã de boas-vindas

welcome-sample-title = É novo? Explore com dados de exemplo
welcome-sample-body = Abre um open rápido de 25 jogadores com quatro rondas jogadas (a última meio introduzida), uma inscrição tardia, uma desistência, byes, categorias de prémios e uma base de jogadores de exemplo. Tudo é fictício; experimente e anule à vontade.
welcome-sample-load = Abrir o torneio de exemplo
welcome-tour = Visita ao programa
welcome-features = O que faz o Chess Organiser
welcome-free = gratuito para todas as federações e clubes
welcome-shot-rounds = Emparceiramentos e resultados
welcome-shot-rounds-body = Introduza resultados pelo teclado: 1, 0 ou 5 em cada tabuleiro, + e − para faltas de comparência. O cursor passa ao tabuleiro seguinte e qualquer alteração pode ser anulada.
welcome-shot-players = Jogadores
welcome-shot-players-body = Inscreva jogadores à mão ou a partir da lista de rating, com títulos, federações e ratings. Inscrições tardias, desistências, byes pedidos e pausas são tratados ronda a ronda.
welcome-shot-lists = Listas
welcome-shot-lists-body = Ranking inicial, emparceiramentos e classificações prontos a imprimir, guardar em PDF ou exportar, na língua que escolher.
welcome-shot-overview = Visão geral
welcome-shot-overview-body = Veja num relance como está o torneio e o que fazer a seguir.
feat-files = Um ficheiro por torneio
feat-files-body = Guardado após cada alteração e seguro em caso de falha de energia. Funciona offline; copie-o ou envie-o como qualquer documento. Anulação ilimitada com histórico completo.
feat-players = Inscrições
feat-players-body = Jogadores da lista FIDE ou nacional, inscrições tardias com byes de zero ou meio ponto, desistências, pausas e ranking inicial FIDE.
feat-results = Introdução rápida de resultados
feat-results-body = Resultados pelo teclado, correções em rondas anteriores e emparceiramento manual com verificação de repetições, colegas de clube e byes.
feat-lists = Listas e impressão
feat-lists-body = Emparceiramentos, classificações, tabelas cruzadas, fichas de jogador, fichas de mesa e listas de prémios para imprimir, guardar em PDF ou exportar em HTML, CSV e Excel.
feat-trf = Relatórios de rating FIDE
feat-trf-body = Exportação e importação de ficheiros TRF-2026 para o responsável de rating e para passar torneios entre programas.
feat-pairing = Emparceiramentos FIDE automáticos
feat-pairing-body = Sistemas holandês e Burstein verificados contra o motor aprovado pela FIDE, aceleração de Baku, Dubov, Lim, Monrad e todos-contra-todos com tabelas de Berger.
feat-teams = Equipas e prémios
feat-teams-body = Suíço e todos-contra-todos por equipas com constituições e desempates por equipas; categorias de prémios para femininos, jovens e escalões de rating.
feat-ratings = Rating e normas
feat-ratings-body = Todos os desempates FIDE, variações de rating pela norma FIDE B.02 e verificação de normas de GM, MI, WGM e WIM.
feat-languages = Línguas e resultados em direto
feat-languages-body = Francês, árabe, suaíli, português e espanhol, e um site de resultados em direto para os telemóveis dos jogadores.
sample-replace-confirm = Fechar o torneio atual e abrir uma nova cópia do torneio de exemplo? O seu torneio está guardado e continua nos torneios recentes.

setup-note-unverified = Os emparceiramentos seguem as regras deste sistema com os mesmos critérios absolutos do motor holandês. Ao contrário dos motores holandês e Burstein, não existe um programa de referência aprovado para o verificar, por isso reveja os emparceiramentos antes de os publicar.
setup-note-monrad = O Monrad emparceira o primeiro jogador livre com o seguinte que ainda não defrontou. Destina-se a torneios de clube, não a torneios válidos para rating FIDE.
setup-note-team-swiss = As equipas são emparceiradas pelos critérios holandeses sobre pontos de encontro (ou de partida); a cor da equipa é a do primeiro tabuleiro.
