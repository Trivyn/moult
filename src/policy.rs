//! Curated policy lists. These are hand-written decisions, licensed
//! Apache-2.0 like the rest of the code; they are not WordNet data and the
//! data generator never copies them into the generated tables. The runtime
//! consults them before the generated tables.
//!
//! Every list is sorted by its first element in byte order, and every entry
//! carries a comment giving its reason. The generator checks that each entry
//! still matches a live ambiguity or a real WordNet form and refuses to write
//! tables while any ambiguity is unclassified.
//!
//! Guiding rule: the output must be safe to show a user. Prefer the reading
//! an ordinary modern text means; where no reading is clearly right, leave
//! the word unchanged rather than invent an obscure or wrong one.

use crate::pos::PartOfSpeech;

/// Nouns returned unchanged in both directions: zero plurals ("sheep") and
/// nouns whose `-s` form should not be stripped ("news", "data"). Each must
/// be a lower-case WordNet noun. A word belongs here only when its `-s` form is
/// misleading; "glasses" and "metrics" are deliberately absent, because their
/// singulars are the ordinary words.
pub(crate) static INVARIANT: &[&str] = &[
    // Sorted by byte order. Groups: zero plurals; singulars in -s and
    // plural-only nouns; fields of study in -ics (not classics, graphics,
    // harmonics, mnemonics, polemics, quadratics, statistics, tactics or
    // metrics, whose singulars are common countable nouns); mass nouns.
    "acoustics",       // field of study
    "acrobatics",      // activity
    "advice",          // mass noun
    "aerodynamics",    // field of study
    "aesthetics",      // field of study
    "aircraft",        // zero plural
    "ammunition",      // mass noun
    "athletics",       // field of study and sport
    "baggage",         // mass noun
    "bedding",         // mass noun
    "bellows",         // singular and plural; bellow is a verb
    "bison",           // zero plural
    "calisthenics",    // activity
    "carp",            // zero plural for the fish
    "cattle",          // plural-only herd noun
    "ceramics",        // craft or field
    "chassis",         // singular and plural
    "civics",          // field of study
    "clergy",          // collective; clergies is wrong
    "clothing",        // mass noun
    "cod",             // zero plural for the fish
    "corps",           // singular and plural; corp is not the word
    "crossroads",      // singular and plural; crossroad is rare
    "cybernetics",     // field of study
    "data",            // mass noun; datum is rare
    "deer",            // zero plural
    "dermatoglyphics", // field of study; the singular is not the word
    "dialectics",      // field of study
    "dynamics",        // field of study; also group dynamics
    "economics",       // field of study
    "electronics",     // field of study
    "equipment",       // mass noun
    "ergonomics",      // field of study
    "esthetics",       // field of study
    "ethics",          // field of study
    "evidence",        // mass noun
    "eyeglasses",      // plural-only; eyeglass is a lens
    "feedback",        // mass noun
    "firmware",        // mass noun
    "fish",            // zero plural in ordinary use; WordNet's fishes is the rare species sense
    "forensics",       // field of study
    "furniture",       // mass noun
    "gallows",         // singular and plural
    "genetics",        // field of study
    "gentry",          // collective; gentries is wrong
    "grouse",          // zero plural for the bird
    "gymnastics",      // sport
    "hardware",        // mass noun
    "headquarters",    // singular and plural
    "heroics",         // plural-only; heroic is an adjective
    "homework",        // mass noun
    "hovercraft",      // zero plural
    "hydraulics",      // field of study
    "hysterics",       // plural-only; hysteric is a person
    "informatics",     // field of study
    "information",     // mass noun
    "innings",         // singular and plural
    "isometrics",      // activity
    "jeans",           // plural-only garment; jean is cloth
    "jewellery",       // mass noun, British spelling
    "jewelry",         // mass noun
    "knowledge",       // mass noun
    "kudos",           // singular mass noun
    "linguistics",     // field of study
    "livestock",       // mass noun
    "logistics",       // field of study
    "luggage",         // mass noun
    "machinery",       // mass noun
    "mathematics",     // field of study
    "means",           // singular and plural; mean is a different word
    "mechanics",       // field of study
    "metadata",        // mass noun, like data
    "mews",            // singular and plural; mew is a bird or a cry
    "moose",           // zero plural
    "news",            // singular mass noun
    "offspring",       // zero plural
    "optics",          // field of study
    "pants",           // plural-only garment; pant is a gasp
    "paperwork",       // mass noun
    "personnel",       // mass noun
    "pharmaceutics",   // field of study
    "phonetics",       // field of study
    "physics",         // field of study; physic is a medicine
    "pliers",          // plural-only tool; plier is not the word
    "police",          // plural collective; policy: never singularized
    "politics",        // field of study
    "poultry",         // mass noun
    "pragmatics",      // field of study
    "propaedeutics",   // field of study
    "pyrotechnics",    // field of study and display
    "research",        // mass noun
    "riches",          // plural-only; rich is an adjective
    "robotics",        // field of study
    "rubbish",         // mass noun
    "salmon",          // zero plural
    "scenery",         // mass noun
    "semantics",       // field of study
    "series",          // singular and plural
    "shambles",        // singular; shamble is a verb
    "shears",          // plural-only tool; shear is a verb
    "sheep",           // zero plural
    "shorts",          // plural-only garment; short is a film or an adjective
    "software",        // mass noun
    "spacecraft",      // zero plural
    "species",         // singular and plural; specie means coin
    "statics",         // field of study
    "sunglasses",      // plural-only; sunglass is not the word
    "swine",           // zero plural
    "therapeutics",    // field of study
    "thermodynamics",  // field of study
    "thrips",          // singular and plural for the insect
    "trousers",        // plural-only garment; trouser is rare
    "trout",           // zero plural
    "underwear",       // mass noun
    "vermin",          // collective
    "watercraft",      // zero plural
    "whereabouts",     // plural-only; whereabout is not the word
    "wildlife",        // mass noun
];

