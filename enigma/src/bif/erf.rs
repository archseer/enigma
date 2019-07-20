//! Provides the [error](https://en.wikipedia.org/wiki/Error_function) and
//! related functions
//! Extracted from statrs so we drop the rand dependency.

//! Provides functions that don't have a numerical solution and must
//! be solved computationally (e.g. evaluation of a polynomial)

/// evaluates a polynomial at `z` where `coeff` are the coeffecients
/// to a polynomial of order `k` where `k` is the length of `coeff` and the
/// coeffecient
/// to the `k`th power is the `k`th element in coeff. E.g. [3,-1,2] equates to
/// `2z^2 - z + 3`
///
/// # Remarks
///
/// Returns 0 for a 0 length coefficient slice
pub fn polynomial(z: f64, coeff: &[f64]) -> f64 {
    let n = coeff.len();
    if n == 0 {
        return 0.0;
    }

    unsafe {
        let mut sum = *coeff.get_unchecked(n - 1);
        for i in (0..n - 1).rev() {
            sum *= z;
            sum += *coeff.get_unchecked(i);
        }
        sum
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[cfg(test)]
mod test {
    use std::f64;

    // these tests probably could be more robust
    #[test]
    fn test_polynomial() {
        let empty: [f64; 0] = [];
        assert_eq!(super::polynomial(2.0, &empty), 0.0);

        let zero = [0.0];
        assert_eq!(super::polynomial(2.0, &zero), 0.0);

        let mut coeff = [1.0, 0.0, 5.0];
        assert_eq!(super::polynomial(2.0, &coeff), 21.0);

        coeff = [-5.0, -2.0, 3.0];
        assert_eq!(super::polynomial(2.0, &coeff), 3.0);
        assert_eq!(super::polynomial(-2.0, &coeff), 11.0);

        let large_coeff = [-1.35e3, 2.5e2, 8.0, -4.0, 1e2, 3.0];
        assert_eq!(super::polynomial(5.0, &large_coeff), 71475.0);
        assert_eq!(super::polynomial(-5.0, &large_coeff), 51225.0);

        coeff = [f64::INFINITY, -2.0, 3.0];
        assert_eq!(super::polynomial(2.0, &coeff), f64::INFINITY);
        assert_eq!(super::polynomial(-2.0, &coeff), f64::INFINITY);

        coeff = [f64::NEG_INFINITY, -2.0, 3.0];
        assert_eq!(super::polynomial(2.0, &coeff), f64::NEG_INFINITY);
        assert_eq!(super::polynomial(-2.0, &coeff), f64::NEG_INFINITY);

        coeff = [f64::NAN, -2.0, 3.0];
        assert!(super::polynomial(2.0, &coeff).is_nan());
        assert!(super::polynomial(-2.0, &coeff).is_nan());
    }
}

// use function::evaluate;
use std::f64;

/// `erf` calculates the error function at `x`.
pub fn erf(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x == f64::INFINITY {
        1.0
    } else if x == f64::NEG_INFINITY {
        -1.0
    } else if x == 0.0 {
        0.0
    } else {
        erf_impl(x, false)
    }
}

/// `erf_inv` calculates the inverse error function
/// at `x`.
pub fn erf_inv(x: f64) -> f64 {
    if x == 0.0 {
        0.0
    } else if x >= 1.0 {
        f64::INFINITY
    } else if x <= -1.0 {
        f64::NEG_INFINITY
    } else if x < 0.0 {
        erf_inv_impl(-x, 1.0 + x, -1.0)
    } else {
        erf_inv_impl(x, 1.0 - x, 1.0)
    }
}

/// `erfc` calculates the complementary error function
/// at `x`.
pub fn erfc(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x == f64::INFINITY {
        0.0
    } else if x == f64::NEG_INFINITY {
        2.0
    } else {
        erf_impl(x, true)
    }
}

/// `erfc_inv` calculates the complementary inverse
/// error function at `x`.
pub fn erfc_inv(x: f64) -> f64 {
    if x <= 0.0 {
        f64::INFINITY
    } else if x >= 2.0 {
        f64::NEG_INFINITY
    } else if x > 1.0 {
        erf_inv_impl(-1.0 + x, 2.0 - x, -1.0)
    } else {
        erf_inv_impl(1.0 - x, x, 1.0)
    }
}

// **********************************************************
// ********** Coefficients for erf_impl polynomial **********
// **********************************************************

/// Polynomial coefficients for a numerator of `erf_impl`
/// in the interval [1e-10, 0.5].
const ERF_IMPL_AN: &'static [f64] = &[
    0.00337916709551257388990745,
    -0.00073695653048167948530905,
    -0.374732337392919607868241,
    0.0817442448733587196071743,
    -0.0421089319936548595203468,
    0.0070165709512095756344528,
    -0.00495091255982435110337458,
    0.000871646599037922480317225,
];

/// Polynomial coefficients for a denominator of `erf_impl`
/// in the interval [1e-10, 0.5]
const ERF_IMPL_AD: &'static [f64] = &[
    1.0,
    -0.218088218087924645390535,
    0.412542972725442099083918,
    -0.0841891147873106755410271,
    0.0655338856400241519690695,
    -0.0120019604454941768171266,
    0.00408165558926174048329689,
    -0.000615900721557769691924509,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [0.5, 0.75].
const ERF_IMPL_BN: &'static [f64] = &[
    -0.0361790390718262471360258,
    0.292251883444882683221149,
    0.281447041797604512774415,
    0.125610208862766947294894,
    0.0274135028268930549240776,
    0.00250839672168065762786937,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [0.5, 0.75].
const ERF_IMPL_BD: &'static [f64] = &[
    1.0,
    1.8545005897903486499845,
    1.43575803037831418074962,
    0.582827658753036572454135,
    0.124810476932949746447682,
    0.0113724176546353285778481,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [0.75, 1.25].
const ERF_IMPL_CN: &'static [f64] = &[
    -0.0397876892611136856954425,
    0.153165212467878293257683,
    0.191260295600936245503129,
    0.10276327061989304213645,
    0.029637090615738836726027,
    0.0046093486780275489468812,
    0.000307607820348680180548455,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [0.75, 1.25].
const ERF_IMPL_CD: &'static [f64] = &[
    1.0,
    1.95520072987627704987886,
    1.64762317199384860109595,
    0.768238607022126250082483,
    0.209793185936509782784315,
    0.0319569316899913392596356,
    0.00213363160895785378615014,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [1.25, 2.25].
const ERF_IMPL_DN: &'static [f64] = &[
    -0.0300838560557949717328341,
    0.0538578829844454508530552,
    0.0726211541651914182692959,
    0.0367628469888049348429018,
    0.00964629015572527529605267,
    0.00133453480075291076745275,
    0.778087599782504251917881e-4,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [1.25, 2.25].
const ERF_IMPL_DD: &'static [f64] = &[
    1.0,
    1.75967098147167528287343,
    1.32883571437961120556307,
    0.552528596508757581287907,
    0.133793056941332861912279,
    0.0179509645176280768640766,
    0.00104712440019937356634038,
    -0.106640381820357337177643e-7,
];

