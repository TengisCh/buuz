// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 the Buuz contributors

// Tiny three-language switch (Mongolian, English, Russian). Mongolian is the default and is also what
// index.html ships, so first paint and search engines see Mongolian.
// English and Russian are opt-in: the toggle button (it cycles mn, en, ru) or ?lang=en or ?lang=ru. The choice is saved.
// We deliberately do NOT follow navigator.language: many Mongolians run
// English-language phones and browsers, and they are this site's audience.

import { MORE, more, type MoreLang } from "./i18n-more.ts";

export type Lang = "mn" | "en" | "ru" | MoreLang;

const mn = {
  title: "Buuz: Латин үсгээр монголоор бич",
  description:
    "Латинаар бичихэд шууд кирилл болно. Үнэгүй, бичсэн зүйл тань хаашаа ч илгээгдэхгүй.",
  tag: "Латинаар бич, кирилл болоод гарчихна.",
  latin: "Латин",
  cyr: "Кирилл",
  clear: "Цэвэрлэх",
  copy: "Хуулах",
  copied: "Хуулагдлаа",
  privacy:
    "Бүх зүйл таны төхөөрөмж дээр ажиллана. Бичсэн зүйл тань хаашаа ч илгээгдэхгүй.",
  howTo: "Яаж бичих вэ?",
  howToBody:
    "<kbd>o</kbd>, <kbd>u</kbd> хоёр эгшиг зохицох ёсоор о/ө, у/ү-гийн алийг нь өөрөө сонгоно. Жишээ нь <kbd>geree</kbd> гэвэл гэрээ, <kbd>gar</kbd> гэвэл гар болно. Заавал ө, ү гаргах бол <kbd>q</kbd>, <kbd>w</kbd>-г (эсвэл <kbd>ö</kbd>, <kbd>ü</kbd>-г) ашиглаарай, үү гаргах бол <kbd>vv</kbd> гэж бичээрэй. Үгийн хойно орох эгшгийг ч харж сонгодог тул <kbd>unee</kbd> гэвэл үнээ болно. Эр үгэнд <kbd>ii</kbd> нь ы болно (<kbd>aaviin</kbd> гэвэл аавын). Заавал ий гаргах бол <kbd>ii'</kbd>.",
  abbreviations: "Товчлол",
  abbreviationsHint:
    "Товчилсон үгийн дараа зай эсвэл цэг, таслал бичихэд бүтнээр нь бичигдэнэ (bn + зай → байна). Энэ сайт дээр тэр зайг арилгавал товчлол буцна. Удахгүй гарах компьютерийн гарт зайны дараа Backspace дарвал буцаана. Унтраавал хэзээ ч өргөжихгүй.",
  completion: "Tab-аар үг гүйцээх",
  completionHint:
    "Үгийн эхийг бичихэд (3-аас дээш үсэг) Buuz хамгийн магадлалтай үгийг ард нь сааралаар харуулна. Санал харагдаж байхад Tab дарвал үг бүтнээрээ орно. Esc дарвал тухайн үгийн санал арилж, дараагийн Tab ердийнхөөрөө (дараагийн талбар руу) шилжинэ. Санал байхгүй үед Tab ердийнхөөрөө ажиллана. Үгийн жагсаалт энэ сайтаас нэг удаа татагдаж, таны төхөөрөмж дээр ажиллана. Бичсэн зүйл тань хаашаа ч илгээгдэхгүй.",
  completionCredit:
    'Үгийн жагсаалт: Монгол Википедиа (хувь нэмэр оруулагчид), <a href="https://creativecommons.org/licenses/by-sa/4.0/" target="_blank" rel="noopener">CC BY-SA 4.0</a>. <a href="https://github.com/TengisCh/buuz/blob/main/data/wordlist/README.md" target="_blank" rel="noopener">Дэлгэрэнгүй</a>.',
  completionHintLabel: "Tab — гүйцээх, Esc — хаах:",
  autocorrect: "Алдаа засах",
  autocorrectHint:
    "Алдаатай бичсэн үгийн дараа зай эсвэл цэг, таслал бичихэд Buuz дээрх ижил үгийн жагсаалтаар алдааг нь засна (delgur + зай → дэлгүүр). Жагсаалтад байгаа үгийг хэзээ ч өөрчлөхгүй. Энэ сайт дээр засварласан бичвэр шууд харагдах бөгөөд Backspace-аар буцаах боломжгүй. Засуулахыг хүсэхгүй бол унтраана уу. Анхнаасаа унтраалттай.",
  doubleV: "vv → үү",
  doubleVHint:
    "Хоёр <kbd>v</kbd> дараалан бичвэл үү болно (svv → сүү). Ганц v нь в хэвээр (avah → авах). Унтраавал vv нь вв болно. Анхнаасаа асаалттай.",
  habitKeys: "ts → ц, zh → ж",
  habitKeysHint:
    "Хоёр үсэг <kbd>ts</kbd> бичвэл ц, <kbd>zh</kbd> бичвэл ж болно (tsagaan → цагаан, gezh → гэж). Үгийн жагсаалт ачаалагдсан үед тс, зх-тэй үгийг тэр хэвээр нь үлдээнэ (hwrtsen → хүртсэн). Жагсаалтгүй үед үргэлж ц, ж болно. Унтраавал ts нь тс, zh нь зх болно. Анхнаасаа асаалттай.",
  wordChoice: "ө, ү-г үгээр таах",
  wordChoiceHint:
    "Үгийн дараа зай эсвэл цэг, таслал бичихэд Buuz о, ө, у, ү үсгийн алийг нь бичихийг үгийн жагсаалтаас харж сонгоно (tov → төв, uvul → өвөл). Мөн урт эгшиг, ь, е-г шаардлагатай үед нөхнө (terer → тэрээр, huuli → хууль, wed → үед). Жагсаалтад байгаа түгээмэл үгийг өөрчлөхгүй (uuriin → уурын). Энэ сайт дээр сонгосон үг шууд харагдах бөгөөд Backspace-аар буцаах боломжгүй. Унтраавал үг бичсэнээрээ үлдэнэ. Анхнаасаа асаалттай.",
  schemeLabel: "Бичих хэл",
  schemeMn: "Монгол (үндсэн)",
  schemeHint: "Өөр кирилл хэлээр латин үсгээр бичих бол тэр хэлээ сонгоорой. Тэгвэл хуудас тухайн хэлний бичих зуршлаар хөрвүүлнэ (zh → ж). Монгол хэл үндсэн бөгөөд Buuz-ийн дүрмээр бичнэ.",
  schemeOnlyMn: "Доорх тохиргоо зөвхөн монгол хэлэнд ажиллана.",
  schemeDraft: "Туршилтын хувилбар: энэ хэлийг эх хэлтэн хараахан шалгаагүй байна.",
  schemeTip: "Латин үсгээр бичихэд кирилл болно. Жишээ нь privet гэвэл привет болно. Нэг апостроф (') бичвэл ь, хоёр апостроф бичвэл ъ болно.",
  mode: "Горим",
  modeSmart: "Ухаалаг",
  modeClassic: "Сонгодог",
  modeHint:
    "Ухаалаг горимд доорх нэмэлт боломжууд ажиллана. Сонгодог горим нь анхны Buuz-ийн дүрмээр яг хэвээр нь, нэмэлт боломжгүйгээр бичнэ.",
  smartVowels: "Эгшиг зохицох",
  smartVowelsHint:
    "Үгийн эгшгийг урагш харж сонгоно (unee → үнээ, aaviin → аавын). Унтраавал анхны Buuz-ийн дүрмээр бичнэ.",
  appTitle: "Ямар ч апп дээр монголоор бич",
  appBody:
    "Buuz удахгүй Windows, macOS-ийн гар болж гарна. Энэ хуудастай яг ижил дүрмээр, хүссэн апп дотроо шууд бичнэ.",
  soon: "удахгүй",
  github: "GitHub дээр нээлттэй эх",
  credit:
    'Buuz 2.0 бол Нямцэрэнгийн Одбаярын бүтээсэн <span class="nw"><a href="https://github.com/odbayar/buuz">Buuz</a>-ийн</span> үргэлжлэл.',
  skip: "Хөрвүүлэгч рүү очих",
  langLabel: "Хэл",
  close: "Хаах",
  settings: "Тохиргоо",
  footProduct: "Бүтээгдэхүүн",
  footWeb: "Вэб хөрвүүлэгч",
  footCommunity: "Нээлттэй эх",
  footSponsor: "Дэмжих",
  footLegal: "Эрх зүй",
  footPrivacy: "Нууцлалын мэдэгдэл",
  footCookies: "Күүкийн бодлого",
  footLicense: "Лиценз",
  tabFeedback: "Санал",
  tabContact: "Холбоо барих",
  fbThanksCat: "Талархал",
  fbCollab: "Хамтран ажиллах",
  contactBody: "Хувийн асуулт, хамтын ажиллагаа, хэвлэлийн хүсэлт байвал шууд имэйлээр бичээрэй. Энэ нь нийтэд харагдахгүй.",
  fbIssue: "Явцыг нь эндээс хараарай →",
  vowels: "Эгшиг",
  yVowels: "Й-тэй эгшиг",
  consonants: "Гийгүүлэгч",
  signs: "Тэмдэг",
  faqTitle: "Түгээмэл асуулт",
  faq0q: "Латинаар кирилл хэрхэн бичих вэ?",
  faq0a: "Дээрх нүдэнд латин үсгээр бичихэд шууд кирилл болно. Жишээ нь <kbd>sain baina uu</kbd> гэж бичвэл «сайн байна уу» болно. Гарсан кирилл текстээ «Хуулах» товчоор хуулаад хүссэн газраа буулгаарай.",
  faq1q: "Латин кирилл хөрвүүлэгч үнэгүй юу?",
  faq1a: "Тийм, үнэгүй бөгөөд нээлттэй эх. Бүртгүүлэх, нэвтрэх шаардлагагүй.",
  faq2q: "Ө, Ү үсгийг яаж бичих вэ?",
  faq2a: "<kbd>q</kbd>, <kbd>ö</kbd> нь ө, <kbd>w</kbd>, <kbd>ü</kbd> нь ү болно. <kbd>vv</kbd> нь үү болно (<kbd>svv</kbd> → сүү). <kbd>o</kbd>, <kbd>u</kbd> нь эгшиг зохицох ёсоор о/ө, у/ү-гийн алийг нь өөрөө сонгоно: <kbd>gar</kbd> → гар, <kbd>geree</kbd> → гэрээ.",
  faq3q: "Бичсэн зүйл маань хаа нэгтээ илгээгдэх үү?",
  faq3a: "Үгүй. Хөрвүүлэлт бүхэлдээ таны төхөөрөмж дээр ажиллана, бичсэн текст хаашаа ч илгээгдэхгүй.",
  faq4q: "Утсан дээр ашиглаж болох уу?",
  faq4a: "Болно. Утасны хөтөч дээр шууд ажиллана. Windows, macOS-ийн гар удахгүй гарна.",
  faq5q: "Уламжлалт монгол бичгээр (босоо) бичиж болох уу?",
  faq5a: "Одоохондоо болохгүй, зөвхөн кирилл үсгээр бичнэ.",
  faq6q: "Buuz гэж юу вэ?",
  faq6a: "Нямцэрэнгийн Одбаярын бүтээсэн Buuz хөтөлбөрийн үргэлжлэл. Дүрэм нь адилхан, одоо нээлттэй эхээр шинэчлэгдэж байна.",
  feedbackTitle: "Санал хүсэлт",
  feedbackBody: "Алдаа олсон уу, эсвэл гоё санаа байна уу? Доор бичээд илгээчихээрэй.",
  feedbackBtn: "Илгээх",
  fbMessage: "Таны санал",
  fbMessagePh: "Юу бодож байна, бидэнд хэлээрэй...",
  fbCategory: "Төрөл",
  fbBug: "Алдаа",
  fbIdea: "Санаа",
  fbOther: "Бусад",
  fbContact: "Холбоо барих (заавал биш)",
  fbContactPh: "Имэйл эсвэл GitHub (заавал биш)",
  fbPublic: "Санал тань GitHub дээр нийтэд харагдана.",
  fbSending: "Илгээж байна...",
  fbThanks: "Баярлалаа! Санал тань илгээгдлээ 🎉",
  fbTooShort: "Арай дэлгэрэнгүй бичээрэй.",
  fbTooLong: "Хэт урт байна, 2000 тэмдэгтээс бага байлгаарай.",
  fbRate: "Хэт олон удаа илгээлээ. Түр хүлээгээд дахин оролдоорой.",
  fbNotReady: "Санал илгээх хэсэг одоохондоо ажиллахгүй байна. GitHub холбоосоор бичээрэй.",
  fbError: "Илгээж чадсангүй. Дахин оролдоорой.",
  fbGithub: "GitHub дээр бичих",
  themeDark: "Харанхуй горим",
  themeLight: "Цайвар горим",
  settingsFile: "Тохиргооны файл",
  settingsFileHint:
    "Тохиргоогоо хуулж эсвэл файл болгон хадгалаад өөр төхөөрөмж дээрээ оруулж болно. Файл тань энэ төхөөрөмжөөс гадагш илгээгдэхгүй.",
  exportCopy: "Тохиргоо хуулах",
  exportDownload: "Файл болгон татах",
  exportCopied: "Тохиргоо хуулагдлаа",
  exportDownloaded: "Файл татагдлаа",
  importTitle: "Тохиргоо оруулах",
  importPh: "Тохиргооны текстээ энд буулгана уу",
  importApply: "Оруулах",
  importFile: "Файл нээх",
  importEmpty: "Эхлээд тохиргооны текстээ буулгана уу.",
  importOk: "Тохиргоо орлоо.",
  importIgnored: "Тохиргоо орлоо. Ойлгоогүй тул алгассан зүйл:",
  copyByHand: "Хуулах эрхийг хөтөч өгсөнгүй. Дээрх нүдэнд тохиргоог гаргалаа: бүгдийг нь сонгоод гараар хуулаарай.",
  classicFeaturesOn: "Сонгодог горимд зарим ухаалаг боломж асаалттай байна.",
  configReset: "Хадгалсан тохиргоо уншигдсангүй тул үндсэн тохиргоог ашиглаж байна. Муу текстийг buuz.config.bad дээр хадгалсан.",
  cfg_too_many_entries: "{line}-р мөр: нэг хэсэгт хэт олон мөр байна (дээд хэмжээ {name}).",
  importNotSaved: "Тохиргоо орлоо, гэхдээ хөтөч хадгалахыг зөвшөөрөхгүй байна. Хуудсыг хаатал л ажиллана.",
  importFileError: "Файлыг уншиж чадсангүй.",
  cfg_missing_version: "{line}-р мөр: файлын эхний мөр заавал buuz_config = 1 байх ёстой.",
  cfg_too_new: "Энэ тохиргоо Buuz-ийн шинэ хувилбарынх ({line}-р хувилбар). Энэ хуудас {name}-р хувилбар хүртэл уншина.",
  cfg_syntax: "{line}-р мөр: ойлгосонгүй. «нэр = утга» эсвэл [хэсэг] гэж бичнэ.",
  cfg_unknown_key: "{line}-р мөр: «{name}» гэсэн нэр байхгүй.",
  cfg_bad_value: "{line}-р мөр: «{name}»-ийн утга буруу байна.",
  cfg_bad_escape: `{line}-р мөр: текст дотор зөвхөн \\", \\\\, \\n гэсэн тэмдэгтийг хэрэглэнэ.`,
  cfg_single_quotes: "{line}-р мөр: ганц хашилт биш, давхар хашилт хэрэглэнэ.",
  cfg_duplicate_key: "{line}-р мөр: «{name}» хоёр удаа бичигдсэн байна.",
  cfg_duplicate_section: "{line}-р мөр: [{name}] хэсэг хоёр удаа бичигдсэн байна.",
  cfg_bad_abbreviation: "{line}-р мөр: «{name}» товчлол буруу байна. Товчлол хоосон, хэт урт байж болохгүй, утга нь нэг мөр байна.",
  cfg_bad_keys: "{line}-р мөр: «{name}» товчлуурын тохиргоо буруу байна. 1-3 товчлуур, кирилл утга хэрэгтэй.",
  cfg_too_large: "Тохиргоо хэт том байна (дээд хэмжээ {name}).",
  cfg_other: "Тохиргоог уншиж чадсангүй.",
  cfg_unknown_option: "{line}-р мөр: «{name}» гэсэн сонголт байхгүй.",
  cfg_unknown_section: "{line}-р мөр: [{name}] гэсэн хэсэг байхгүй.",
  tabTyping: "Бичих",
  tabWords: "Үгс",
  tabRules: "Дүрэм",
  modeShort: "Сонгодог = анхны Buuz, нэмэлтгүй",
  tileHint: "Хавтан дээр дарж асаана, унтраана",
  calTile: "Бичих хэвээ заах",
  calTileSub: "1 минут",
  rulesKeys: "Товчлуурууд",
  rulesFeatures: "Боломж бүрийн тайлбар",
  rulesFull: "Бүрэн дүрэм (GitHub)",
  rulesOther: "Энэ хэлний бүрэн хүснэгт (GitHub)",
  privacyShort: "Бүгд зөвхөн таны төхөөрөмж дээр",
  mapFormat: "Формат",
  mapSmart: "ухаалаг горимд",
  mapHarmony: "эгшиг зохицлоор о/ө, у/ү",
  mapAfterVowel: "эгшгийн дараа",
  mapWordStart: "үгийн эхэнд",
  calButton: "Би яаж бичдэгийг Buuz-д зааж өгөх",
  calHint: "Хэдэн өгүүлбэрийг өөрийн ердийн аргаар латинаар бичихэд Buuz таны товчлуурыг сурна. Бичсэн зүйл тань хадгалагдахгүй, цонхыг хаахад хаягдана.",
  calIntro: "Доорх өгүүлбэрийг өөрийн ердийн аргаар латинаар бичнэ үү. Энд кирилл болгохгүй, бичсэнээр нь харуулна.",
  calProgress: "{n} / {total}",
  calSentenceLabel: "Энэ өгүүлбэрийг латинаар бичнэ үү",
  calBack: "Өмнөх",
  calNext: "Дараах",
  calFinish: "Дүн харах",
  calCancel: "Болих",
  calResultTitle: "Дүн",
  calNone: "Buuz таны бичдэг аргаар аль хэдийн бичиж байна.",
  calSuggestLead: "Та үндсэн дүрмээс өөр товчлуур хэрэглэсэн байна. Хадгалахыг нь сонгоно уу:",
  calSuggestion: "{letter}: та {chunk} гэж бичсэн ({count} удаа)",
  calPair: "{target} хоёуланд хэрэглэнэ.",
  calDisplaces: "{chunk} товчлуур өөр үсэг гаргахаа болино. Өмнөх үсэг: {other}.",
  calTie: "Өөр товчлуурыг мөн адил олон удаа хэрэглэсэн тул хоёулаа санал болгов.",
  calConflict: "{chunk} товчлуурыг {letter} ({count} удаа), {other} ({other_count} удаа) хоёуланд хэрэглэсэн тул санал болгохгүй.",
  calUnsupported: "Үсэг: {letter}. Та {chunk} ({count} удаа) гэж бичсэн. Үүнийг тохиргоонд хийх боломжгүй тул алгаслаа.",
  calUnmatched: "Өгүүлбэр {n} таарсангүй тул алгаслаа.",
  calNotTyped: "Өгүүлбэр {n} бичигдээгүй тул алгаслаа.",
  calSkipped: "Шалгаагүй үсэг: {letters}.",
  calSave: "Хадгалах",
  calDefault: "Үндсэнийг ашиглах",
  calSavedOne: "Тохиргоо хадгалагдлаа. 1 товчлуур нэмэгдлээ.",
  calSaved: "Тохиргоо хадгалагдлаа. {n} товчлуур нэмэгдлээ.",
  calNothing: "Ашиглах боломжтой зүйл бичигдсэнгүй. Дахин оролдоно уу.",
  calRetry: "Дахин оролдох",
  calError: "Тохиргоог шинэчилж чадсангүй.",
  calNotReady: "Хуудас ачаалж дуусаагүй байна. Түр хүлээгээд дахин оролдоно уу.",
  udTitle: "Buuz-ийн сурсан үгс",
  udHint: "Таны нэмсэн үгийг Buuz алдаа гэж засахгүй, Tab-аар гүйцээнэ. Зөвхөн энэ хөтөч дотор хадгалагдана.",
  udEmpty: "Одоогоор үг алга.",
  udAddLabel: "Үг нэмэх (кириллээр)",
  udAdd: "Нэмэх",
  udDelete: "Устгах",
  udDeleteLabel: "{word} үгийг устгах",
  udClear: "Buuz-ийн сурсныг арилгах",
  udClearConfirm: "Нэмсэн бүх үгээ устгах уу? Үүнийг буцаах боломжгүй.",
  udExport: "Файл болгон татах",
  udImport: "Файл нээх",
  udKindAdd: "нэмсэн",
  udKindUse: "хэрэглэсэн",
  udKindReject: "засварыг татгалзсан",
  udCount: "{n} удаа",
  udAdded: "Үг нэмэгдлээ.",
  udRemoved: "Үг устлаа.",
  udCleared: "Бүгдийг арилгалаа.",
  udExported: "Файл татагдлаа",
  udImportOk: "Үгс орлоо.",
  udImportIgnored: "Үгс орлоо. Ойлгоогүй тул алгассан мөр:",
  udNotSaved: "Өөрчлөлт хадгалагдсангүй: хөтөч хадгалахыг зөвшөөрөхгүй байна. Хуудсыг хаатал л ажиллана.",
  cfg_ud_bad_line: "{line}-р мөр: «төрөл, үг, тоо» гэсэн гурван хэсэг табаар тусгаарлагдсан байх ёстой.",
  cfg_ud_unknown_kind: "{line}-р мөр: төрөл нь add, reject эсвэл use байна.",
  cfg_ud_bad_count: "{line}-р мөр: тоо буруу байна.",
  cfg_ud_empty_word: "Үг хоосон байна.",
  cfg_ud_bad_word: "Үгэнд хоосон зай эсвэл удирдах тэмдэгт байж болохгүй.",
  cfg_ud_word_too_long: "Үг хэт урт байна (дээд хэмжээ 64 тэмдэгт).",
  cfg_ud_duplicate: "{line}-р мөр: энэ үг өмнө нь орсон байна.",
  cfg_ud_not_cyrillic: "Зөвхөн кирилл үсэг, «-» болон «'» тэмдэг оруулна уу.",
};

