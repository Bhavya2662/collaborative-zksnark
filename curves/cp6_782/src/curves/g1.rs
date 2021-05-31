use ark_ec::{
    models::{ModelParameters, SWModelParameters},
    short_weierstrass_jacobian::{GroupAffine, GroupProjective},
};
use ark_ff::field_new;

use crate::{Fq, Fr};

pub type G1Affine = GroupAffine<Parameters>;
pub type G1Projective = GroupProjective<Parameters>;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Parameters;

impl ModelParameters for Parameters {
    type BaseField = Fq;
    type ScalarField = Fr;
}

impl SWModelParameters for Parameters {
    /// COEFF_A = 5
    #[rustfmt::skip]
    const COEFF_A: Fq = field_new!(Fq, "5");

    /// COEFF_B = 17764315118651679038286329069295091506801468118146712649886336045535808055361274148466772191243305528312843236347777260247138934336850548243151534538734724191505953341403463040067571652261229308333392040104884438208594329793895206056414
    #[rustfmt::skip]
    const COEFF_B: Fq = field_new!(Fq, "17764315118651679038286329069295091506801468118146712649886336045535808055361274148466772191243305528312843236347777260247138934336850548243151534538734724191505953341403463040067571652261229308333392040104884438208594329793895206056414");

    /// COFACTOR =
    /// 86482221941698704497288378992285180119495364068003923046442785886272123124361700722982503222189455144364945735564951561028
    #[rustfmt::skip]
    const COFACTOR: &'static [u64] = &[
        0x5657b9b57b942344,
        0x84f9a65f3bd54eaf,
        0x5ea4214e35cd127,
        0xe3cbcbc14ec1501d,
        0xf196cb845a3092ab,
        0x7e14627ad0e19017,
        0x217db4,
    ];

    /// COFACTOR^(-1) mod r =
    /// 163276846538158998893990986356139314746223949404500031940624325017036397274793417940375498603127780919653358641788
    #[rustfmt::skip]
    const COFACTOR_INV: Fr = field_new!(Fr, "163276846538158998893990986356139314746223949404500031940624325017036397274793417940375498603127780919653358641788");

    /// AFFINE_GENERATOR_COEFFS = (G1_GENERATOR_X, G1_GENERATOR_Y)
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField) =
        (G1_GENERATOR_X, G1_GENERATOR_Y);
}

/// G1_GENERATOR_X =
/// 5511163824921585887915590525772884263960974614921003940645351443740084257508990841338974915037175497689287870585840954231884082785026301437744745393958283053278991955159266640440849940136976927372133743626748847559939620888818486853646
#[rustfmt::skip]
pub const G1_GENERATOR_X: Fq = field_new!(Fq, "5511163824921585887915590525772884263960974614921003940645351443740084257508990841338974915037175497689287870585840954231884082785026301437744745393958283053278991955159266640440849940136976927372133743626748847559939620888818486853646");

/// G1_GENERATOR_Y =
/// 7913123550914612057135582061699117755797758113868200992327595317370485234417808273674357776714522052694559358668442301647906991623400754234679697332299689255516547752391831738454121261248793568285885897998257357202903170202349380518443
#[rustfmt::skip]
pub const G1_GENERATOR_Y: Fq = field_new!(Fq, "7913123550914612057135582061699117755797758113868200992327595317370485234417808273674357776714522052694559358668442301647906991623400754234679697332299689255516547752391831738454121261248793568285885897998257357202903170202349380518443");