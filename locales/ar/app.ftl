# رسائل واجهة البرنامج. العناوين والأعمدة المشتركة مع القوائم المطبوعة
# موجودة في lists.ftl.

## عام

untitled = بطولة بدون عنوان
new-tournament = بطولة جديدة
action-open = فتح…
action-import-trf = استيراد ملف TRF…
action-export-trf = تصدير ملف TRF…
action-create = إنشاء
action-cancel = إلغاء
action-close = إغلاق
action-save = حفظ
action-edit = تعديل
action-remove = حذف
action-clear = مسح
action-undo = تراجع
action-redo = إعادة
action-dismiss = تجاهل
action-move-up = نقل لأعلى
action-move-down = نقل لأسفل
undo-what = تراجع عن { $what }
planned-badge = المرحلة { $phase }
no-tournament-hint = افتح بطولة أو أنشئ واحدة أولاً.
export-done = تم حفظ { $path }
import-warnings = تم الاستيراد مع ملاحظات:
import-new-window-hint = أغلق هذه البطولة أولاً، أو استخدم ملف › فتح في نافذة جديدة.
start-tagline = إدارة مجانية لبطولات النظام السويسري والدوري والفرق — دون اتصال، على حاسوبك.
start-recent = البطولات الأخيرة
status-rounds = الجولة { $paired } من { $total }
status-withdrawn = منسحب ابتداءً من الجولة { $round }
status-late = ينضم في الجولة { $round }

## التنقل

nav-label = الأقسام
nav-overview = نظرة عامة
nav-setup = إعدادات البطولة
nav-players = اللاعبون
nav-rounds = المقابلات والنتائج
nav-lists = القوائم
nav-teams = الفرق
nav-prizes = الجوائز
nav-ratings = التصنيف والنورمات
nav-publish = النشر
nav-player-db = قاعدة بيانات اللاعبين
nav-settings = الإعدادات

## نظرة عامة

overview-type = نوع البطولة
overview-players = اللاعبون
overview-active = { $count } نشطون
overview-rounds = الجولات المقترنة
overview-pending = نتائج ناقصة
overview-next = الخطوة التالية
overview-add-players = إضافة لاعبين
overview-enter-results = إدخال نتائج الجولة { $round }
overview-pair = إجراء قرعة الجولة { $round }
overview-final = الترتيب النهائي

## أنواع البطولات

kind-swiss-dutch = سويسري – النظام الهولندي (فيدي)
kind-swiss-dubov = سويسري – نظام دوبوف (فيدي)
kind-swiss-burstein = سويسري – نظام بورشتاين (فيدي)
kind-swiss-lim = سويسري – نظام ليم (فيدي)
kind-swiss-monrad = سويسري – بسيط (مونراد)
kind-rr = دوري
kind-rr-double = دوري من دورين
kind-team-swiss = سويسري للفرق
kind-team-rr = دوري للفرق
kind-team-rr-double = دوري للفرق من دورين

## الإعدادات