///  Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [2.25, 3.5].
const ERF_IMPL_EN: &'static [f64] = &[
    -0.0117907570137227847827732,
    0.014262132090538809896674,
    0.0202234435902960820020765,
    0.00930668299990432009042239,
    0.00213357802422065994322516,
    0.00025022987386460102395382,
    0.120534912219588189822126e-4,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [2.25, 3.5].
const ERF_IMPL_ED: &'static [f64] = &[
    1.0,
    1.50376225203620482047419,
    0.965397786204462896346934,
    0.339265230476796681555511,
    0.0689740649541569716897427,
    0.00771060262491768307365526,
    0.000371421101531069302990367,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [3.5, 5.25].
const ERF_IMPL_FN: &'static [f64] = &[
    -0.00546954795538729307482955,
    0.00404190278731707110245394,
    0.0054963369553161170521356,
    0.00212616472603945399437862,
    0.000394984014495083900689956,
    0.365565477064442377259271e-4,
    0.135485897109932323253786e-5,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [3.5, 5.25].
const ERF_IMPL_FD: &'static [f64] = &[
    1.0,
    1.21019697773630784832251,
    0.620914668221143886601045,
    0.173038430661142762569515,
    0.0276550813773432047594539,
    0.00240625974424309709745382,
    0.891811817251336577241006e-4,
    -0.465528836283382684461025e-11,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [5.25, 8].