export type Key = keyof typeof mn;

// Same keys as mn, enforced by the type.
const en: Record<Key, string> = {
  title: "Buuz: type Mongolian with a Latin keyboard",
  description:
    "Type Latin letters, get Mongolian Cyrillic instantly. Free, and nothing you type leaves your device.",
  tag: "Type Mongolian with a Latin keyboard.",
  latin: "Latin",
  cyr: "Cyrillic",
  clear: "Clear",
  copy: "Copy",
  copied: "Copied",
  privacy:
    "Everything runs on your device. What you type is never sent anywhere.",
  howTo: "How to type",
  howToBody:
    "<kbd>o</kbd> and <kbd>u</kbd> pick о/ө and у/ү by vowel harmony, so <kbd>geree</kbd> gives гэрээ and <kbd>gar</kbd> gives гар. To force a letter, use <kbd>q</kbd> or <kbd>w</kbd> (or <kbd>ö</kbd> or <kbd>ü</kbd>) for ө and ү, and <kbd>vv</kbd> for үү. Buuz also looks ahead in the word, so <kbd>unee</kbd> gives үнээ, and <kbd>ii</kbd> becomes ы in masculine words (<kbd>aaviin</kbd> gives аавын). To force ий, type <kbd>ii'</kbd>.",
  abbreviations: "Abbreviations",
  abbreviationsHint:
    "A short form expands when you type a space or punctuation after it (bn then a space gives байна). On this website, deleting that space brings the abbreviation back; in the upcoming desktop keyboards, Backspace right after the space undoes it. Turn the checkbox off to never expand.",
  completion: "Finish words with Tab",
  completionHint:
    "When you type the start of a word (more than 2 letters), Buuz shows the most likely word in grey after it. While the hint shows, press Tab to take the whole word. Press Esc to hide the hint for that word, and the next Tab moves focus as usual. With no hint, Tab works as usual. The word list loads once from this site and works on your device. What you type is never sent anywhere.",
  completionCredit:
    'Word list: Mongolian Wikipedia (its contributors), <a href="https://creativecommons.org/licenses/by-sa/4.0/" target="_blank" rel="noopener">CC BY-SA 4.0</a>. <a href="https://github.com/TengisCh/buuz/blob/main/data/wordlist/README.md" target="_blank" rel="noopener">Details</a>.',
  completionHintLabel: "Tab to finish, Esc to dismiss:",
  autocorrect: "Fix typos",
  autocorrectHint:
    "When you type a space or punctuation after a misspelled word, Buuz fixes it with the same word list (delgur then a space gives дэлгүүр). It never changes a word that is in the list. This website shows the fixed text and has no Backspace undo. Turn the checkbox off to stop the fixes. It is off by default.",
  doubleV: "vv → үү",
  doubleVHint:
    "Two <kbd>v</kbd> keys in a row give үү (svv gives сүү). A single v stays в (avah gives авах). Turn the checkbox off to type вв. It is on by default.",
  habitKeys: "ts → ц, zh → ж",
  habitKeysHint:
    "Two keys <kbd>ts</kbd> give ц and <kbd>zh</kbd> gives ж (tsagaan gives цагаан, gezh gives гэж). When the word list is loaded, a word that has тс or зх keeps them (hwrtsen gives хүртсэн). Without the list the keys always give ц and ж. Turn the checkbox off to type тс and зх. It is on by default.",
  wordChoice: "Guess ө, ү from words",
  wordChoiceHint:
    "When you type a space or punctuation after a word, Buuz uses the word list to choose between о, ө, у and ү (tov gives төв, uvul gives өвөл). It also adds a long vowel, ь or е where the list needs them (terer gives тэрээр, huuli gives хууль, wed gives үед). It does not change a common word that is in the list (uuriin gives уурын). This website shows the chosen word and has no Backspace undo. Turn the checkbox off to keep words as typed. It is on by default.",
  schemeLabel: "Language to type",
  schemeMn: "Mongolian (default)",
  schemeHint:
    "Choose another Cyrillic language to type it with Latin letters. The page then uses the typing habits of that language (zh gives ж). Mongolian is the default and uses the Buuz rules.",
  schemeOnlyMn: "The settings below work for Mongolian only.",
  schemeDraft: "Draft: no native speaker has checked this language yet.",
  schemeTip: "Type Latin letters and get Cyrillic. For example privet gives привет. An apostrophe gives ь and two apostrophes give ъ.",
  mode: "Mode",
  modeSmart: "Smart",
  modeClassic: "Classic",
  modeHint:
    "Smart mode adds the features below. Classic types exactly like the original Buuz, with every smart feature off.",
  smartVowels: "Vowel harmony",
  smartVowelsHint:
    "Picks vowels by looking ahead in the word (unee gives үнээ, aaviin gives аавын). Turn off for the original Buuz rules.",
  appTitle: "Type Mongolian in every app",
  appBody:
    "Buuz is also coming as a keyboard for Windows and macOS, with the same rules as this page, so you can type straight into any app.",
  soon: "coming soon",
  github: "Open source on GitHub",
  credit: 'Buuz 2.0 continues <a href="https://github.com/odbayar/buuz">Buuz</a> by Odbayar Nyamtseren.',
  skip: "Skip to the converter",
  langLabel: "Language",
  close: "Close",
  settings: "Settings",
  footProduct: "Product",
  footWeb: "Web converter",
  footCommunity: "Open source",
  footSponsor: "Sponsor",
  footLegal: "Legal",
  footPrivacy: "Privacy notice",
  footCookies: "Cookie policy",
  footLicense: "License",
  tabFeedback: "Feedback",
  tabContact: "Contact",
  fbThanksCat: "Thanks",
  fbCollab: "Collaborate",
  contactBody: "Private questions, collaboration or press? Email us directly. This isn't public.",
  fbIssue: "Follow it here →",
  vowels: "Vowels",
  yVowels: "Y-vowels",
  consonants: "Consonants",
  signs: "Signs",
  faqTitle: "Frequently asked questions",
  faq0q: "How do I type Mongolian Cyrillic with a Latin keyboard?",
  faq0a: "Type Latin letters in the top box and Cyrillic appears right away. For example <kbd>sain baina uu</kbd> becomes «сайн байна уу». Press Copy and paste the result wherever you like.",
  faq1q: "Is the Latin to Cyrillic converter free?",
  faq1a: "Yes, it is free and open source. No sign-up or login.",
  faq2q: "How do I type Ө and Ү?",
  faq2a: "<kbd>q</kbd> or <kbd>ö</kbd> gives ө and <kbd>w</kbd> or <kbd>ü</kbd> gives ү. <kbd>vv</kbd> gives үү (<kbd>svv</kbd> → сүү). <kbd>o</kbd> and <kbd>u</kbd> choose о/ө and у/ү by vowel harmony: <kbd>gar</kbd> → гар, <kbd>geree</kbd> → гэрээ.",
  faq3q: "Is what I type sent anywhere?",
  faq3a: "No. Conversion runs entirely on your device and the text never leaves it.",
  faq4q: "Can I use it on my phone?",
  faq4a: "Yes, it works in your phone's browser. Keyboards for Windows and macOS are coming soon.",
  faq5q: "Can it write the traditional vertical Mongol script?",
  faq5a: "Not yet. For now it writes Cyrillic only.",
  faq6q: "What is Buuz?",
  faq6a: "A continuation of Buuz by Odbayar Nyamtseren, with the same typing rules, now being renewed as open source.",
  feedbackTitle: "Feedback",
  feedbackBody: "Found a bug or have a fun idea? Write it below and send it.",
  feedbackBtn: "Send",
  fbMessage: "Your feedback",
  fbMessagePh: "What's on your mind?",
  fbCategory: "Type",
  fbBug: "Bug",
  fbIdea: "Idea",
  fbOther: "Other",
  fbContact: "Contact (optional)",
  fbContactPh: "Email or GitHub (optional)",
  fbPublic: "Your feedback is posted publicly on GitHub.",
  fbSending: "Sending...",
  fbThanks: "Thank you! Your feedback was sent 🎉",
  fbTooShort: "Please write a little more.",
  fbTooLong: "Too long, please keep it under 2000 characters.",
  fbRate: "Too many messages. Please wait a bit and try again.",
  fbNotReady: "The feedback form isn't switched on yet. Please use the GitHub link instead.",
  fbError: "Couldn't send. Please try again.",
  fbGithub: "Write on GitHub",
  themeDark: "Dark mode",
  themeLight: "Light mode",
  settingsFile: "Settings file",
  settingsFileHint:
    "Copy your settings, or save them as a file, and load them on another device. Your file is never sent away from this device.",
  exportCopy: "Copy settings",
  exportDownload: "Download as a file",
  exportCopied: "Settings copied",
  exportDownloaded: "File downloaded",
  importTitle: "Import settings",
  importPh: "Paste your settings text here",
  importApply: "Import",
  importFile: "Open a file",
  importEmpty: "Paste your settings text first.",
  importOk: "Settings imported.",
  importIgnored: "Settings imported. Not understood, so skipped:",
  copyByHand: "The browser did not allow copying. The settings are in the box above: select all of it and copy it by hand.",
  classicFeaturesOn: "Some smart features are on in Classic mode.",
  configReset: "The saved settings could not be read, so the defaults are in use. The bad text is kept in buuz.config.bad.",
  cfg_too_many_entries: "Line {line}: too many entries in one section (the limit is {name}).",
  importNotSaved: "Settings imported, but the browser does not let the page save them. They last until you close the page.",
  importFileError: "Could not read the file.",
  cfg_missing_version: "Line {line}: the first line of the file must be buuz_config = 1.",
  cfg_too_new: "These settings come from a newer Buuz (version {line}). This page reads up to version {name}.",
  cfg_syntax: 'Line {line}: not understood. Write "name = value" or [section].',
  cfg_unknown_key: 'Line {line}: there is no setting named "{name}".',
  cfg_bad_value: 'Line {line}: the value of "{name}" is not right.',
  cfg_bad_escape: `Line {line}: in text, use only \\", \\\\ and \\n.`,
  cfg_single_quotes: "Line {line}: use double quotes, not single quotes.",
  cfg_duplicate_key: 'Line {line}: "{name}" is written twice.',
  cfg_duplicate_section: "Line {line}: the [{name}] section is written twice.",
  cfg_bad_abbreviation: 'Line {line}: the abbreviation "{name}" is not right. It cannot be empty or too long, and its text must be one line.',
  cfg_bad_keys: 'Line {line}: the key setting "{name}" is not right. It needs 1 to 3 keys and Cyrillic text.',
  cfg_too_large: "The settings are too large (the limit is {name}).",
  cfg_other: "Could not read the settings.",
  cfg_unknown_option: 'Line {line}: there is no option named "{name}".',
  cfg_unknown_section: "Line {line}: there is no section named [{name}].",
  tabTyping: "Typing",
  tabWords: "Words",
  tabRules: "Rules",
  modeShort: "Classic = the original Buuz, no extras",
  tileHint: "Tap a tile to turn it on or off",
  calTile: "Teach Buuz your style",
  calTileSub: "1 minute",
  rulesKeys: "Keys",
  rulesFeatures: "What each switch does",
  rulesFull: "Full rules (GitHub)",
  rulesOther: "Full table for this language (GitHub)",
  privacyShort: "Everything stays on your device",
  mapFormat: "Format",
  mapSmart: "smart mode only",
  mapHarmony: "о/ө, у/ү by vowel harmony",
  mapAfterVowel: "after a vowel",
  mapWordStart: "at the start of a word",
  calButton: "Teach Buuz how you type",
  calHint: "Type a few sentences the way you usually do, and Buuz learns your keys. What you type is not stored and is discarded when the window closes.",
  calIntro: "Type the sentence below in Latin letters, the way you usually do. It is not converted here: you see what you type.",
  calProgress: "{n} / {total}",
  calSentenceLabel: "Type this sentence in Latin letters",
  calBack: "Back",
  calNext: "Next",
  calFinish: "See the result",
  calCancel: "Cancel",
  calResultTitle: "Result",
  calNone: "Buuz already types the way you do.",
  calSuggestLead: "You used keys that differ from the default. Choose which to keep:",
  calSuggestion: "{letter}: you typed {chunk} ({count} times)",
  calPair: "Applies to both {target}.",
  calDisplaces: "{chunk} will stop giving {other}.",
  calTie: "You used another key just as often, so both are suggested.",
  calConflict: "You typed {chunk} for {letter} ({count} times) and for {other} ({other_count} times), so it is not suggested.",
  calUnsupported: "You typed {chunk} for {letter} ({count} times). Settings cannot hold this, so it was skipped.",
  calUnmatched: "Sentence {n} did not match; it was skipped.",
  calNotTyped: "Sentence {n} was not typed; it was skipped.",
  calSkipped: "Letters not tested: {letters}.",
  calSave: "Save",
  calDefault: "Use the default",
  calSavedOne: "Settings saved. 1 key added.",
  calSaved: "Settings saved. {n} keys added.",
  calNothing: "Nothing usable was typed. Try again.",
  calRetry: "Try again",
  calError: "Could not update the settings.",
  calNotReady: "The page is still loading. Wait a moment and try again.",
  udTitle: "Words Buuz learned",
  udHint: "Buuz never corrects a word you add, and Tab can finish it. The list stays in this browser only.",
  udEmpty: "No words yet.",
  udAddLabel: "Add a word (in Cyrillic)",
  udAdd: "Add",
  udDelete: "Delete",
  udDeleteLabel: "Delete {word}",
  udClear: "Clear what Buuz learned",
  udClearConfirm: "Delete all your added words? You cannot undo this.",
  udExport: "Download as a file",
  udImport: "Open a file",
  udKindAdd: "added",
  udKindUse: "used",
  udKindReject: "correction refused",
  udCount: "{n} times",
  udAdded: "Word added.",
  udRemoved: "Word deleted.",
  udCleared: "Everything cleared.",
  udExported: "File downloaded",
  udImportOk: "Words imported.",
  udImportIgnored: "Words imported. These lines were not understood and were skipped:",
  udNotSaved: "The change was not saved: the browser does not allow storage. It works until you close the page.",
  cfg_ud_bad_line: "Line {line}: it needs three parts, kind, word and count, split by tabs.",
  cfg_ud_unknown_kind: "Line {line}: the kind must be add, reject or use.",
  cfg_ud_bad_count: "Line {line}: the count is not valid.",
  cfg_ud_empty_word: "The word is empty.",
  cfg_ud_bad_word: "A word cannot have a space or a control character.",
  cfg_ud_word_too_long: "The word is too long (at most 64 characters).",
  cfg_ud_duplicate: "Line {line}: this word is already there.",
  cfg_ud_not_cyrillic: "Use only Cyrillic letters, \"-\" and \"'\".",
};