/// `(singular, plural)` pairs that WordNet's form list lacks. The singular
/// must be a lower-case WordNet noun with no form at all in the list, so both
/// directions need the entry. WordNet already lists most canonical irregulars
/// (man, child, tooth, foot, goose, mouse, ox, criterion, datum, index,
/// wolf, knife, ...) and the compounds of child, foot, goose, louse, mouse,
/// tooth and wife; they need no entry. Where WordNet lists a form but the
/// modern plural differs, use [`PLURAL_CHOICE`].
pub(crate) static IRREGULAR: &[(&str, &str)] = &[
    // Sorted by byte order. Pairs WordNet's form list lacks entirely: the lemma
    // has no form at all, so neither direction can come from the tables.
    ("alga", "algae"),                 // Latin plural
    ("bacterium", "bacteria"),         // Latin plural
    ("person", "people"),              // WordNet has no form for it
    ("schoolchild", "schoolchildren"), // compound of child; WordNet lists the other -child compounds
];

/// `(input, output)` choices for a singularize input with more than one
/// reading: two lemmas share the plural ("cookies"), a form maps to several
/// lemmas ("leaves"), or an irregular form is a noun itself ("men"). An
/// output equal to the input means "keep unchanged".
pub(crate) static SINGULAR_CHOICE: &[(&str, &str)] = &[
    // Sorted by byte order. Each comment starts with its class: B1 (two
    // lemmas share one regular plural), B2 (one form maps to several
    // lemmas), C (an irregular form that is itself a noun).
    ("adzes", "adze"),                        // B1: adz is the variant spelling
    ("aeries", "aerie"),                      // B1: aery is the variant spelling
    ("annexes", "annex"),                     // B1: annexe is the British variant
    ("anomies", "anomie"),                    // B1: anomy is the variant spelling
    ("apses", "apse"), // B1: apsis is the astronomical term; apse is the architectural word
    ("asbestoses", "asbestosis"), // B1: asbestos is a mass noun with no plural
    ("assegais", "assegai"), // B2: assagai is the variant spelling
    ("aunties", "auntie"), // B1: auntie is the commoner spelling
    ("auspices", "auspices"), // B2: keep: used as a plural-only noun (under the auspices of); the singular is rare
    ("axes", "axis"),         // B2: axis is the common noun; ax or axe are the tool
    ("bacchantes", "bacchante"), // B2: bacchante is the female votary, the plural form in use
    ("bases", "base"), // B2: base is the common noun; basis takes its own bases only in the formal reading
    ("battle-axes", "battle-axe"), // B1: battle-ax is the US variant
    ("beadsmen", "beadsman"), // B2: bedesman is the variant spelling
    ("beanies", "beanie"), // B1: beany is the adjective
    ("bennies", "bennie"), // B1: benny is the variant spelling
    ("bizes", "biz"),  // B1: bize is not a word
    ("blintzes", "blintz"), // B1: blintze is the variant spelling
    ("bogies", "bogie"), // B1: bogy is the variant spelling
    ("bolshies", "bolshie"), // B1: bolshy is the adjective
    ("booties", "bootie"), // B1: booty means plunder; the footwear reading is the plural in use
    ("breakaxes", "breakaxe"), // B1: breakax is the US variant
    ("brethren", "brother"), // C: archaic plural of brother; reduce
    ("broadaxes", "broadaxe"), // B1: broadax is the US variant
    ("burnouses", "burnous"), // B1: burnouse is the variant spelling
    ("caddies", "caddie"), // B1: the golf reading; caddy is the tea box
    ("candelabra", "candelabra"), // C: keep: widely used as a singular; candelabrum is rare
    ("capitula", "capitulum"), // C: Latin plural; reduce
    ("cartouches", "cartouche"), // B1: cartouch is the variant spelling
    ("catties", "catty"), // B1: catty is the unit of weight
    ("caves", "cave"), // B1: cafe is a stem artefact
    ("chasses", "chasses"), // B1: keep: chasse (a liqueur chaser) and chassis (which is its own plural) both read; leave it unchanged
    ("chevaux-de-frise", "chevaux-de-frise"), // C: keep: singular and plural, and rare
    ("cineraria", "cineraria"), // C: keep: the plant's own name, used as a singular
    ("clostridia", "clostridium"), // C: Latin plural; reduce
    ("cocci", "coccus"),    // C: Latin plural; reduce
    ("cohoes", "coho"),     // B1: cohoe is the variant spelling
    ("cola", "cola"),       // C: keep: the drink dominates; colon takes colons
    ("cookies", "cookie"),  // B1: the biscuit and web reading; cooky is the variant
    ("coolies", "coolie"),  // B1: cooly is the variant spelling
    ("corpses", "corpse"),  // B1: corps is singular and plural
    ("corrigenda", "corrigendum"), // C: Latin plural; reduce
    ("cowries", "cowrie"),  // B1: cowry is the variant spelling
    ("crosses", "cross"),   // B1: crosse is the lacrosse stick
    ("cruses", "cruse"),    // B1: crus is the shin
    ("darkies", "darkie"),  // B1: variant spelling of an offensive word; either way the same word
    ("dearies", "dearie"),  // B1: deary is the variant spelling
    ("dickies", "dickie"),  // B1: dicky is the variant spelling
    ("dive", "dive"),       // C: keep: dive is the common noun; diva takes divas
    ("djinn", "djinn"),     // B2: keep: djinn is both the plural and the common singular
    ("doggies", "doggie"),  // B1: doggy is the variant spelling
    ("dogies", "dogie"),    // B1: dogy is the variant spelling
    ("dominoes", "domino"), // C: regular -oes plural; reduce
    ("eclipses", "eclipse"), // B1: eclipsis is a rare grammatical term
    ("ellipses", "ellipses"), // B1: keep: the shapes and the punctuation marks are both common; leave it unchanged
    ("elves", "elf"),         // C: irregular plural; reduce
    ("entozoa", "entozoon"),  // B2: entozoan is the variant
    ("ephemera", "ephemera"), // C: keep: used as a mass or plural noun; ephemeron is rare
    ("epicardia", "epicardium"), // C: Latin plural; reduce
    ("epizoa", "epizoon"),    // B2: epizoan is the variant
    ("eyries", "eyrie"),      // B1: eyry is the variant spelling
    ("faeries", "faerie"),    // B1: faery is the variant spelling
    ("fesses", "fess"),       // B1: fesse is the variant spelling
    ("fives", "five"),        // B1: fife is the flute
    ("floozies", "floozie"),  // B1: floozy is the variant spelling
    ("galactoses", "galactose"), // B1: galactosis is a rare medical term
    ("gendarmeries", "gendarmerie"), // B1: gendarmery is the variant spelling
    ("goonies", "goonie"),    // B1: goony is the variant spelling
    ("graffiti", "graffiti"), // C: keep: used as a mass noun; graffito is rare
    ("grannies", "granny"),   // B1: granny is the commoner spelling
    ("grotesqueries", "grotesquerie"), // B1: grotesquery is the variant spelling
    ("halteres", "haltere"),  // B2: halter takes halters
    ("hankies", "hankie"),    // B1: hanky is the variant spelling
    ("hippies", "hippie"),    // B1: hippy is the variant spelling
    ("hoagies", "hoagie"),    // B1: hoagy is the variant spelling
    ("honkies", "honkie"),    // B1: honky is the variant spelling
    ("hypochondria", "hypochondria"), // C: keep: the condition; hypochondrium is anatomical
    ("junkies", "junkie"),    // B1: junky is the variant spelling
    ("kelpies", "kelpie"),    // B1: kelpy is the adjective
    ("ketoses", "ketose"),    // B1: ketosis is a mass condition
    ("kiddies", "kiddie"),    // B1: kiddy is the variant spelling
    ("knobkerries", "knobkerrie"), // B1: knobkerry is the variant spelling
    ("largesses", "largesse"), // B1: largess is the US variant
    ("leaves", "leaf"),       // B2: the noun reading; leave takes leaves only as a rare noun
    ("lei", "lei"),           // C: keep: the garland; leu takes leus or lei only for the coin
    ("lenses", "lens"),       // B1: lense is not a word
    ("litai", "litas"),       // B2: lit is the old coin
    ("loonies", "loonie"),    // B1: loony is the variant spelling
    ("marquises", "marquis"), // B1: marquise is the feminine title; marquises is the usual plural of marquis
    ("masses", "mass"),       // B1: masse is a billiards shot
    ("meanies", "meanie"),    // B1: meany is the variant spelling
    ("memoranda", "memorandum"), // C: Latin plural; reduce
    ("men", "man"),           // C: irregular plural; reduce
    ("misses", "miss"),       // B1: missis is the dialect form of missus
    ("mollies", "molly"),     // B1: mollie is the variant spelling
    ("monades", "monad"),     // B2: monas is the genus
    ("motives", "motive"),    // B1: motif has its own plural motifs
    ("mycobacteria", "mycobacterium"), // C: Latin plural; reduce
    ("nookies", "nookie"),    // B1: nooky is the variant spelling
    ("oedemata", "oedema"),   // B2: keep the input's British spelling
    ("oesophagi", "oesophagus"), // B2: keep the input's British spelling
    ("oraches", "orache"),    // B1: orach is the variant spelling
    ("organdies", "organdie"), // B1: organdy is the variant spelling
    ("oxen", "ox"),           // C: irregular plural; reduce
    ("panties", "panty"),     // B1: pantie is the variant spelling
    ("paraleipses", "paraleipsis"), // B2: keep the input's spelling
    ("paramecia", "paramecium"), // C: Latin plural; reduce
    ("paraphrases", "paraphrase"), // B1: paraphrasis is a rare form
    ("pavises", "pavise"),    // B1: pavis is the variant spelling
    ("pickaxes", "pickaxe"),  // B1: pickax is the US variant
    ("pinches", "pinch"),     // B1: pinche is not a word
    ("pinkies", "pinkie"),    // B1: pinky is the variant spelling
    ("pixies", "pixie"),      // B1: pixy is the variant spelling
    ("plectra", "plectrum"),  // B2: plectron is the variant
    ("ploughmen", "ploughman"), // B2: keep the input's British spelling
    ("poleaxes", "poleaxe"),  // B1: poleax is the US variant
    ("pooves", "pooves"), // B1: keep: both candidates are slurs or obscure, so leave it unchanged
    ("prothalamia", "prothalamion"), // B2: prothalamion is the title word
    ("pubes", "pubes"),   // C: keep: the region; pubis is anatomical
    ("punkies", "punkie"), // B1: punky is the adjective
    ("quickies", "quickie"), // B1: quicky is the variant spelling
    ("reeves", "reeve"),  // B1: reef takes reefs
    ("reveries", "reverie"), // B1: revery is the variant spelling
    ("reverses", "reverse"), // B1: revers is the lapel
    ("roomies", "roomie"), // B1: roomy is the adjective
    ("saves", "save"),    // B1: safe takes safes
    ("scholia", "scholium"), // C: Latin plural; reduce
    ("senses", "sense"),  // B1: sens is a coin
    ("serves", "serve"),  // B1: serf takes serfs
    ("sharpies", "sharpie"), // B1: sharpy is the variant spelling
    ("shittim", "shittim"), // C: keep: the wood; shittah is rare
    ("shofroth", "shofar"), // B2: shofar is the standard spelling
    ("signore", "signore"), // C: keep: the Italian title; signora takes signore only in Italian
    ("signori", "signore"), // B2: signore is the modern Italian title
    ("sises", "sis"),     // B1: sise is a variant of size
    ("smoothies", "smoothie"), // B1: smoothy is the variant spelling
    ("softies", "softie"), // B1: softy is the variant spelling
    ("soles", "sole"),    // B2: sol is the coin or the colloid
    ("spirilla", "spirillum"), // C: Latin plural; reduce
    ("stamina", "stamina"), // C: keep: the endurance noun dominates; stamen takes stamens
    ("staphylococci", "staphylococcus"), // C: Latin plural; reduce
    ("stigmata", "stigma"), // C: Latin plural; reduce
    ("stogies", "stogie"), // B1: stogy is the variant spelling
    ("streptococci", "streptococcus"), // C: Latin plural; reduce
    ("stymies", "stymie"), // B1: stymy is the variant spelling
    ("synapses", "synapse"), // B1: synapsis is the rare cell-biology term
    ("taeniae", "taenia"), // B2: keep the input's Latin spelling
    ("taxes", "tax"),     // B2: taxis takes taxes only in biology
    ("teeth", "tooth"),   // C: irregular plural; reduce
    ("townies", "townie"), // B1: towny is the variant spelling
    ("trivia", "trivia"), // C: keep: the mass noun dominates; trivium is rare
    ("vases", "vase"),    // B1: vas is the anatomical duct
    ("weirdies", "weirdie"), // B1: weirdy is the variant spelling
];