const ERF_IMPL_GN: &'static [f64] = &[
    -0.00270722535905778347999196,
    0.0013187563425029400461378,
    0.00119925933261002333923989,
    0.00027849619811344664248235,
    0.267822988218331849989363e-4,
    0.923043672315028197865066e-6,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [5.25, 8].
const ERF_IMPL_GD: &'static [f64] = &[
    1.0,
    0.814632808543141591118279,
    0.268901665856299542168425,
    0.0449877216103041118694989,
    0.00381759663320248459168994,
    0.000131571897888596914350697,
    0.404815359675764138445257e-11,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [8, 11.5].
const ERF_IMPL_HN: &'static [f64] = &[
    -0.00109946720691742196814323,
    0.000406425442750422675169153,
    0.000274499489416900707787024,
    0.465293770646659383436343e-4,
    0.320955425395767463401993e-5,
    0.778286018145020892261936e-7,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [8, 11.5].
const ERF_IMPL_HD: &'static [f64] = &[
    1.0,
    0.588173710611846046373373,
    0.139363331289409746077541,
    0.0166329340417083678763028,
    0.00100023921310234908642639,
    0.24254837521587225125068e-4,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [11.5, 17].
const ERF_IMPL_IN: &'static [f64] = &[
    -0.00056907993601094962855594,
    0.000169498540373762264416984,
    0.518472354581100890120501e-4,
    0.382819312231928859704678e-5,
    0.824989931281894431781794e-7,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [11.5, 17].
const ERF_IMPL_ID: &'static [f64] = &[
    1.0,
    0.339637250051139347430323,
    0.043472647870310663055044,
    0.00248549335224637114641629,
    0.535633305337152900549536e-4,
    -0.117490944405459578783846e-12,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [17, 24].
const ERF_IMPL_JN: &'static [f64] = &[
    -0.000241313599483991337479091,
    0.574224975202501512365975e-4,
    0.115998962927383778460557e-4,
    0.581762134402593739370875e-6,
    0.853971555085673614607418e-8,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [17, 24].
const ERF_IMPL_JD: &'static [f64] = &[
    1.0,
    0.233044138299687841018015,
    0.0204186940546440312625597,
    0.000797185647564398289151125,
    0.117019281670172327758019e-4,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [24, 38].
const ERF_IMPL_KN: &'static [f64] = &[
    -0.000146674699277760365803642,
    0.162666552112280519955647e-4,
    0.269116248509165239294897e-5,
    0.979584479468091935086972e-7,
    0.101994647625723465722285e-8,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [24, 38].
const ERF_IMPL_KD: &'static [f64] = &[
    1.0,
    0.165907812944847226546036,
    0.0103361716191505884359634,
    0.000286593026373868366935721,
    0.298401570840900340874568e-5,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [38, 60].
const ERF_IMPL_LN: &'static [f64] = &[
    -0.583905797629771786720406e-4,
    0.412510325105496173512992e-5,
    0.431790922420250949096906e-6,
    0.993365155590013193345569e-8,
    0.653480510020104699270084e-10,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [38, 60].
const ERF_IMPL_LD: &'static [f64] = &[
    1.0,
    0.105077086072039915406159,
    0.00414278428675475620830226,
    0.726338754644523769144108e-4,
    0.477818471047398785369849e-6,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [60, 85].
const ERF_IMPL_MN: &'static [f64] = &[
    -0.196457797609229579459841e-4,
    0.157243887666800692441195e-5,
    0.543902511192700878690335e-7,
    0.317472492369117710852685e-9,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [60, 85].