info-name = اسم البطولة
info-name-placeholder = مثلاً: بطولة كمبالا المفتوحة 2026
setup-info = معلومات البطولة
setup-venue = المكان
setup-start-date = تاريخ البداية
setup-end-date = تاريخ النهاية
setup-deputies = الحكام المساعدون (مفصولون بفواصل)
setup-organizer = المنظم
setup-system-title = النظام والجولات
setup-system = نوع البطولة
setup-rounds = عدد الجولات
setup-rating-type = نوع التصنيف
setup-initial-color = لون المصنف الأول على الطاولة 1 في الجولة 1
setup-acceleration = التسريع
setup-acceleration-none = بدون
setup-acceleration-baku = تسريع باكو
setup-acceleration-manual = يدوي (من ملف TRF)
setup-fide-rated = بطولة محتسبة لتصنيف فيدي
setup-kind-locked = لا يمكن تغيير نوع البطولة بعد قرعة الجولة 1.
setup-points = النقاط
points-win = فوز
points-draw = تعادل
points-loss = خسارة
points-forfeit-win = فوز بالغياب
points-forfeit-loss = خسارة بالغياب
points-pab = راحة مخصصة بالقرعة
points-half-bye = راحة بنصف نقطة
points-full-bye = راحة بنقطة كاملة
points-zero-bye = راحة بدون نقاط / غائب
setup-tiebreaks = كسر التعادل (فيدي C.07)
setup-tiebreak = معيار كسر التعادل
setup-tb-cut = الحذف / الوسيط
setup-tb-forfeits = اعتبار مباريات الغياب كأنها لُعبت (/P)
setup-tb-add = إضافة معيار
setup-tb-help = تتبع الرموز واصفات الترتيب في TRF-2026 (مثلاً BH/C1 = بوخهولز مع حذف أضعف نتيجة). تعني «–» أن القيمة لا يمكن حسابها لذلك اللاعب، مثل معيار تصنيف بلا منافسين مصنفين.
setup-forbidden = مقابلات ممنوعة
setup-forbidden-help = لا يتقابل أبداً لاعبو المجموعة الواحدة: زملاء النادي في الجولات الأولى، أو أفراد العائلة، أو أي ثنائي تمنعه اللوائح.
setup-group-name = اسم المجموعة
setup-group-ranks = أرقام البداية، مثلاً 3، 17، 24
setup-group-add = إضافة مجموعة
setup-group-default = مجموعة
setup-group-too-small = تحتاج المجموعة إلى لاعبين اثنين على الأقل (حسب رقم البداية).
setup-team = إعدادات الفرق
setup-boards = عدد الطاولات في المباراة
setup-reserves = الاحتياط
setup-match-win = نقاط المباراة للفوز
setup-match-draw = نقاط المباراة للتعادل
setup-match-loss = نقاط المباراة للخسارة
setup-primary-score = الترتيب حسب
setup-color-preference = تفضيلات اللون
score-match-points = نقاط المباريات
score-game-points = نقاط الأدوار
color-pref-a = النوع أ
color-pref-b = النوع ب
color-pref-none = بدون
color-white = الأبيض
color-black = الأسود
setup-saved = تم حفظ الإعدادات.

## أسماء معايير كسر التعادل

tb-DE = المواجهة المباشرة
tb-BPG = عدد الأدوار بالأسود
tb-BWG = عدد الانتصارات بالأسود
tb-REP = الجولات التي اختار اللعب فيها
tb-SB = سونبورن-بيرغر
tb-ARO = متوسط تصنيف المنافسين
tb-TPR = تصنيف الأداء
tb-PTP = الأداء المثالي
tb-APRO = متوسط أداء المنافسين
tb-APPO = متوسط الأداء المثالي للمنافسين
tb-WIN = عدد الانتصارات
tb-WON = الانتصارات على الرقعة
tb-PS = النقاط التراكمية
tb-BH = بوخهولز
tb-AOB = متوسط بوخهولز المنافسين
tb-FB = بوخهولز المسبق
tb-KS = نظام كويا
tb-BC = عدّ الطاولات
tb-TBR = نتيجة الطاولة الأولى
tb-BBE = استبعاد الطاولة الأخيرة
tb-MPvGP = نقاط المباريات مقابل نقاط الأدوار
tb-EMMSB = سونبورن-بيرغر للفرق (ن.م × ن.م)
tb-EMGSB = سونبورن-بيرغر للفرق (ن.م × ن.أ)
tb-EGMSB = سونبورن-بيرغر للفرق (ن.أ × ن.م)
tb-EGGSB = سونبورن-بيرغر للفرق (ن.أ × ن.أ)
tb-EDE = المواجهة المباشرة الموسعة
tb-EDEBT = م.م.م + الطاولات + الطاولة الأولى
tb-EDEBB = م.م.م + الطاولات + الطاولة الأخيرة
tb-EDET = م.م.م + الطاولة الأولى
tb-EDEB = م.م.م + الطاولة الأخيرة
tb-SSSC = النقاط المعدلة حسب القوة
tb-OTHER_SEC = النقاط الثانوية

## اللاعبون