/// `(lemma, plural)` choices for the plural of a lemma WordNet lists a form
/// for. Either the plural is ambiguous (two or more WordNet forms, self-forms
/// included, or only the doubled-consonant "busses"), or the modern plural
/// differs from the rare, archaic or formal one WordNet lists. The plural is
/// one of the lemma's WordNet forms or its regular rule plural.
pub(crate) static PLURAL_CHOICE: &[(&str, &str)] = &[
    // Sorted by byte order. Two kinds: a lemma with two or more WordNet forms
    // (class D), and a deliberate preference for the modern regular plural over
    // the rare, archaic or mistaken form WordNet lists.
    ("anus", "anuses"), // regular plural; WordNet lists only anus itself
    ("auditorium", "auditoriums"), // auditoria is formal
    ("aura", "auras"),  // aurae is rare
    ("ban", "bans"),    // WordNet's bani is the Romanian coin
    ("bandit", "bandits"), // banditti is archaic
    ("brother", "brothers"), // WordNet lists only brethren, which stays a singular input
    ("bus", "buses"),   // standard plural; busses is the variant
    ("callus", "calluses"), // calli is rare
    ("camera", "cameras"), // camerae is the legal chamber sense
    ("caryopsis", "caryopses"), // standard botanical plural
    ("chapati", "chapatis"), // chapaties is not used
    ("chapatti", "chapattis"), // chapatties is not used
    ("cherub", "cherubs"), // cherubim is the Hebrew plural
    ("chlamys", "chlamyses"), // regular plural; chlamydes is the Greek plural
    ("colon", "colons"), // anatomy and punctuation; colones and cola are not it
    ("coma", "comas"),  // comae is the astronomical sense
    ("consortium", "consortiums"), // consortia is formal
    ("cylix", "cylices"), // standard classical plural
    ("diva", "divas"),  // WordNet's dive is the Italian plural, and a common noun
    ("dogma", "dogmas"), // dogmata is rare
    ("duo", "duos"),    // WordNet's dui is not English
    ("duodenum", "duodena"), // standard medical plural; duodenums is also correct
    ("edema", "edemas"), // regular plural; edemata is formal
    ("esophagus", "esophagi"), // standard medical plural; esophaguses is also correct
    ("fellah", "fellahin"), // standard plural; fellaheen is a variant
    ("fez", "fezzes"),  // standard plural; fezes is wrong
    ("formula", "formulas"), // formulae is formal
    ("forum", "forums"), // fora is formal
    ("gas", "gases"),   // standard plural; gasses is the variant and gas is the singular
    ("genius", "geniuses"), // genii is the spirit sense
    ("genus", "genera"), // standard plural; genuses is rare and genus is the singular
    ("gymnasium", "gymnasiums"), // gymnasia is formal
    ("gynoecium", "gynoecia"), // standard botanical plural
    ("hallah", "hallahs"), // regular plural; hallot is Hebrew
    ("halter", "halters"), // WordNet's halteres is the insect organ
    ("hippopotamus", "hippopotamuses"), // hippopotami is rare
    ("hydra", "hydras"), // hydrae is rare
    ("hymenium", "hymenia"), // standard biological plural
    ("ichthyosaurus", "ichthyosauruses"), // regular plural; ichthyosauri is formal
    ("imago", "imagos"), // regular plural; imagoes and imagines are the others
    ("iris", "irises"), // irides is anatomical
    ("krone", "kroner"), // the Danish and Norwegian plural; kronen is the German
    ("lat", "lats"),    // regular plural; lati is Latvian
    ("litas", "litai"), // the Lithuanian plural; litu is the genitive
    ("loggia", "loggias"), // WordNet's loggie is not English
    ("lustre", "lustres"), // lustra is a five-year period
    ("man-of-war", "men-of-war"), // the compound plural; men-o'-war is a variant
    ("mausoleum", "mausoleums"), // mausolea is formal
    ("monad", "monads"), // WordNet's monades is the Greek plural
    ("muskellunge", "muskellunges"), // WordNet lists a variant spelling as the plural
    ("novella", "novellas"), // WordNet's novelle is Italian
    ("octopus", "octopuses"), // octopi is a mistaken Latin plural
    ("os", "oses"),     // the regular plural; ossa is bone, osar is esker, and ora is mouth
    ("pea", "peas"),    // WordNet's pease is archaic
    ("penis", "penises"), // penes is anatomical
    ("persona", "personas"), // personae is formal
    ("planetarium", "planetariums"), // planetaria is formal
    ("podium", "podiums"), // podia is formal
    ("quiz", "quizzes"), // standard plural; quizes is wrong
    ("real", "reals"),  // regular plural; reales is Spanish
    ("rotl", "rotls"),  // regular plural
    ("sanitarium", "sanitariums"), // sanitaria is formal
    ("schema", "schemas"), // schemata is formal
    ("seraph", "seraphs"), // seraphim is the Hebrew plural
    ("shophar", "shophars"), // regular plural; shophroth is Hebrew
    ("sol", "sols"),    // WordNet's soles is the plural of sole
    ("solarium", "solariums"), // solaria is formal
    ("solo", "solos"),  // soli is musical jargon
    ("soprano", "sopranos"), // soprani is musical jargon
    ("sphinx", "sphinxes"), // sphinges is rare
    ("stadium", "stadiums"), // stadia is formal
    ("starets", "startsy"), // the Russian plural in English use; staretses is not
    ("stigma", "stigmas"), // stigmata stays a singular input
    ("substratum", "substrata"), // standard plural; substratums is rare
    ("swami", "swamis"), // swamies is not used
    ("symposium", "symposiums"), // symposia is formal
    ("tallith", "talliths"), // regular plural; the others are Hebrew
    ("taxi", "taxis"),  // taxies is rare
    ("tenia", "teniae"), // standard medical plural
    ("terrarium", "terrariums"), // terraria is formal
    ("thesaurus", "thesauruses"), // thesauri is formal
    ("torso", "torsos"), // torsi is rare
    ("trauma", "traumas"), // traumata is rare
    ("tuba", "tubas"),  // tubae is rare
    ("turf", "turfs"),  // WordNet's turves is archaic
    ("ultimatum", "ultimatums"), // ultimata is formal
    ("vacuum", "vacuums"), // vacua is formal
    ("vagina", "vaginas"), // vaginae is rare
    ("vertigo", "vertigoes"), // standard plural; vertigines is formal
    ("yeshiva", "yeshivas"), // regular plural; yeshivot is Hebrew
    ("yogi", "yogis"),  // WordNet's yogin is the other singular
    ("zero", "zeros"),  // zeroes is the rarer spelling
];