const ERF_IMPL_MD: &'static [f64] = &[
    1.0,
    0.052803989240957632204885,
    0.000926876069151753290378112,
    0.541011723226630257077328e-5,
    0.535093845803642394908747e-15,
];

/// Polynomial coefficients for a numerator in `erf_impl`
/// in the interval [85, 110].
const ERF_IMPL_NN: &'static [f64] = &[
    -0.789224703978722689089794e-5,
    0.622088451660986955124162e-6,
    0.145728445676882396797184e-7,
    0.603715505542715364529243e-10,
];

/// Polynomial coefficients for a denominator in `erf_impl`
/// in the interval [85, 110].
const ERF_IMPL_ND: &'static [f64] = &[
    1.0,
    0.0375328846356293715248719,
    0.000467919535974625308126054,
    0.193847039275845656900547e-5,
];

// **********************************************************
// ********** Coefficients for erf_inv_impl polynomial ******
// **********************************************************

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0, 0.5].
const ERF_INV_IMPL_AN: &'static [f64] = &[
    -0.000508781949658280665617,
    -0.00836874819741736770379,
    0.0334806625409744615033,
    -0.0126926147662974029034,
    -0.0365637971411762664006,
    0.0219878681111168899165,
    0.00822687874676915743155,
    -0.00538772965071242932965,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0, 0.5].
const ERF_INV_IMPL_AD: &'static [f64] = &[
    1.0,
    -0.970005043303290640362,
    -1.56574558234175846809,
    1.56221558398423026363,
    0.662328840472002992063,
    -0.71228902341542847553,
    -0.0527396382340099713954,
    0.0795283687341571680018,
    -0.00233393759374190016776,
    0.000886216390456424707504,
];

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0.5, 0.75].
const ERF_INV_IMPL_BN: &'static [f64] = &[
    -0.202433508355938759655,
    0.105264680699391713268,
    8.37050328343119927838,
    17.6447298408374015486,
    -18.8510648058714251895,
    -44.6382324441786960818,
    17.445385985570866523,
    21.1294655448340526258,
    -3.67192254707729348546,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0.5, 0.75].
const ERF_INV_IMPL_BD: &'static [f64] = &[
    1.0,
    6.24264124854247537712,
    3.9713437953343869095,
    -28.6608180499800029974,
    -20.1432634680485188801,
    48.5609213108739935468,
    10.8268667355460159008,
    -22.6436933413139721736,
    1.72114765761200282724,
];

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0.75, 1] with x less than 3.
const ERF_INV_IMPL_CN: &'static [f64] = &[
    -0.131102781679951906451,
    -0.163794047193317060787,
    0.117030156341995252019,
    0.387079738972604337464,
    0.337785538912035898924,
    0.142869534408157156766,
    0.0290157910005329060432,
    0.00214558995388805277169,
    -0.679465575181126350155e-6,
    0.285225331782217055858e-7,
    -0.681149956853776992068e-9,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0.75, 1] with x less than 3.
const ERF_INV_IMPL_CD: &'static [f64] = &[
    1.0,
    3.46625407242567245975,
    5.38168345707006855425,
    4.77846592945843778382,
    2.59301921623620271374,
    0.848854343457902036425,
    0.152264338295331783612,
    0.01105924229346489121,
];

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0.75, 1] with x between 3 and 6.
const ERF_INV_IMPL_DN: &'static [f64] = &[
    -0.0350353787183177984712,
    -0.00222426529213447927281,
    0.0185573306514231072324,
    0.00950804701325919603619,
    0.00187123492819559223345,
    0.000157544617424960554631,
    0.460469890584317994083e-5,
    -0.230404776911882601748e-9,
    0.266339227425782031962e-11,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0.75, 1] with x between 3 and 6.