col-status = الحالة
player-name = الاسم (اسم العائلة، الاسم الأول)
player-name-placeholder = اسم العائلة، الاسم الأول
player-national-id = الرقم الوطني
player-k-factor = معامل K
sex-male = ذكر
sex-female = أنثى
rating-standard = كلاسيكي
rating-rapid = سريع
rating-blitz = خاطف
rating-national = وطني
players-count = { $count ->
    [one] لاعب واحد
    [two] لاعبان
    [few] { $count } لاعبين
   *[other] { $count } لاعباً
}
players-add = إضافة لاعب
players-add-another = إضافة ومتابعة
players-edit = تعديل اللاعب
players-empty = لا يوجد لاعبون بعد. أضفهم واحداً تلو الآخر، أو من قاعدة بيانات اللاعبين، أو استورد ملف TRF.
players-hint = Enter للتعديل · Insert للإضافة · Delete لحذف لاعب لم يلعب بعد.
players-remove-confirm = حذف { $name } من البطولة؟
players-withdraw = انسحاب
players-withdraw-from = بدون مقابلة ابتداءً من الجولة
players-reenter = إعادة الإدراج
players-byes = الراحات والتوقفات…
players-byes-title = راحات { $name }
players-byes-help = تُحترم الراحات المطلوبة عند إجراء قرعة الجولة. والراحة بدون نقاط تصلح أيضاً كتوقف.
players-sort = ترتيب قائمة البداية
players-late-entry = تسجيل متأخر: ينضم في الجولة { $round }. تُحتسب الجولات السابقة على أنها
bye-none = يلعب
bye-half-points = راحة بنصف نقطة
bye-zero-points = راحة بدون نقاط (غائب)
bye-full-points = راحة بنقطة كاملة

## قاعدة بيانات اللاعبين

player-db-help = نزّل قائمة تصنيف فيدي الشهرية (أو استورد قائمة محفوظة بصيغة zip أو XML أو TXT) وأضف القوائم الوطنية من ملفات CSV. بعدها يملأ البحث الأسماء والألقاب والتصنيفات والأرقام.
player-db-download-fide = تنزيل قائمة فيدي
player-db-connecting = جارٍ الاتصال بـ ratings.fide.com…
player-db-downloading = جارٍ تنزيل قائمة فيدي… { $mb } ميغابايت
player-db-importing = جارٍ الاستيراد… { $count } لاعب
player-db-imported = تم استيراد { $count } لاعب.
player-db-mapping-title = استيراد قائمة وطنية
player-db-mapping-help = اختر العمود الذي يحتوي كل حقل. يلزم عمود للاسم الكامل أو عمود لاسم العائلة.
player-db-source-name = اسم القائمة
player-db-has-header = الصف الأول يحتوي أسماء الأعمدة
player-db-field-name = الاسم الكامل
player-db-field-surname = اسم العائلة
player-db-field-first-name = الاسم الأول
player-db-field-national-id = الرقم الوطني
player-db-field-rating = التصنيف الوطني
player-db-import = استيراد
player-db-import-fide = استيراد قائمة تصنيف فيدي…
player-db-import-national = استيراد قائمة وطنية (CSV)…
player-db-search-placeholder = البحث بالاسم أو رقم فيدي
player-db-no-results = لم يُعثر على لاعبين.
player-db-type-to-search = اكتب حرفين على الأقل للبحث.

## الجولات