// Russian. Same keys as mn, enforced by the type.
const ru: Record<Key, string> = {
  title: "Buuz: пишите по-монгольски латиницей",
  description:
    "Печатайте латиницей и сразу получайте монгольскую кириллицу. Бесплатно, всё остаётся на вашем устройстве.",
  tag: "Печатайте латиницей, получайте кириллицу.",
  latin: "Латиница",
  cyr: "Кириллица",
  clear: "Очистить",
  copy: "Копировать",
  copied: "Скопировано",
  privacy: "Всё работает на вашем устройстве. То, что вы печатаете, никуда не отправляется.",
  howTo: "Как печатать",
  howToBody:
    "<kbd>o</kbd> и <kbd>u</kbd> выбирают о/ө и у/ү по гармонии гласных, поэтому <kbd>geree</kbd> даёт гэрээ, а <kbd>gar</kbd> даёт гар. Чтобы задать букву, используйте <kbd>q</kbd> или <kbd>w</kbd> (или <kbd>ö</kbd>, <kbd>ü</kbd>) для ө и ү и <kbd>vv</kbd> для үү. Buuz смотрит и вперёд по слову: <kbd>unee</kbd> даёт үнээ, а <kbd>ii</kbd> в мужских словах становится ы (<kbd>aaviin</kbd> даёт аавын). Чтобы получить ий, введите <kbd>ii'</kbd>.",
  abbreviations: "Сокращения",
  abbreviationsHint:
    "Сокращение раскрывается, когда после него вы ставите пробел или знак препинания (bn и пробел дают байна). На этом сайте удаление пробела возвращает сокращение; в будущих клавиатурах для компьютера это сделает Backspace сразу после пробела. Снимите флажок, чтобы сокращения не раскрывались.",
  completion: "Дополнять слова клавишей Tab",
  completionHint:
    "Когда вы вводите начало слова (больше 2 букв), Buuz показывает серым самое вероятное слово. Пока подсказка видна, Tab вставляет слово целиком. Esc скрывает подсказку для этого слова, и следующий Tab переводит фокус как обычно. Без подсказки Tab работает как обычно. Список слов загружается один раз с этого сайта и работает на вашем устройстве. То, что вы печатаете, никуда не отправляется.",
  completionCredit:
    'Список слов: монгольская Википедия (её участники), <a href="https://creativecommons.org/licenses/by-sa/4.0/" target="_blank" rel="noopener">CC BY-SA 4.0</a>. <a href="https://github.com/TengisCh/buuz/blob/main/data/wordlist/README.md" target="_blank" rel="noopener">Подробнее</a>.',
  completionHintLabel: "Tab: дополнить, Esc: скрыть:",
  autocorrect: "Исправлять опечатки",
  autocorrectHint:
    "Когда после слова с ошибкой вы ставите пробел или знак препинания, Buuz исправляет его по тому же списку слов (delgur и пробел дают дэлгүүр). Слова из списка он не меняет. На этом сайте виден уже исправленный текст, отмены через Backspace нет. Снимите флажок, чтобы отключить исправления. По умолчанию выключено.",
  doubleV: "vv → үү",
  doubleVHint:
    "Две клавиши <kbd>v</kbd> подряд дают үү (svv даёт сүү). Одна v остаётся в (avah даёт авах). Снимите флажок, чтобы vv давало вв. По умолчанию включено.",
  habitKeys: "ts → ц, zh → ж",
  habitKeysHint:
    "Две клавиши <kbd>ts</kbd> дают ц, а <kbd>zh</kbd> даёт ж (tsagaan даёт цагаан, gezh даёт гэж). Когда список слов загружен, слово с тс или зх сохраняет их (hwrtsen даёт хүртсэн). Без списка клавиши всегда дают ц и ж. Снимите флажок, чтобы вводить тс и зх. По умолчанию включено.",
  mode: "Режим",
  modeSmart: "Умный",
  modeClassic: "Классический",
  modeHint:
    "Умный режим добавляет функции ниже. Классический печатает в точности как оригинальный Buuz, со всеми умными функциями выключенными.",
  smartVowels: "Гармония гласных",
  smartVowelsHint:
    "Выбирает гласные, глядя вперёд по слову (unee даёт үнээ, aaviin даёт аавын). Выключите для правил оригинального Buuz.",
  appTitle: "Пишите по-монгольски в любом приложении",
  appBody:
    "Buuz скоро появится и как клавиатура для Windows и macOS с теми же правилами, что и на этой странице, чтобы печатать прямо в любом приложении.",
  soon: "скоро",
  github: "Открытый код на GitHub",
  credit: 'Buuz 2.0 продолжает <a href="https://github.com/odbayar/buuz">Buuz</a> Odbayar Nyamtseren.',
  skip: "Перейти к конвертеру",
  langLabel: "Язык",
  close: "Закрыть",
  settings: "Настройки",
  footProduct: "Продукт",
  footWeb: "Веб-конвертер",
  footCommunity: "Открытый код",
  footSponsor: "Поддержать",
  footLegal: "Правовая информация",
  footPrivacy: "Уведомление о конфиденциальности",
  footCookies: "Политика cookie",
  footLicense: "Лицензия",
  tabFeedback: "Отзыв",
  tabContact: "Контакты",
  fbThanksCat: "Спасибо",
  fbCollab: "Сотрудничество",
  contactBody: "Личный вопрос, сотрудничество или пресса? Напишите нам напрямую. Это не публикуется.",
  fbIssue: "Следить здесь →",
  vowels: "Гласные",
  yVowels: "Йотированные",
  consonants: "Согласные",
  signs: "Знаки",
  faqTitle: "Частые вопросы",
  faq0q: "Как печатать монгольской кириллицей латинской клавиатурой?",
  faq0a: "Печатайте латиницей в верхнем поле, и кириллица появится сразу. Например, <kbd>sain baina uu</kbd> превращается в «сайн байна уу». Нажмите «Копировать» и вставьте результат куда нужно.",
  faq1q: "Конвертер латиницы в кириллицу бесплатный?",
  faq1a: "Да, он бесплатный и с открытым кодом. Без регистрации и входа.",
  faq2q: "Как напечатать Ө и Ү?",
  faq2a: "<kbd>q</kbd> или <kbd>ö</kbd> даёт ө, а <kbd>w</kbd> или <kbd>ü</kbd> даёт ү. <kbd>vv</kbd> даёт үү (<kbd>svv</kbd> → сүү). <kbd>o</kbd> и <kbd>u</kbd> выбирают о/ө и у/ү по гармонии гласных: <kbd>gar</kbd> → гар, <kbd>geree</kbd> → гэрээ.",
  faq3q: "Отправляется ли то, что я печатаю, куда-нибудь?",
  faq3a: "Нет. Конвертация целиком идёт на вашем устройстве, текст его не покидает.",
  faq4q: "Можно ли пользоваться на телефоне?",
  faq4a: "Да, работает в браузере телефона. Клавиатуры для Windows и macOS скоро появятся.",
  faq5q: "Умеет ли он писать традиционной вертикальной монгольской письменностью?",
  faq5a: "Пока нет. Сейчас только кириллица.",
  faq6q: "Что такое Buuz?",
  faq6a: "Продолжение Buuz Odbayar Nyamtseren с теми же правилами набора, теперь обновляемое как проект с открытым кодом.",
  feedbackTitle: "Отзыв",
  feedbackBody: "Нашли ошибку или есть идея? Напишите ниже и отправьте.",
  feedbackBtn: "Отправить",
  fbMessage: "Ваш отзыв",
  fbMessagePh: "Что у вас на уме?",
  fbCategory: "Тип",
  fbBug: "Ошибка",
  fbIdea: "Идея",
  fbOther: "Другое",
  fbContact: "Контакт (необязательно)",
  fbContactPh: "Email или GitHub (необязательно)",
  fbPublic: "Ваш отзыв публикуется на GitHub.",
  fbSending: "Отправка...",
  fbThanks: "Спасибо! Ваш отзыв отправлен 🎉",
  fbTooShort: "Напишите, пожалуйста, чуть подробнее.",
  fbTooLong: "Слишком длинно, уложитесь в 2000 символов.",
  fbRate: "Слишком много сообщений. Подождите немного и попробуйте снова.",
  fbNotReady: "Форма отзывов ещё не включена. Воспользуйтесь ссылкой на GitHub.",
  fbError: "Не удалось отправить. Попробуйте ещё раз.",
  fbGithub: "Написать на GitHub",
  themeDark: "Тёмная тема",
  themeLight: "Светлая тема",
  settingsFile: "Файл настроек",
  settingsFileHint:
    "Скопируйте настройки или сохраните их в файл и загрузите на другом устройстве. Ваш файл никогда не уходит с этого устройства.",
  exportCopy: "Скопировать настройки",
  exportDownload: "Скачать файлом",
  exportCopied: "Настройки скопированы",
  exportDownloaded: "Файл скачан",
  importTitle: "Импорт настроек",
  importPh: "Вставьте сюда текст настроек",
  importApply: "Импортировать",
  importFile: "Открыть файл",
  importEmpty: "Сначала вставьте текст настроек.",
  importOk: "Настройки импортированы.",
  importIgnored: "Настройки импортированы. Не распознано и пропущено:",
  copyByHand: "Браузер не разрешил копирование. Настройки в поле выше: выделите всё и скопируйте вручную.",
  classicFeaturesOn: "В классическом режиме включены некоторые умные функции.",
  configReset: "Сохранённые настройки не удалось прочитать, поэтому используются настройки по умолчанию. Неверный текст сохранён в buuz.config.bad.",
  cfg_too_many_entries: "Строка {line}: слишком много записей в одном разделе (предел: {name}).",
  importNotSaved: "Настройки импортированы, но браузер не позволяет странице их сохранить. Они действуют, пока вы не закроете страницу.",
  importFileError: "Не удалось прочитать файл.",
  cfg_missing_version: "Строка {line}: первая строка файла должна быть buuz_config = 1.",
  cfg_too_new: "Эти настройки из более новой версии Buuz (версия {line}). Страница читает до версии {name}.",
  cfg_syntax: 'Строка {line}: не распознано. Пишите "имя = значение" или [раздел].',
  cfg_unknown_key: 'Строка {line}: нет настройки с именем "{name}".',
  cfg_bad_value: 'Строка {line}: неверное значение "{name}".',
  cfg_bad_escape: `Строка {line}: в тексте допустимы только \\", \\\\ и \\n.`,
  cfg_single_quotes: "Строка {line}: используйте двойные кавычки, а не одинарные.",
  cfg_duplicate_key: 'Строка {line}: "{name}" записано дважды.',
  cfg_duplicate_section: "Строка {line}: раздел [{name}] записан дважды.",
  cfg_bad_abbreviation: 'Строка {line}: сокращение "{name}" неверно. Оно не может быть пустым или слишком длинным, а его текст должен быть в одну строку.',
  cfg_bad_keys: 'Строка {line}: настройка клавиш "{name}" неверна. Нужны от 1 до 3 клавиш и текст кириллицей.',
  cfg_too_large: "Настройки слишком велики (предел: {name}).",
  cfg_other: "Не удалось прочитать настройки.",
  cfg_unknown_option: 'Строка {line}: нет параметра с именем "{name}".',
  cfg_unknown_section: "Строка {line}: нет раздела с именем [{name}].",
  tabTyping: "Набор",
  tabWords: "Слова",
  tabRules: "Правила",
  modeShort: "Классический = оригинальный Buuz без добавок",
  tileHint: "Нажмите на плитку, чтобы включить или выключить",
  calTile: "Научить Buuz",
  calTileSub: "1 минута",
  rulesKeys: "Клавиши",
  rulesFeatures: "Что делает каждый переключатель",
  rulesFull: "Все правила (GitHub)",
  rulesOther: "Полная таблица для этого языка (GitHub)",
  privacyShort: "Всё остаётся на вашем устройстве",
  mapFormat: "Формат",
  mapSmart: "только в умном режиме",
  mapHarmony: "о/ө, у/ү по гармонии гласных",
  mapAfterVowel: "после гласной",
  mapWordStart: "в начале слова",
  calButton: "Научить Buuz вашей манере набора",
  calHint: "Напечатайте несколько предложений так, как вы обычно это делаете, и Buuz выучит ваши клавиши. Напечатанное не сохраняется и стирается при закрытии окна.",
  calIntro: "Напечатайте предложение ниже латиницей, как вы обычно это делаете. Здесь оно не конвертируется: вы видите то, что печатаете.",
  calProgress: "{n} / {total}",
  calSentenceLabel: "Напечатайте это предложение латиницей",
  calBack: "Назад",
  calNext: "Далее",
  calFinish: "Показать результат",
  calCancel: "Отмена",
  calResultTitle: "Результат",
  calNone: "Buuz уже печатает так же, как вы.",
  calSuggestLead: "Вы использовали клавиши, отличные от стандартных. Выберите, какие оставить:",
  calSuggestion: "{letter}: вы напечатали {chunk} ({count} раз)",
  calPair: "Относится к обоим: {target}.",
  calDisplaces: "{chunk} перестанет давать {other}.",
  calTie: "Вы использовали другую клавишу так же часто, поэтому предложены обе.",
  calConflict: "Вы напечатали {chunk} для {letter} ({count} раз) и для {other} ({other_count} раз), поэтому это не предлагается.",
  calUnsupported: "Вы напечатали {chunk} для {letter} ({count} раз). Настройки не могут это хранить, поэтому пропущено.",
  calUnmatched: "Предложение {n} не совпало и пропущено.",
  calNotTyped: "Предложение {n} не напечатано и пропущено.",
  calSkipped: "Буквы не проверены: {letters}.",
  calSave: "Сохранить",
  calDefault: "Оставить по умолчанию",
  calSavedOne: "Настройки сохранены. Добавлена 1 клавиша.",
  calSaved: "Настройки сохранены. Добавлено клавиш: {n}.",
  calNothing: "Ничего подходящего не напечатано. Попробуйте ещё раз.",
  calRetry: "Попробовать снова",
  calError: "Не удалось обновить настройки.",
  calNotReady: "Страница ещё загружается. Подождите немного и попробуйте снова.",
  udTitle: "Слова, которые выучил Buuz",
  udHint: "Buuz не исправляет добавленные слова, а Tab может их дополнить. Список хранится только в этом браузере.",
  udEmpty: "Слов пока нет.",
  udAddLabel: "Добавить слово (кириллицей)",
  udAdd: "Добавить",
  udDelete: "Удалить",
  udDeleteLabel: "Удалить {word}",
  udClear: "Очистить выученное Buuz",
  udClearConfirm: "Удалить все добавленные слова? Это нельзя отменить.",
  udExport: "Скачать файлом",
  udImport: "Открыть файл",
  udKindAdd: "добавлено",
  udKindUse: "использовано",
  udKindReject: "исправление отклонено",
  udCount: "{n} раз",
  udAdded: "Слово добавлено.",
  udRemoved: "Слово удалено.",
  udCleared: "Всё очищено.",
  udExported: "Файл скачан",
  udImportOk: "Слова импортированы.",
  udImportIgnored: "Слова импортированы. Эти строки не распознаны и пропущены:",
  udNotSaved: "Изменение не сохранено: браузер не разрешает хранение. Оно действует, пока вы не закроете страницу.",
  cfg_ud_bad_line: "Строка {line}: нужны три части (вид, слово и число), разделённые табуляцией.",
  cfg_ud_unknown_kind: "Строка {line}: вид должен быть add, reject или use.",
  cfg_ud_bad_count: "Строка {line}: число недопустимо.",
  cfg_ud_empty_word: "Слово пустое.",
  cfg_ud_bad_word: "В слове не может быть пробела или управляющего символа.",
  cfg_ud_word_too_long: "Слово слишком длинное (не более 64 символов).",
  cfg_ud_duplicate: "Строка {line}: это слово уже есть.",
  cfg_ud_not_cyrillic: "Используйте только кириллицу, \"-\" и \"'\".",
  wordChoice: "Угадывать ө, ү по словам",
  wordChoiceHint:
    "Когда после слова вы ставите пробел или знак препинания, Buuz по списку слов выбирает между о, ө, у, ү (tov даёт төв, uvul даёт өвөл). Он также добавляет долгие гласные, ь и е, когда нужно (terer даёт тэрээр, huuli даёт хууль, wed даёт үед). Частые слова из списка он не меняет (uuriin даёт уурын). На этом сайте виден уже выбранный вариант, отмены через Backspace нет. Если выключить, слово остаётся как набрано. По умолчанию включено.",
  schemeLabel: "Язык набора",
  schemeMn: "Монгольский (по умолчанию)",
  schemeHint:
    "Выберите другой язык на кириллице, чтобы печатать на нём латиницей. Страница применит привычки набора этого языка (zh даёт ж). Монгольский используется по умолчанию и работает по правилам Buuz.",
  schemeOnlyMn: "Настройки ниже действуют только для монгольского языка.",
  schemeDraft: "Черновик: носитель языка эту схему ещё не проверил.",
  schemeTip:
    "Печатайте латиницей и получайте кириллицу. Например, privet даёт привет. Один апостроф даёт ь, два апострофа дают ъ.",
};