const ERF_INV_IMPL_DD: &'static [f64] = &[
    1.0,
    1.3653349817554063097,
    0.762059164553623404043,
    0.220091105764131249824,
    0.0341589143670947727934,
    0.00263861676657015992959,
    0.764675292302794483503e-4,
];

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0.75, 1] with x between 6 and 18.
const ERF_INV_IMPL_EN: &'static [f64] = &[
    -0.0167431005076633737133,
    -0.00112951438745580278863,
    0.00105628862152492910091,
    0.000209386317487588078668,
    0.149624783758342370182e-4,
    0.449696789927706453732e-6,
    0.462596163522878599135e-8,
    -0.281128735628831791805e-13,
    0.99055709973310326855e-16,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0.75, 1] with x between 6 and 18.
const ERF_INV_IMPL_ED: &'static [f64] = &[
    1.0,
    0.591429344886417493481,
    0.138151865749083321638,
    0.0160746087093676504695,
    0.000964011807005165528527,
    0.275335474764726041141e-4,
    0.282243172016108031869e-6,
];

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0.75, 1] with x between 18 and 44.
const ERF_INV_IMPL_FN: &'static [f64] = &[
    -0.0024978212791898131227,
    -0.779190719229053954292e-5,
    0.254723037413027451751e-4,
    0.162397777342510920873e-5,
    0.396341011304801168516e-7,
    0.411632831190944208473e-9,
    0.145596286718675035587e-11,
    -0.116765012397184275695e-17,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0.75, 1] with x between 18 and 44.
const ERF_INV_IMPL_FD: &'static [f64] = &[
    1.0,
    0.207123112214422517181,
    0.0169410838120975906478,
    0.000690538265622684595676,
    0.145007359818232637924e-4,
    0.144437756628144157666e-6,
    0.509761276599778486139e-9,
];

/// Polynomial coefficients for a numerator of `erf_inv_impl`
/// in the interval [0.75, 1] with x greater than 44.
const ERF_INV_IMPL_GN: &'static [f64] = &[
    -0.000539042911019078575891,
    -0.28398759004727721098e-6,
    0.899465114892291446442e-6,
    0.229345859265920864296e-7,
    0.225561444863500149219e-9,
    0.947846627503022684216e-12,
    0.135880130108924861008e-14,
    -0.348890393399948882918e-21,
];

/// Polynomial coefficients for a denominator of `erf_inv_impl`
/// in the interval [0.75, 1] with x greater than 44.
const ERF_INV_IMPL_GD: &'static [f64] = &[
    1.0,
    0.0845746234001899436914,
    0.00282092984726264681981,
    0.468292921940894236786e-4,
    0.399968812193862100054e-6,
    0.161809290887904476097e-8,
    0.231558608310259605225e-11,
];