/// `(lemma, form)` WordNet form pairs dropped entirely: typos, misattached
/// variants, and forms that are not plurals ("cry", "crying").
pub(crate) static EXCLUDED_FORMS: &[(&str, &str)] = &[
    // Sorted by byte order. Groups: typos and misattached variant plurals;
    // forms that are not plurals.
    ("antheridium", "andtheridia"),     // typo for antheridia
    ("argali", "araglis"),              // typo for argalis
    ("bicentenary", "bicennaries"),     // typo for bicentenaries
    ("bicentennial", "bicennaries"),    // typo for bicentennials
    ("brent", "brants"),                // form of the variant brant
    ("cercaria", "cercariiae"),         // typo for cercariae
    ("cry", "crying"),                  // the gerund, not a plural
    ("cry", "cryings"),                 // the gerund, not a plural
    ("diaeresis", "diaerses"),          // typo for diaereses
    ("diathesis", "diathses"),          // typo for diatheses
    ("filaria", "filariiae"),           // typo for filariae
    ("gemsbok", "gemboks"),             // typo for gemsboks
    ("gemsbuck", "gembucks"),           // typo for gemsbucks
    ("glochidium", "glochidcia"),       // typo for glochidia
    ("ichthyosaur", "ichthyosauruses"), // the plural of ichthyosaurus, misattached
    ("madrono", "madornos"),            // typo for madronos
    ("rachis", "rhachises"),            // plural of the variant spelling rhachis, misattached
    ("rebato", "rabatos"),              // plural of the variant rabato, misattached
    ("sahuaro", "saguaros"),            // plural of the variant saguaro, misattached
    ("sente", "listente"),              // typo for lisente
    ("stotinka", "stotkini"),           // typo for stotinki
    ("thyrse", "therses"),              // typo for thyrses
    ("tiro", "tyros"),                  // plural of the variant tyro, misattached
    ("turaco", "touracos"),             // plural of the variant touraco, misattached
];