// mn, en and ru have every key. The other languages have the keys of the main
// page and fall back to English for the rest (see t()).
export const strings: Record<Lang, Partial<Record<Key, string>>> = { mn, en, ru, ...more };

/** The languages the language button cycles through: the fully translated ones. */
const ORDER: Lang[] = ["mn", "en", "ru"];
export const ALL_LANGS: Lang[] = [...ORDER, ...MORE];
const isLang = (v: unknown): v is Lang => ALL_LANGS.includes(v as Lang);

/** A parse error or warning from the config, in the current language. */
export function issueText(issue: { code: string; number: number; detail: string }): string {
  const key = `cfg_${issue.code}` as Key;
  const template = strings[current][key] ?? en[key] ?? en.cfg_other;
  return template.replace("{line}", () => String(issue.number)).replace("{name}", () => issue.detail);
}

/** A string with `{name}` placeholders filled in (no markup, so use textContent). */
export function fmt(key: Key, vars: Record<string, string | number>): string {
  return t(key).replace(/\{(\w+)\}/g, (m, name: string) => (name in vars ? String(vars[name]) : m));
}

const STORE = "buuz.lang";
const ORIGIN = "https://buuz.thetengis.com";

/** One URL for each page language, so search engines can index each one. */
export const PATHS = Object.fromEntries(ALL_LANGS.map((l) => [l, l === "mn" ? "/" : `/${l}/`])) as Record<Lang, string>;