round-n = الجولة { $round }
round-short = ج{ $round }
rounds-next-tab = ج{ $round } – التالية
rounds-results-entered = أُدخلت { $entered } من { $total } نتيجة
rounds-swap = تبديل الألوان
rounds-check = فحص المقابلات
rounds-check-ok = الجولة { $round }: لا توجد مخالفات للقواعد.
rounds-check-problems = الجولة { $round } تخالف هذه القواعد:
rounds-print = طباعة المقابلات
rounds-delete = حذف الجولة
rounds-delete-confirm = حذف الجولة { $round } وجميع نتائجها؟ يمكنك التراجع.
rounds-keys-hint = اكتب 1 أو 0 أو 5 (=) لـ 1-0 و0-1 و½-½ · + / - للفوز بالغياب · * غياب الطرفين · Delete للمسح. ينتقل المؤشر إلى الطاولة التالية.
rounds-pair-title = قرعة الجولة { $round }
rounds-auto-help = قرعة آلية ({ $system })، أو يدوية في الأسفل.
rounds-pair-auto = قرعة آلية
rounds-manual = قرعة يدوية
rounds-manual-help = انقر على لاعبين لوضعهما على طاولة (الأول يلعب بالأبيض). اللاعبون مرتبون حسب النقاط.
rounds-give-pab = منح { $name } الراحة المخصصة بالقرعة
rounds-requested-byes = الراحات المطلوبة في هذه الجولة
rounds-boards = { $count ->
    [one] طاولة واحدة
    [two] طاولتان
    [few] { $count } طاولات
   *[other] { $count } طاولة
}
rounds-commit = قرعة الجولة { $round }
rounds-unpaired-left = { $count } لاعب ما زالوا بدون مقابلة.
rounds-none = لا توجد جولة لعرضها.
rounds-matches = { $count } مباراة
rounds-lineups = التشكيلات…
rounds-lineups-title = التشكيلات
rounds-lineups-help = تتبع الطاولات الترتيب المختار هنا. وتُحفظ النتائج المدخلة للطاولات التي لم يتغير لاعبوها.
rounds-team-keys-hint = اختر طاولة واكتب 1 أو 0 أو 5 (=) لـ 1-0 و0-1 و½-½ · + / - للغياب · Delete للمسح. تعرض صفوف المباريات النتيجة الحالية.
rounds-violations-title = هذه المقابلات تخالف القواعد
rounds-violations-help = وجد Chess Organiser هذه المشكلات. صحّحها، أو أجرِ القرعة رغم ذلك إن قرر الحكم ذلك.
rounds-violations-fix = الرجوع والتصحيح
rounds-violations-force = إجراء القرعة رغم ذلك

## القوائم

list-name-pairings = المقابلات
list-name-standings = الترتيب
list-name-crosstable = الجدول المتقاطع
list-name-team-pairings = مقابلات الفرق
list-name-team-standings = ترتيب الفرق
lists-language = اللغة
lists-print = طباعة / PDF…
lists-format = صيغة التصدير
lists-export = تصدير…
lists-round = الجولة
lists-no-rounds = لم تُجرَ قرعة أي جولة بعد.
lists-print-hint = يفتح زر طباعة / PDF القائمة في المتصفح؛ استخدم نافذة الطباعة فيه للطباعة أو الحفظ بصيغة PDF.

## الفرق والجوائز

teams-add = إضافة فريق
teams-edit = تعديل الفريق
teams-name = اسم الفريق
teams-captain = القائد
teams-members = الأعضاء
teams-board-order = ترتيب الطاولات (حسب ترتيب التحديد)
teams-empty = لا توجد فرق بعد.
teams-individual-note = هذه بطولة فردية؛ تُستخدم الفرق هنا لجوائز الفرق فقط.
teams-remove-confirm = حذف الفريق { $name }؟
teams-standings = ترتيب الفرق بعد الجولة { $round }
prizes-add = إضافة فئة
prizes-category = الفئة
prizes-filter = المؤهلون
prizes-prizes = الجوائز
prizes-prizes-help = الجوائز، واحدة في كل سطر، حسب ترتيب المراكز
prizes-empty = لا توجد فئات جوائز بعد.
prizes-max-rating = التصنيف لا يزيد عن
prizes-min-rating = التصنيف لا يقل عن
prizes-born-from-label = مولود في هذه السنة أو بعدها (الناشئون)
prizes-born-until-label = مولود في هذه السنة أو قبلها (الرواد)
prizes-born-from = الميلاد ≥ { $year }
prizes-born-until = الميلاد ≤ { $year }
prizes-unrated = غير المصنفين فقط
prizes-women = السيدات
prizes-winners = الفائزون بالجوائز بعد الجولة { $round }
prizes-one-each = يحصل كل لاعب على جائزة واحدة على الأكثر، حسب ترتيب الفئات.

## التصنيف والنشر