/// Known nouns ending in "ch" whose plural is plain `-s` because the "ch" is
/// not the sound in "church" (stomachs, not stomaches). Every other noun
/// ending in "ch" (church, march, couch, speech, ...) takes `-es`. Each must be a
/// lower-case WordNet noun ending in "ch" that no WordNet form or other list
/// already settles.
pub(crate) static CH_REGULAR: &[&str] = &[
    // Sorted by byte order.
    "amphibrach",    // metrical foot; Greek, ch is /k/
    "azedarach",     // the chinaberry tree; /k/
    "azederach",     // variant of azedarach
    "conch", // marine snail; -chs and -ches are both standard, -chs is the dictionaries' first
    "coronach", // Gaelic dirge; /k/ or /x/
    "cromlech", // Welsh stone circle; /k/ or /x/
    "dibrach", // metrical foot; Greek, /k/
    "diptych", // two-panel artwork; /k/
    "distich", // couplet; /k/
    "elasmobranch", // shark or ray; -branch is Greek branchia, /k/
    "eparch", // Orthodox bishop; /k/
    "epoch", // era; /k/
    "ethnarch", // ruler of a people; /k/
    "eunuch", // castrated man; /k/
    "exarch", // Byzantine viceroy; /k/
    "hierarch", // high priest; /k/
    "isotach", // line of equal wind speed; /k/
    "lamellibranch", // bivalve mollusc; /k/
    "loch",  // Scottish lake; /x/ or /k/
    "logomach", // one who fights over words; /k/
    "matriarch", // female head of a family; /k/
    "moloch", // the thorny lizard; /k/
    "monarch", // sovereign; /k/
    "nebbech", // Yiddish nebbish; /x/
    "nudibranch", // sea slug; /k/
    "oligarch", // member of a ruling few; /k/
    "patriarch", // male head of a family; /k/
    "pibroch", // Highland bagpipe music; /x/ or /k/
    "rugelach", // pastry; /x/, so -s is the English plural
    "ruggelach", // variant of rugelach
    "sandarach", // resin or its tree; /k/
    "stomach", // organ; /k/
    "sumach", // variant of sumac; /k/
    "symposiarch", // master of a banquet; /k/
    "tach",  // tachometer; /k/
    "taoiseach", // Irish prime minister; /x/
    "tech",  // technician or technology; /k/
    "triptych", // three-panel artwork; /k/
];