/** The page language a path belongs to, or null (the Mongolian root is "/"). */
export function langOfPath(path: string): Lang | null {
  for (const l of ALL_LANGS) if (l !== "mn" && (path === PATHS[l] || path === PATHS[l].slice(0, -1) || path.startsWith(PATHS[l] + "index"))) return l;
  return path === "/" || path === "/index.html" ? "mn" : null;
}
let current: Lang = "mn";

/** The language the page shows now. */
export function lang(): Lang {
  return current;
}

export function t(key: Key): string {
  return strings[current][key] ?? en[key];
}

function save(lang: Lang): void {
  try {
    localStorage.setItem(STORE, lang);
  } catch {
    /* private mode etc.: just don't remember */
  }
}

function initial(): Lang {
  // The URL decides, so each language has its own page for search engines.
  const fromPath = langOfPath(location.pathname);
  if (fromPath && fromPath !== "mn") return fromPath;
  const q = new URLSearchParams(location.search).get("lang");
  if (isLang(q)) {
    save(q);
    return q;
  }
  try {
    const s = localStorage.getItem(STORE);
    if (isLang(s)) return s;
  } catch {
    /* ignore */
  }
  return "mn";
}

export function apply(lang: Lang): void {
  current = lang;
  const root = document.documentElement;
  root.lang = lang;
  document.title = t("title");
  const canon = document.querySelector<HTMLLinkElement>('link[rel="canonical"]');
  if (canon && Object.values(PATHS).includes(new URL(canon.href).pathname)) canon.href = ORIGIN + PATHS[lang];
  document
    .querySelector('meta[name="description"]')
    ?.setAttribute("content", t("description"));
  // Text-only keys use textContent; keys with markup (<kbd>, links) are ours.
  for (const el of document.querySelectorAll<HTMLElement>("[data-i18n]")) {
    el.textContent = t(el.dataset.i18n as Key);
  }
  for (const el of document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>("[data-i18n-ph]")) {
    el.placeholder = t(el.dataset.i18nPh as Key);
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-i18n-aria]")) {
    const label = t(el.dataset.i18nAria as Key);
    el.setAttribute("aria-label", label);
    el.title = label;
  }
  for (const el of document.querySelectorAll<HTMLElement>("[data-i18n-html]")) {
    el.innerHTML = t(el.dataset.i18nHtml as Key);
  }
  for (const a of document.querySelectorAll<HTMLAnchorElement>("#langs a")) {
    if (a.hreflang === lang) a.setAttribute("aria-current", "true");
    else a.removeAttribute("aria-current");
  }
  const sel = document.getElementById("lang");
  if (sel instanceof HTMLSelectElement) sel.value = lang;
  document.dispatchEvent(new Event("buuz:lang"));
}

export function setupLang(): void {
  const lang = initial();
  if (lang !== "mn") apply(lang); // the HTML of "/" is Mongolian
  else current = "mn";
  // A saved choice or ?lang= on the root page moves to that language's own URL.
  if (langOfPath(location.pathname) === "mn" && lang !== "mn") history.replaceState(null, "", PATHS[lang]);
  document.getElementById("lang")?.addEventListener("change", (e) => {
    const next = (e.target as HTMLSelectElement).value;
    if (!isLang(next) || next === current) return;
    save(next);
    apply(next);
    history.pushState(null, "", PATHS[next]);
    document.dispatchEvent(new CustomEvent("buuz:lang-changed", { detail: next }));
  });
  // Back and forward move between the language URLs.
  window.addEventListener("popstate", () => {
    const l = langOfPath(location.pathname);
    if (l && l !== current) {
      apply(l);
      document.dispatchEvent(new CustomEvent("buuz:lang-changed", { detail: l }));
    }
  });
}