ratings-trf-title = تقرير تصنيف فيدي
ratings-trf-help = صدّر البطولة كملف TRF-2026 لمسؤول التصنيف في الاتحاد.
ratings-changes-title = تغييرات التصنيف
ratings-changes-help = النقاط المتوقعة ومعاملات K وتغيير التصنيف لكل لاعب مصنف.
ratings-calculate = حساب تغييرات التصنيف
ratings-norms-title = نورمات الألقاب
ratings-check-norms = فحص النورمات
ratings-norms-help = يفحص شروط فيدي B.01 لنورم أستاذ كبير أو أستاذ دولي أو أستاذة كبيرة أو أستاذة دولية في الجولات الملعوبة: عدد الأدوار، والمنافسون المصنفون وحاملو الألقاب، والاتحادات، والأداء.
ratings-norms-none = لا ينطبق أي نورم على هذا اللاعب.
ratings-norm-achieved = تحقق النورم
ratings-norm-missed = لم يتحقق
ratings-norm-stats = { $games } أدوار · متوسط تصنيف المنافسين { $aro } · الأداء { $perf }
ratings-first-rating = أول تصنيف { $rating }
publish-help = ارفع المقابلات والترتيب إلى موقع للنتائج ليتابع اللاعبون البطولة على هواتفهم. يواصل البرنامج العمل دون اتصال؛ ويتم الرفع عند توفر الاتصال.
publish-server = خادم النتائج
publish-token = رمز الرفع
publish-upload = الرفع الآن
publish-done = تم الرفع. يمكن للاعبين متابعة البطولة عبر الإنترنت.
publish-live-at = متاحة على
publish-server-title = تشغيل خادم للنتائج
publish-server-help = يقدّم برنامج co-publish البطولات المرفوعة. شغّله على أي خادم خلف HTTPS مع رمز رفع؛ ثم ضع العنوان والرمز في الحقلين أعلاه.

## الإعدادات والاختصارات

settings-language = اللغة
settings-theme = المظهر
settings-translations-note = يمكن أن تستخدم القوائم المطبوعة لغة مختلفة عن لغة البرنامج: اخترها في شاشة القوائم. أي نص غير مترجم بعد يظهر بالإنجليزية.
settings-updates = التحديثات
settings-check-updates = البحث عن تحديثات
settings-install-update = تثبيت الإصدار { $version } وإعادة التشغيل
settings-updates-disabled = هذا الإصدار ({ $version }) لا يحدّث نفسه؛ نزّل الإصدارات الجديدة من موقع المشروع.
settings-up-to-date = لديك أحدث إصدار ({ $version }).
theme-system = مثل النظام
theme-light = فاتح
theme-dark = داكن
shortcuts-title = اختصارات لوحة المفاتيح
shortcut-white-wins = فوز الأبيض (1-0)
shortcut-black-wins = فوز الأسود (0-1)
shortcut-draw = تعادل (½-½)
shortcut-white-forfeit = فوز الأبيض بالغياب (+/-)
shortcut-black-forfeit = فوز الأسود بالغياب (-/+)
shortcut-double-forfeit = غياب الطرفين (-/-)
shortcut-clear-result = مسح النتيجة
shortcut-move = التنقل بين الصفوف
shortcut-edit = تعديل الصف المحدد
shortcut-undo = التراجع عن آخر تغيير
shortcut-redo = إعادة
shortcut-pair = قرعة الجولة التالية
shortcut-sections = الانتقال إلى قسم

## شاشة الترحيب