/// Known nouns that end in `s` after a consonant or `e` (the shape of a plural)
/// but are real singulars, so pluralize adds `-es` instead of leaving them as
/// the plural they look like. Nearly every other WordNet noun of that shape with
/// no listed plural (thanks, clothes, earnings, 1990s) is already plural. Each
/// must be a lower-case WordNet noun of that shape that no WordNet form or
/// other list already settles.
pub(crate) static S_REGULAR: &[&str] = &[
    // Sorted by byte order.
    "lens",    // optical element; the singular of lenses
    "summons", // court order; the plural is summonses
    "yes",     // an affirmative vote or answer; the plural is yeses
];

/// Known nouns ending in "man" whose plural is `-mans`, not `-men`, because
/// "man" is not the word man. Every other noun ending in "man" (policeman,
/// fireman, woman, ...) takes `-men`. Each must be a lower-case WordNet noun
/// ending in "man".
pub(crate) static MAN_REGULAR: &[&str] = &[
    // Sorted by byte order.
    "brahman",       // Hindu priest or the absolute; a Sanskrit word
    "caiman",        // reptile; Spanish
    "cayman",        // reptile; Spanish
    "ceriman",       // the monstera plant
    "cousin-german", // first cousin; german is "of the same parents"
    "dolman",        // Turkish robe or hussar jacket
    "dragoman",      // interpreter; Arabic
    "hanuman",       // the langur monkey, from the Hindu god
    "hooman",        // informal spelling of human
    "human",         // etymologically unrelated to man
    "life-of-man",   // a plant name
    "liman",         // Turkish estuary or lagoon
    "ottoman",       // sofa, fabric or Turk; Turkish
    "roman",         // the upright typeface
    "saman",         // the rain tree
    "shaman",        // Tungus priest
    "soman",         // the nerve agent
    "talisman",      // charm; Arabic
    "zaman",         // the rain tree
];