/// `erf_impl` computes the error function at `z`.
/// If `inv` is true, `1 - erf` is calculated as opposed to `erf`
fn erf_impl(z: f64, inv: bool) -> f64 {
    if z < 0.0 {
        if !inv {
            return -erf_impl(-z, false);
        }
        if z < -0.5 {
            return 2.0 - erf_impl(-z, true);
        }
        return 1.0 + erf_impl(-z, false);
    }

    let result = if z < 0.5 {
        if z < 1e-10 {
            z * 1.125 + z * 0.003379167095512573896158903121545171688
        } else {
            z * 1.125 + z * polynomial(z, ERF_IMPL_AN) / polynomial(z, ERF_IMPL_AD)
        }
    } else if z < 110.0 {
        let (r, b) = if z < 0.75 {
            (
                polynomial(z - 0.5, ERF_IMPL_BN) / polynomial(z - 0.5, ERF_IMPL_BD),
                0.3440242112,
            )
        } else if z < 1.25 {
            (
                polynomial(z - 0.75, ERF_IMPL_CN) / polynomial(z - 0.75, ERF_IMPL_CD),
                0.419990927,
            )
        } else if z < 2.25 {
            (
                polynomial(z - 1.25, ERF_IMPL_DN) / polynomial(z - 1.25, ERF_IMPL_DD),
                0.4898625016,
            )
        } else if z < 3.5 {
            (
                polynomial(z - 2.25, ERF_IMPL_EN) / polynomial(z - 2.25, ERF_IMPL_ED),
                0.5317370892,
            )
        } else if z < 5.25 {
            (
                polynomial(z - 3.5, ERF_IMPL_FN) / polynomial(z - 3.5, ERF_IMPL_FD),
                0.5489973426,
            )
        } else if z < 8.0 {
            (
                polynomial(z - 5.25, ERF_IMPL_GN) / polynomial(z - 5.25, ERF_IMPL_GD),
                0.5571740866,
            )
        } else if z < 11.5 {
            (
                polynomial(z - 8.0, ERF_IMPL_HN) / polynomial(z - 8.0, ERF_IMPL_HD),
                0.5609807968,
            )
        } else if z < 17.0 {
            (
                polynomial(z - 11.5, ERF_IMPL_IN) / polynomial(z - 11.5, ERF_IMPL_ID),
                0.5626493692,
            )
        } else if z < 24.0 {
            (
                polynomial(z - 17.0, ERF_IMPL_JN) / polynomial(z - 17.0, ERF_IMPL_JD),
                0.5634598136,
            )
        } else if z < 38.0 {
            (
                polynomial(z - 24.0, ERF_IMPL_KN) / polynomial(z - 24.0, ERF_IMPL_KD),
                0.5638477802,
            )
        } else if z < 60.0 {
            (
                polynomial(z - 38.0, ERF_IMPL_LN) / polynomial(z - 38.0, ERF_IMPL_LD),
                0.5640528202,
            )
        } else if z < 85.0 {
            (
                polynomial(z - 60.0, ERF_IMPL_MN) / polynomial(z - 60.0, ERF_IMPL_MD),
                0.5641309023,
            )
        } else {
            (
                polynomial(z - 85.0, ERF_IMPL_NN) / polynomial(z - 85.0, ERF_IMPL_ND),
                0.5641584396,
            )
        };
        let g = (-z * z).exp() / z;
        g * b + g * r
    } else {
        0.0
    };

    if inv && z >= 0.5 {
        result
    } else if z >= 0.5 {
        1.0 - result
    } else if inv {
        1.0 - result
    } else {
        result
    }
}