welcome-sample-title = جديد هنا؟ استكشف ببيانات تجريبية
welcome-sample-body = يفتح بطولة سريعة مفتوحة من 25 لاعباً لُعبت منها أربع جولات (الأخيرة أُدخلت نصف نتائجها)، مع تسجيل متأخر وانسحاب وراحات وفئات جوائز وقاعدة لاعبين تجريبية. كل شيء خيالي؛ جرّب ما تشاء وتراجع عما تشاء.
welcome-sample-load = فتح البطولة التجريبية
welcome-tour = جولة في البرنامج
welcome-features = ما يقدمه Chess Organiser
welcome-free = مجاني لكل اتحاد ونادٍ
welcome-shot-rounds = المقابلات والنتائج
welcome-shot-rounds-body = أدخل النتائج من لوحة المفاتيح مباشرة: 1 أو 0 أو 5 لكل طاولة، و+ و− للغياب. ينتقل المؤشر إلى الطاولة التالية، ويمكن التراجع عن أي تغيير.
welcome-shot-players = اللاعبون
welcome-shot-players-body = سجّل اللاعبين يدوياً أو من قائمة التصنيف، مع الألقاب والاتحادات والتصنيفات. تُعالَج التسجيلات المتأخرة والانسحابات والراحات المطلوبة والتوقفات لكل جولة.
welcome-shot-lists = القوائم
welcome-shot-lists-body = قائمة البداية والمقابلات والترتيب، جاهزة للطباعة أو الحفظ بصيغة PDF أو التصدير، باللغة التي تختارها.
welcome-shot-overview = نظرة عامة
welcome-shot-overview-body = اعرف بلمحة أين وصلت البطولة وما الخطوة التالية.
feat-files = ملف واحد لكل بطولة
feat-files-body = يُحفظ بعد كل تغيير ويبقى سليماً عند انقطاع الكهرباء. يعمل دون اتصال؛ انسخه أو أرسله بالبريد كأي مستند. تراجع غير محدود مع سجل كامل.
feat-players = التسجيل
feat-players-body = لاعبون من قائمة فيدي أو القائمة الوطنية، وتسجيلات متأخرة براحات بدون نقاط أو بنصف نقطة، وانسحابات وتوقفات، وترتيب البداية حسب فيدي.
feat-results = إدخال سريع للنتائج
feat-results-body = إدخال النتائج من لوحة المفاتيح، وتصحيح الجولات السابقة، وقرعة يدوية مع فحص التكرار وزملاء النادي والراحات.
feat-lists = القوائم والطباعة
feat-lists-body = المقابلات والترتيب والجداول المتقاطعة وبطاقات اللاعبين والطاولات وقوائم الجوائز للطباعة أو الحفظ بصيغة PDF أو التصدير إلى HTML وCSV وExcel.
feat-trf = تقارير تصنيف فيدي
feat-trf-body = تصدير واستيراد ملفات TRF-2026 لمسؤول التصنيف ولنقل البطولات بين البرامج.
feat-pairing = قرعة فيدي آلية
feat-pairing-body = النظامان الهولندي وبورشتاين مفحوصان مقابل المحرك المعتمد من فيدي، وتسريع باكو، ودوبوف، وليم، ومونراد، والدوري بجداول بيرغر.
feat-teams = الفرق والجوائز
feat-teams-body = السويسري والدوري للفرق مع التشكيلات ومعايير كسر التعادل للفرق؛ وفئات جوائز للسيدات والناشئين وشرائح التصنيف.
feat-ratings = التصنيف والنورمات
feat-ratings-body = جميع معايير كسر التعادل في فيدي، وتغييرات التصنيف وفق فيدي B.02، وفحص نورمات الألقاب.
feat-languages = اللغات والنتائج المباشرة
feat-languages-body = الفرنسية والعربية والسواحيلية والبرتغالية والإسبانية، وموقع نتائج مباشرة لهواتف اللاعبين.
sample-replace-confirm = إغلاق البطولة الحالية وفتح نسخة جديدة من البطولة التجريبية؟ بطولتك محفوظة وتبقى في البطولات الأخيرة.

setup-note-unverified = تتبع المقابلات قواعد هذا النظام مع المعايير المطلقة نفسها المستخدمة في المحرك الهولندي. وعلى خلاف المحركين الهولندي وبورشتاين، لا يوجد برنامج مرجعي معتمد للتحقق منه، لذا راجع المقابلات قبل نشرها.
setup-note-monrad = يقابل نظام مونراد أعلى لاعب بدون مقابلة مع اللاعب التالي الذي لم يواجهه بعد. وهو مخصص لبطولات الأندية، لا للبطولات المحتسبة لتصنيف فيدي.
setup-note-team-swiss = تُجرى قرعة الفرق بالمعايير الهولندية على نقاط المباريات (أو نقاط الأدوار)، ولون الفريق هو لون الطاولة الأولى.