/// `(form, part of speech, lemma)`: the lemma that comes first among the
/// candidates of `form` as that part of speech. A row settles an ambiguity the
/// generator reports as L1 (a listed form, or the word itself, competing with
/// another reading) and may also settle an L2 one (rules only). Sorted by
/// `(form, part of speech)`; the part of speech is never `Noun`, whose
/// answers come from the noun lists above. The lemma is one of the form's
/// candidates. Each entry's comment starts with its class tag.
pub(crate) static LEMMA_CHOICE: &[(&str, PartOfSpeech, &str)] = &[
    // Sorted by byte order of the form, then part of speech (verb, adjective,
    // adverb). Groups: past tenses and participles of irregular verbs; spelling
    // variants; comparatives and superlatives; words the rules misread.
    ("abye", PartOfSpeech::Verb, "abye"), // L1: spelling variant of aby; a lemma stays itself
    ("after", PartOfSpeech::Adjective, "after"), // L1: lemma in its own right (the after part); not a comparative of aft
    ("appalled", PartOfSpeech::Verb, "appall"),  // L1: US and common spelling; appal is the variant
    ("appalling", PartOfSpeech::Verb, "appall"), // L1: US and common spelling; appal is the variant
    ("best", PartOfSpeech::Adjective, "good"), // L1: superlative of good; the adjective best (best of all) second
    ("best", PartOfSpeech::Adverb, "well"), // L1: superlative of well; best as its own adverb second
    ("better", PartOfSpeech::Adjective, "good"), // L1: comparative of good; well (in good health) and better itself after
    ("better", PartOfSpeech::Adverb, "well"),    // L1: comparative of well
    ("bigger", PartOfSpeech::Adjective, "big"),  // L1: comparative of big
    ("bittier", PartOfSpeech::Adjective, "bitty"), // L1: bitty is the usual spelling; bittie variant
    ("bittiest", PartOfSpeech::Adjective, "bitty"), // L1: bitty is the usual spelling; bittie variant
    ("bonnier", PartOfSpeech::Adjective, "bonny"), // L1: bonny is the usual spelling; bonnie variant
    ("bonniest", PartOfSpeech::Adjective, "bonny"), // L1: bonny is the usual spelling; bonnie variant
    ("bore", PartOfSpeech::Verb, "bear"), // L1: past of bear (bore a child, bore the cost); bore (tire) second
    ("bound", PartOfSpeech::Verb, "bind"), // L1: past of bind; bound (leap) rare as a verb
    ("burred", PartOfSpeech::Verb, "burr"), // L1: burr is the common verb (burred r); bur (remove burs) rare
    ("burring", PartOfSpeech::Verb, "burr"), // L1: burr is the common verb (burred r); bur (remove burs) rare
    ("caddied", PartOfSpeech::Verb, "caddie"), // L1: caddie is the golf spelling; caddy variant
    ("caddies", PartOfSpeech::Verb, "caddie"), // L1: caddie is the golf spelling; caddy variant
    ("caddying", PartOfSpeech::Verb, "caddie"), // L1: caddie is the golf spelling; caddy variant
    ("cagier", PartOfSpeech::Adjective, "cagey"), // L1: cagey is the more common spelling; cagy variant
    ("cagiest", PartOfSpeech::Adjective, "cagey"), // L1: cagey is the more common spelling; cagy variant
    ("chevied", PartOfSpeech::Verb, "chivy"), // L1: chevy, chivy, chivvy are variants; WordNet lists chivy
    ("chevies", PartOfSpeech::Verb, "chivy"), // L1: chevy, chivy, chivvy are variants; WordNet lists chivy
    ("chevying", PartOfSpeech::Verb, "chivy"), // L1: chevy, chivy, chivvy are variants; WordNet lists chivy
    ("chivvied", PartOfSpeech::Verb, "chivy"), // L1: spelling variant; WordNet lists chivy
    ("chivvies", PartOfSpeech::Verb, "chivy"), // L1: spelling variant; WordNet lists chivy
    ("chivvying", PartOfSpeech::Verb, "chivy"), // L1: spelling variant; WordNet lists chivy
    ("coiffed", PartOfSpeech::Verb, "coif"),   // L1: coif is the verb; coiffe is a variant
    ("coiffing", PartOfSpeech::Verb, "coif"),  // L1: coif is the verb; coiffe is a variant
    ("conned", PartOfSpeech::Verb, "con"),     // L1: con (swindle) common; conn (steer) rare
    ("conning", PartOfSpeech::Verb, "con"),    // L1: con (swindle) common; conn (steer) rare
    ("coquetted", PartOfSpeech::Verb, "coquet"), // L1: WordNet lists coquet; coquette is the variant
    ("coquetting", PartOfSpeech::Verb, "coquet"), // L1: WordNet lists coquet; coquette is the variant
    ("cuter", PartOfSpeech::Adjective, "cute"),   // L1: comparative of cute, not cut plus er
    ("distilled", PartOfSpeech::Verb, "distill"), // L1: US spelling, doubled l as in the input; distil is British
    ("distilling", PartOfSpeech::Verb, "distill"), // L1: US spelling, doubled l as in the input; distil is British
    ("dopier", PartOfSpeech::Adjective, "dopey"), // L1: dopey is the more common spelling; dopy variant
    ("dopiest", PartOfSpeech::Adjective, "dopey"), // L1: dopey is the more common spelling; dopy variant
    ("dying", PartOfSpeech::Verb, "die"),          // L1: gerund of die; dye makes dyeing
    ("earlier", PartOfSpeech::Adjective, "early"), // L1: comparative of early; earlier as a lemma second
    ("earliest", PartOfSpeech::Adjective, "early"), // L1: superlative of early; earliest as a lemma second
    ("eerier", PartOfSpeech::Adjective, "eerie"),   // L1: eerie is the usual spelling; eery variant
    ("eeriest", PartOfSpeech::Adjective, "eerie"),  // L1: eerie is the usual spelling; eery variant
    ("enrolled", PartOfSpeech::Verb, "enroll"), // L1: US spelling, doubled l as in the input; enrol is British
    ("enrolling", PartOfSpeech::Verb, "enroll"), // L1: US spelling, doubled l as in the input; enrol is British
    ("enthralled", PartOfSpeech::Verb, "enthrall"), // L1: US spelling, doubled l as in the input; enthral is British
    ("enthralling", PartOfSpeech::Verb, "enthrall"), // L1: US spelling, doubled l as in the input; enthral is British
    ("farther", PartOfSpeech::Adverb, "far"),        // L1: comparative of far
    ("feed", PartOfSpeech::Verb, "feed"),            // L1: a lemma; fee plus ed is spurious
    ("fell", PartOfSpeech::Verb, "fall"),            // L1: past of fall; fell (cut down) rare
    ("felt", PartOfSpeech::Verb, "feel"),            // L1: past of feel; felt (make felt) rare
    ("fewer", PartOfSpeech::Adjective, "few"),       // L1: comparative of few
    ("fewest", PartOfSpeech::Adjective, "few"),      // L1: superlative of few
    ("finer", PartOfSpeech::Adjective, "fine"),      // L1: comparative of fine
    ("fitter", PartOfSpeech::Adjective, "fit"),      // L1: comparative of fit
    ("found", PartOfSpeech::Verb, "find"),           // L1: past of find; found (establish) second
    ("fulfilled", PartOfSpeech::Verb, "fulfill"), // L1: US spelling, doubled l as in the input; fulfil is British
    ("fulfilling", PartOfSpeech::Verb, "fulfill"), // L1: US spelling, doubled l as in the input; fulfil is British
    ("further", PartOfSpeech::Adverb, "far"), // L1: comparative of far; further as its own adverb second
    ("greater", PartOfSpeech::Adjective, "great"), // L1: comparative of great
    ("greatest", PartOfSpeech::Adjective, "great"), // L1: superlative of great
    ("ground", PartOfSpeech::Verb, "grind"), // L1: past of grind (ground coffee); the base verb ground is rare
    ("healthier", PartOfSpeech::Adjective, "healthy"), // L1: comparative of healthy
    ("higher", PartOfSpeech::Adjective, "high"), // L1: comparative of high
    ("hinder", PartOfSpeech::Adjective, "hinder"), // L1: lemma in its own right (the hinder part); not a comparative of hind
    ("homier", PartOfSpeech::Adjective, "homey"), // L1: homey is the more common spelling; homy variant
    ("homiest", PartOfSpeech::Adjective, "homey"), // L1: homey is the more common spelling; homy variant
    ("installed", PartOfSpeech::Verb, "install"),  // L1: install; instal is the variant
    ("installing", PartOfSpeech::Verb, "install"), // L1: install; instal is the variant
    ("jigged", PartOfSpeech::Verb, "jig"),         // L1: jig; jigg is a variant
    ("jigging", PartOfSpeech::Verb, "jig"),        // L1: jig; jigg is a variant
    ("kookier", PartOfSpeech::Adjective, "kooky"), // L1: kooky is the usual spelling; kookie variant
    ("kookiest", PartOfSpeech::Adjective, "kooky"), // L1: kooky is the usual spelling; kookie variant
    ("laden", PartOfSpeech::Verb, "lade"),          // L1: participle of lade
    ("larger", PartOfSpeech::Adjective, "large"),   // L1: comparative of large
    ("later", PartOfSpeech::Adjective, "late"),     // L1: comparative of late
    ("latest", PartOfSpeech::Adjective, "late"),    // L1: superlative of late
    ("lay", PartOfSpeech::Verb, "lay"), // L1: base verb (lay the table) outweighs lay as past of lie
    ("lesser", PartOfSpeech::Adjective, "lesser"), // L1: lemma in its own right (the lesser evil); not a plain comparative
    ("littler", PartOfSpeech::Adjective, "little"), // L1: comparative of little
    ("liver", PartOfSpeech::Adjective, "liver"), // L1: spurious comparative of live; keep the word
    ("lower", PartOfSpeech::Adjective, "low"),   // L1: comparative of low
    ("lowest", PartOfSpeech::Adjective, "low"),  // L1: superlative of low
    ("matted", PartOfSpeech::Verb, "mat"),       // L1: mat; matte (give a dull finish) rare
    ("matting", PartOfSpeech::Verb, "mat"),      // L1: mat; matte (give a dull finish) rare
    ("modest", PartOfSpeech::Adjective, "modest"), // L1: spurious superlative of mod; keep the word
    ("netted", PartOfSpeech::Verb, "net"),       // L1: net; nett is a variant
    ("netting", PartOfSpeech::Verb, "net"),      // L1: net; nett is a variant
    ("older", PartOfSpeech::Adjective, "old"),   // L1: comparative of old
    ("outer", PartOfSpeech::Adjective, "outer"), // L1: lemma in its own right (outer layer); not a comparative of out
    ("overlay", PartOfSpeech::Verb, "overlay"), // L1: base verb (overlay a map) outweighs overlay as past of overlie
    ("palled", PartOfSpeech::Verb, "pall"),     // L1: pall (grow dull) common; pal (pal up) rare
    ("palling", PartOfSpeech::Verb, "pall"),    // L1: pall (grow dull) common; pal (pal up) rare
    ("programmed", PartOfSpeech::Verb, "program"), // L1: program; programme is the British noun spelling
    ("programmes", PartOfSpeech::Verb, "program"), // L1: spelling variant; WordNet lists program
    ("programming", PartOfSpeech::Verb, "program"), // L1: program; programme is the British noun spelling
    ("putting", PartOfSpeech::Verb, "put"),         // L1: gerund of put; putt (golf) second
    ("rent", PartOfSpeech::Verb, "rent"), // L1: base verb rent outweighs rent as past of rend
    ("rove", PartOfSpeech::Verb, "rove"), // L1: base verb rove (wander); rove as past of reeve is nautical
    ("saw", PartOfSpeech::Verb, "see"),   // L1: past of see; saw (cut with a saw) second
    ("seed", PartOfSpeech::Verb, "seed"), // L1: a lemma; see plus d is spurious
    ("shellacked", PartOfSpeech::Verb, "shellac"), // L1: shellac; shellack is a variant
    ("shellacking", PartOfSpeech::Verb, "shellac"), // L1: shellac; shellack is a variant
    ("sicked", PartOfSpeech::Verb, "sic"), // L1: sic (set a dog on); sick rare as a verb
    ("sicking", PartOfSpeech::Verb, "sic"), // L1: sic (set a dog on); sick rare as a verb
    ("singing", PartOfSpeech::Verb, "sing"), // L1: gerund of sing; singe makes singeing
    ("slew", PartOfSpeech::Verb, "slay"), // L1: past of slay; slew (turn) rare
    ("smaller", PartOfSpeech::Adjective, "small"), // L1: comparative of small
    ("smelt", PartOfSpeech::Verb, "smell"), // L1: past of smell (British); smelt (ore) is the other reading
    ("sonsier", PartOfSpeech::Adjective, "sonsy"), // L1: sonsy is the usual spelling; sonsie variant
    ("sonsiest", PartOfSpeech::Adjective, "sonsy"), // L1: sonsy is the usual spelling; sonsie variant
    ("spat", PartOfSpeech::Verb, "spit"), // L1: past of spit; spat (quarrel) rare as a verb
    ("stymying", PartOfSpeech::Verb, "stymie"), // L1: stymie is the usual spelling; stymy variant
    ("summons", PartOfSpeech::Verb, "summon"), // L1: third person of summon; summons as a verb is rare
    ("swopped", PartOfSpeech::Verb, "swap"),   // L1: swap; swop is the variant
    ("swopping", PartOfSpeech::Verb, "swap"),  // L1: swap; swop is the variant
    ("swops", PartOfSpeech::Verb, "swap"),     // L1: swap; swop is the variant
    ("underlay", PartOfSpeech::Verb, "underlay"), // L1: base verb outweighs underlay as past of underlie
    ("weed", PartOfSpeech::Verb, "weed"),         // L1: a lemma; wee plus d is spurious
    ("whirred", PartOfSpeech::Verb, "whir"),      // L1: spelling variant; WordNet lists whir
    ("whirring", PartOfSpeech::Verb, "whir"),     // L1: spelling variant; WordNet lists whir
    ("whizzed", PartOfSpeech::Verb, "whiz"),      // L1: spelling variant; WordNet lists whiz
    ("whizzes", PartOfSpeech::Verb, "whiz"),      // L1: spelling variant; WordNet lists whiz
    ("whizzing", PartOfSpeech::Verb, "whiz"),     // L1: spelling variant; WordNet lists whiz
    ("worse", PartOfSpeech::Adjective, "bad"),    // L1: comparative of bad
    ("worst", PartOfSpeech::Adjective, "bad"),    // L1: superlative of bad
    ("wound", PartOfSpeech::Verb, "wind"), // L1: past of wind; wound (injure) rare as a base verb
    ("younger", PartOfSpeech::Adjective, "young"), // L1: comparative of young
];

/// `(form, part of speech, lemma)` WordNet form pairs dropped from the lemma
/// form tables: typos, and forms that are not inflections of the lemma. Sorted
/// by `(form, part of speech, lemma)`; the part of speech is never `Noun`
/// (see `EXCLUDED_FORMS` for nouns, whose pairs read `(lemma, form)`).
#[allow(dead_code)] // the generator applies it; the crate reads it only in table tests
pub(crate) static LEMMA_EXCLUDED: &[(&str, PartOfSpeech, &str)] = &[
    // Sorted by byte order of the form, then part of speech, then lemma.
    ("overflown", PartOfSpeech::Verb, "overflow"), // participle of overfly only; overflow makes overflowed
];