// `erf_inv_impl` computes the inverse error function where
// `p`,`q`, and `s` are the first, second, and third intermediate
// parameters respectively
fn erf_inv_impl(p: f64, q: f64, s: f64) -> f64 {
    let result = if p <= 0.5 {
        let y = 0.0891314744949340820313;
        let g = p * (p + 10.0);
        let r = polynomial(p, ERF_INV_IMPL_AN) / polynomial(p, ERF_INV_IMPL_AD);
        g * y + g * r
    } else if q >= 0.25 {
        let y = 2.249481201171875;
        let g = (-2.0 * q.ln()).sqrt();
        let xs = q - 0.25;
        let r = polynomial(xs, ERF_INV_IMPL_BN) / polynomial(xs, ERF_INV_IMPL_BD);
        g / (y + r)
    } else {
        let x = (-q.ln()).sqrt();
        if x < 3.0 {
            let y = 0.807220458984375;
            let xs = x - 1.125;
            let r = polynomial(xs, ERF_INV_IMPL_CN) / polynomial(xs, ERF_INV_IMPL_CD);
            y * x + r * x
        } else if x < 6.0 {
            let y = 0.93995571136474609375;
            let xs = x - 3.0;
            let r = polynomial(xs, ERF_INV_IMPL_DN) / polynomial(xs, ERF_INV_IMPL_DD);
            y * x + r * x
        } else if x < 18.0 {
            let y = 0.98362827301025390625;
            let xs = x - 6.0;
            let r = polynomial(xs, ERF_INV_IMPL_EN) / polynomial(xs, ERF_INV_IMPL_ED);
            y * x + r * x
        } else if x < 44.0 {
            let y = 0.99714565277099609375;
            let xs = x - 18.0;
            let r = polynomial(xs, ERF_INV_IMPL_FN) / polynomial(xs, ERF_INV_IMPL_FD);
            y * x + r * x
        } else {
            let y = 0.99941349029541015625;
            let xs = x - 44.0;
            let r = polynomial(xs, ERF_INV_IMPL_GN) / polynomial(xs, ERF_INV_IMPL_GD);
            y * x + r * x
        }
    };
    s * result
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[cfg(test)]
mod test {
    use std::f64;

    #[test]
    fn test_erf() {
        assert!(super::erf(f64::NAN).is_nan());
        assert_almost_eq!(super::erf(-1.0), -0.84270079294971486934122063508260925929606699796630291, 1e-11);
        assert_eq!(super::erf(0.0), 0.0);
        assert_eq!(super::erf(1e-15), 0.0000000000000011283791670955126615773132947717431253912942469337536);
        assert_eq!(super::erf(0.1), 0.1124629160182848984047122510143040617233925185058162);
        assert_almost_eq!(super::erf(0.2), 0.22270258921047846617645303120925671669511570710081967, 1e-16);
        assert_eq!(super::erf(0.3), 0.32862675945912741618961798531820303325847175931290341);
        assert_eq!(super::erf(0.4), 0.42839235504666847645410962730772853743532927705981257);
        assert_almost_eq!(super::erf(0.5), 0.5204998778130465376827466538919645287364515757579637, 1e-9);
        assert_almost_eq!(super::erf(1.0), 0.84270079294971486934122063508260925929606699796630291, 1e-11);
        assert_almost_eq!(super::erf(1.5), 0.96610514647531072706697626164594785868141047925763678, 1e-11);
        assert_almost_eq!(super::erf(2.0), 0.99532226501895273416206925636725292861089179704006008, 1e-11);
        assert_almost_eq!(super::erf(2.5), 0.99959304798255504106043578426002508727965132259628658, 1e-13);
        assert_almost_eq!(super::erf(3.0), 0.99997790950300141455862722387041767962015229291260075, 1e-11);
        assert_eq!(super::erf(4.0), 0.99999998458274209971998114784032651311595142785474641);
        assert_eq!(super::erf(5.0), 0.99999999999846254020557196514981165651461662110988195);
        assert_eq!(super::erf(6.0), 0.99999999999999997848026328750108688340664960081261537);
        assert_eq!(super::erf(f64::INFINITY), 1.0);
        assert_eq!(super::erf(f64::NEG_INFINITY), -1.0);
    }

    #[test]
    fn test_erfc() {
        assert!(super::erfc(f64::NAN).is_nan());
        assert_almost_eq!(super::erfc(-1.0), 1.8427007929497148693412206350826092592960669979663028, 1e-11);
        assert_eq!(super::erfc(0.0), 1.0);
        assert_almost_eq!(super::erfc(0.1), 0.88753708398171510159528774898569593827660748149418343, 1e-15);
        assert_eq!(super::erfc(0.2), 0.77729741078952153382354696879074328330488429289918085);
        assert_eq!(super::erfc(0.3), 0.67137324054087258381038201468179696674152824068709621);
        assert_almost_eq!(super::erfc(0.4), 0.57160764495333152354589037269227146256467072294018715, 1e-15);
        assert_almost_eq!(super::erfc(0.5), 0.47950012218695346231725334610803547126354842424203654, 1e-9);
        assert_almost_eq!(super::erfc(1.0), 0.15729920705028513065877936491739074070393300203369719, 1e-11);
        assert_almost_eq!(super::erfc(1.5), 0.033894853524689272933023738354052141318589520742363247, 1e-11);
        assert_almost_eq!(super::erfc(2.0), 0.0046777349810472658379307436327470713891082029599399245, 1e-11);
        assert_almost_eq!(super::erfc(2.5), 0.00040695201744495893956421573997491272034867740371342016, 1e-13);
        assert_almost_eq!(super::erfc(3.0), 0.00002209049699858544137277612958232037984770708739924966, 1e-11);
        assert_almost_eq!(super::erfc(4.0), 0.000000015417257900280018852159673486884048572145253589191167, 1e-18);
        assert_almost_eq!(super::erfc(5.0), 0.0000000000015374597944280348501883434853833788901180503147233804, 1e-22);
        assert_almost_eq!(super::erfc(6.0), 2.1519736712498913116593350399187384630477514061688559e-17, 1e-26);
        assert_almost_eq!(super::erfc(10.0), 2.0884875837625447570007862949577886115608181193211634e-45, 1e-55);
        assert_almost_eq!(super::erfc(15.0), 7.2129941724512066665650665586929271099340909298253858e-100, 1e-109);
        assert_almost_eq!(super::erfc(20.0), 5.3958656116079009289349991679053456040882726709236071e-176, 1e-186);
        assert_eq!(super::erfc(30.0), 2.5646562037561116000333972775014471465488897227786155e-393);
        assert_eq!(super::erfc(50.0), 2.0709207788416560484484478751657887929322509209953988e-1088);
        assert_eq!(super::erfc(80.0), 2.3100265595063985852034904366341042118385080919280966e-2782);
        assert_eq!(super::erfc(f64::INFINITY), 0.0);
        assert_eq!(super::erfc(f64::NEG_INFINITY), 2.0);
    }

    #[test]
    fn test_erf_inv() {
        assert!(super::erf_inv(f64::NAN).is_nan());
        assert_eq!(super::erf_inv(-1.0), f64::NEG_INFINITY);
        assert_eq!(super::erf_inv(0.0), 0.0);
        assert_almost_eq!(super::erf_inv(1e-15), 8.86226925452758013649e-16, 1e-30);
        assert_eq!(super::erf_inv(0.1), 0.08885599049425768701574);
        assert_almost_eq!(super::erf_inv(0.2), 0.1791434546212916764927, 1e-15);
        assert_eq!(super::erf_inv(0.3), 0.272462714726754355622);
        assert_eq!(super::erf_inv(0.4), 0.3708071585935579290582);
        assert_eq!(super::erf_inv(0.5), 0.4769362762044698733814);
        assert_eq!(super::erf_inv(1.0), f64::INFINITY);
        assert_eq!(super::erf_inv(f64::INFINITY), f64::INFINITY);
        assert_eq!(super::erf_inv(f64::NEG_INFINITY), f64::NEG_INFINITY);
    }

    #[test]
    fn test_erfc_inv() {
        assert_eq!(super::erfc_inv(0.0), f64::INFINITY);
        assert_almost_eq!(super::erfc_inv(1e-100), 15.065574702593, 1e-11);
        assert_almost_eq!(super::erfc_inv(1e-30), 8.1486162231699, 1e-12);
        assert_almost_eq!(super::erfc_inv(1e-20), 6.6015806223551, 1e-13);
        assert_almost_eq!(super::erfc_inv(1e-10), 4.5728249585449249378479309946884581365517663258840893, 1e-7);
        assert_almost_eq!(super::erfc_inv(1e-5), 3.1234132743415708640270717579666062107939039971365252, 1e-11);
        assert_almost_eq!(super::erfc_inv(0.1), 1.1630871536766741628440954340547000483801487126688552, 1e-14);
        assert_almost_eq!(super::erfc_inv(0.2), 0.90619380243682330953597079527631536107443494091638384, 1e-15);
        assert_eq!(super::erfc_inv(0.5), 0.47693627620446987338141835364313055980896974905947083);
        assert_eq!(super::erfc_inv(1.0), 0.0);
        assert_eq!(super::erfc_inv(1.5), -0.47693627620446987338141835364313055980896974905947083);
        assert_eq!(super::erfc_inv(2.0), f64::NEG_INFINITY);
    }
}

use crate::bif;
use crate::process::RcProcess;
use crate::value::{self, Term};
use crate::vm;

pub fn math_erf_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(badarg!()),
    };
    Ok(Term::from(erf(res)))
}

pub fn math_erfc_1(_vm: &vm::Machine, _process: &RcProcess, args: &[Term]) -> bif::Result {
    let res = match args[0].into_number() {
        Ok(value::Num::Integer(i)) => f64::from(i),
        Ok(value::Num::Float(f)) => f,
        Ok(value::Num::Bignum(..)) => unimplemented!(),
        Err(_) => return Err(badarg!()),
    };
    Ok(Term::from(1.0_f64 - erf(res)))
}
